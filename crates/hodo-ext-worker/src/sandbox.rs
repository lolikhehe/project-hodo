//! Wave-1 defence in depth. Linux gets three layers; other targets are a no-op
//! because the broker is still the authoritative permission gate.
//!
//! Linux CI must run `cargo test -p hodo-ext-worker --test isolation_e2e`
//! before merging any change to the syscall tables: seccomp is verified by
//! actually spawning the worker on a Linux kernel, not by the Windows tests.

/// Apply all available restrictions, reporting at most one log line.
///
/// # Errors
/// A *required* Linux layer failing to install is fatal; an old kernel without
/// Landlock support is explicitly not fatal (see [`landlock_fs`]). The filter
/// is installed before Hello, so a sandbox failure can never bind a channel.
#[cfg(target_os = "linux")]
pub fn apply(log: impl Fn(String)) -> Result<(), String> {
    landlock_fs(&log)?;
    if let Some(line) = seccomp_allowlist()? {
        log(line);
    }
    rlimits()?;
    log("sandbox: landlock + seccomp + rlimits active".to_string());
    Ok(())
}

/// Non-Linux wave 1: no sandbox primitives, one diagnostic line.
#[cfg(not(target_os = "linux"))]
pub fn apply(log: impl Fn(String)) -> Result<(), String> {
    log("sandbox: no-op on this target (wave 1; broker gates permissions)".to_string());
    Ok(())
}

#[cfg(any(target_os = "linux", test))]
const ENOSYS: i32 = 38;
#[cfg(any(target_os = "linux", test))]
const EOPNOTSUPP: i32 = 95;

/// Which Landlock operation failed. Only the two kernel-entry syscalls can
/// prove the kernel lacks Landlock; a failure anywhere else is real.
#[cfg(any(target_os = "linux", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LandlockCall {
    /// `landlock_create_ruleset(2)` — the kernel-entry probe.
    CreateRuleset,
    /// `landlock_restrict_self(2)` — enforcement of the prepared ruleset.
    RestrictSelf,
    /// `handle_access`, `add_rule`, `prctl(PR_SET_NO_NEW_PRIVS)`, flag setup...
    Other,
}

/// Only ENOSYS/EOPNOTSUPP from the `create`/`restrict_self` syscalls mean "this
/// kernel has no Landlock". An errno from any other call — or EACCES/EINVAL/
/// EPERM from those calls — is a real failure and must stay fatal: classifying
/// it as "unsupported" would boot an unrestricted worker.
#[cfg(any(target_os = "linux", test))]
fn errno_means_no_landlock(call: LandlockCall, errno: Option<i32>) -> Option<i32> {
    match call {
        LandlockCall::CreateRuleset | LandlockCall::RestrictSelf => {
            errno.filter(|&errno| errno == ENOSYS || errno == EOPNOTSUPP)
        }
        LandlockCall::Other => None,
    }
}

/// Landlock with `HardRequirement` (BestEffort is forbidden here): a kernel
/// that supports Landlock must enforce the rule or we refuse to start. On a
/// kernel that does not implement Landlock at all we log and continue, because
/// the broker still enforces permissions and a 2020-era kernel must still boot.
#[cfg(target_os = "linux")]
fn landlock_fs(log: &impl Fn(String)) -> Result<(), String> {
    use landlock::{
        Access, AccessFs, CompatLevel, Compatible, PathBeneath, PathFd, RestrictionStatus, Ruleset,
        RulesetAttr, RulesetCreatedAttr, RulesetError, RulesetStatus, ABI,
    };

    let cwd = std::env::current_dir().map_err(|err| format!("sandbox cwd: {err}"))?;
    let dir = PathFd::new(&cwd).map_err(|err| format!("sandbox cwd fd: {err}"))?;
    let access = AccessFs::from_all(ABI::V1);
    let result: Result<RestrictionStatus, RulesetError> = (|| {
        let ruleset = Ruleset::default()
            .set_compatibility(CompatLevel::HardRequirement)
            .handle_access(access)?
            .create()?
            .set_compatibility(CompatLevel::HardRequirement)
            .add_rule(
                PathBeneath::new(dir, access).set_compatibility(CompatLevel::HardRequirement),
            )?;
        ruleset.restrict_self()
    })();

    match result {
        Ok(status) if status.ruleset == RulesetStatus::FullyEnforced => Ok(()),
        Ok(status) => Err(format!("landlock status {:?}", status.ruleset)),
        Err(err) => match kernel_lacks_landlock(&err) {
            Some(errno) => {
                log(format!(
                    "sandbox: kernel has no Landlock (errno {errno}); continuing (broker gates permissions)"
                ));
                Ok(())
            }
            None => Err(format!("sandbox landlock: {err}")),
        },
    }
}

/// Classify a Landlock failure by the syscall that produced it. Only the raw
/// errno of `create`/`restrict_self` can mean "kernel has no Landlock"; an
/// ENOSYS buried anywhere else in the chain (error text included) is a real
/// failure and stays fatal.
#[cfg(target_os = "linux")]
fn kernel_lacks_landlock(err: &landlock::RulesetError) -> Option<i32> {
    use landlock::{CreateRulesetError, RestrictSelfError, RulesetError};

    let (call, errno) = match err {
        RulesetError::CreateRuleset(CreateRulesetError::CreateRulesetCall { source, .. }) => {
            (LandlockCall::CreateRuleset, source.raw_os_error())
        }
        RulesetError::RestrictSelf(RestrictSelfError::RestrictSelfCall { source, .. }) => {
            (LandlockCall::RestrictSelf, source.raw_os_error())
        }
        // `SetNoNewPrivsCall`, `HandleAccesses`, `AddRules`, `RestrictSelfFlags`
        // and `Scope` are setup/configuration failures, not a missing kernel.
        _ => (LandlockCall::Other, None),
    };
    errno_means_no_landlock(call, errno)
}

/// Syscalls permitted after the filter is installed. Numbers are from the
/// x86_64 `syscall_64.tbl` and the generic `syscall.tbl` (aarch64/riscv64).
/// Every entry exists because Rust std / seccompiler / the worker use it after
/// `apply_filter`; there is deliberately no `socket`, `connect`, `open`,
/// `execve`, `clone`, `kill` or `ptrace`. Default action stays `Errno(38)`.
#[cfg(any(target_os = "linux", test))]
const X86_64_ALLOWED: &[i64] = &[
    0,   // read: IPC frames from stdin, /dev/urandom fallback
    1,   // write: IPC frames to stdout, diagnostics to stderr
    3,   // close: drop /dev/urandom fallback fd and pipe ends
    8,   // lseek: std file-offset handling on the fallback fd
    9,   // mmap: allocator and Boa heap
    10,  // mprotect: stack/allocator guard pages
    11,  // munmap: allocator
    12,  // brk: allocator
    13,  // rt_sigaction: std installs SIGPIPE/SIGSEGV handlers
    14,  // rt_sigprocmask: std and seccompiler mask signals during setup
    15,  // rt_sigreturn: return from any delivered signal
    16,  // ioctl: stdout isatty check (line buffering); harmless if refused
    28,  // madvise: allocator page hints
    39,  // getpid: std process/thread ids
    60,  // exit: worker's fatal exits
    72,  // fcntl: std fd flags on the inherited IPC pipes
    157, // prctl: seccompiler setup / no_new_privs
    158, // arch_prctl: x86_64 TLS (FS base) during startup
    160, // setrlimit: rustix fallback on older kernels
    186, // gettid: std thread id
    202, // futex: locks and thread parking
    204, // sched_getaffinity: std::thread::available_parallelism
    218, // set_tid_address: std thread startup clears child tid on exit
    228, // clock_gettime: Instant, deadlines, timers
    231, // exit_group: process exit
    257, // openat: /dev/urandom fallback when getrandom is unavailable
    262, // newfstatat: stat the fallback path before opening
    273, // set_robust_list: std robust-futex thread startup
    302, // prlimit64: rustix setrlimit on modern kernels
    317, // seccomp: seccompiler::apply_filter
    318, // getrandom: std hash seeds / session token
];

/// aarch64 and riscv64 share the generic syscall table. Note the numbers that
/// differ from x86_64 (set_robust_list=99, sched_getaffinity=123, ...).
#[cfg(any(target_os = "linux", test))]
const GENERIC_ALLOWED: &[i64] = &[
    25,  // fcntl: std fd flags
    29,  // ioctl: stdout isatty check
    56,  // openat: /dev/urandom fallback
    57,  // close: drop fallback fd and pipe ends
    62,  // lseek: file-offset handling on the fallback fd
    63,  // read: IPC frames from stdin, /dev/urandom fallback
    64,  // write: IPC frames and diagnostics
    79,  // newfstatat: stat the fallback path before opening
    93,  // exit: worker's fatal exits
    94,  // exit_group: process exit
    96,  // set_tid_address: std thread startup
    98,  // futex: locks and thread parking
    99,  // set_robust_list: std robust-futex thread startup
    113, // clock_gettime: Instant, deadlines, timers
    123, // sched_getaffinity: std::thread::available_parallelism
    132, // sigaltstack: std signal alternate stack
    134, // rt_sigaction: std installs SIGPIPE/SIGSEGV handlers
    135, // rt_sigprocmask: std and seccompiler signal masking
    139, // rt_sigreturn: return from any delivered signal
    167, // prctl: seccompiler setup / no_new_privs
    172, // getpid: std process id
    178, // gettid: std thread id
    214, // brk: allocator
    215, // munmap: allocator
    216, // mremap: allocator growth
    222, // mmap: allocator and Boa heap
    226, // mprotect: guard pages
    233, // madvise: allocator page hints
    261, // prlimit64: rustix setrlimit
    277, // seccomp: seccompiler::apply_filter
    278, // getrandom: std hash seeds / session token
];

/// Allow only the syscalls the worker itself needs. Denied calls return
/// `ENOSYS` rather than killing the process: the syscall never succeeds, so
/// the filter stays fail-closed while an overlooked startup syscall degrades
/// to an error instead of a crash. The filter is installed before Hello.
#[cfg(target_os = "linux")]
fn seccomp_allowlist() -> Result<Option<String>, String> {
    use std::collections::BTreeMap;

    use seccompiler::{apply_filter, BpfProgram, SeccompAction, SeccompFilter, TargetArch};

    let arch: TargetArch = std::env::consts::ARCH
        .try_into()
        .map_err(|_| "sandbox: unknown seccomp architecture".to_string())?;
    let allowed: &[i64] = match arch {
        TargetArch::x86_64 => X86_64_ALLOWED,
        TargetArch::aarch64 | TargetArch::riscv64 => GENERIC_ALLOWED,
    };
    let rules: BTreeMap<i64, Vec<_>> = allowed
        .iter()
        .copied()
        .map(|number| (number, Vec::new()))
        .collect();
    let filter = SeccompFilter::new(rules, SeccompAction::Errno(38), SeccompAction::Allow, arch)
        .map_err(|err| format!("sandbox seccomp filter: {err}"))?;
    let program: BpfProgram = filter
        .try_into()
        .map_err(|err| format!("sandbox seccomp bpf: {err}"))?;
    apply_filter(&program).map_err(|err| format!("sandbox seccomp apply: {err}"))?;
    Ok(Some("sandbox: seccomp allowlist active".to_string()))
}

/// 512 MiB address space, 30 s CPU, 64 open files.
#[cfg(target_os = "linux")]
fn rlimits() -> Result<(), String> {
    use rustix::process::{setrlimit, Resource, Rlimit};

    let fixed = |value: u64| Rlimit {
        current: Some(value),
        maximum: Some(value),
    };
    setrlimit(Resource::As, fixed(512 * 1024 * 1024))
        .map_err(|err| format!("sandbox RLIMIT_AS: {err}"))?;
    setrlimit(Resource::Cpu, fixed(30)).map_err(|err| format!("sandbox RLIMIT_CPU: {err}"))?;
    setrlimit(Resource::Nofile, fixed(64))
        .map_err(|err| format!("sandbox RLIMIT_NOFILE: {err}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errno_classification_is_exact_and_fail_closed() {
        // Only the two kernel-entry syscalls with the two "not implemented"
        // errnos classify as a kernel without Landlock.
        for call in [LandlockCall::CreateRuleset, LandlockCall::RestrictSelf] {
            assert_eq!(errno_means_no_landlock(call, Some(ENOSYS)), Some(38));
            assert_eq!(errno_means_no_landlock(call, Some(EOPNOTSUPP)), Some(95));
            assert_eq!(errno_means_no_landlock(call, None), None);
        }
        // EACCES/EINVAL/EPERM and friends are real failures, even if their
        // Debug text happens to say "Unsupported". They must stay fatal.
        for errno in [Some(13), Some(22), Some(1), None] {
            for call in [LandlockCall::CreateRuleset, LandlockCall::RestrictSelf] {
                assert_eq!(errno_means_no_landlock(call, errno), None);
            }
            // An ENOSYS/EOPNOTSUPP from anywhere else in the error chain is
            // not evidence the kernel lacks Landlock.
            assert_eq!(errno_means_no_landlock(LandlockCall::Other, errno), None);
        }
        assert_eq!(
            errno_means_no_landlock(LandlockCall::Other, Some(ENOSYS)),
            None
        );
        assert_eq!(
            errno_means_no_landlock(LandlockCall::Other, Some(EOPNOTSUPP)),
            None
        );
    }

    #[test]
    fn seccomp_tables_cover_rust_startup_without_duplicates() {
        let required_x86_64 = [
            0, 1, 3, 9, 10, 11, 12, 13, 14, 15, 39, 60, 72, 157, 158, 160, 186, 202, 204, 218, 228,
            231, 257, 262, 273, 302, 317, 318,
        ];
        for number in required_x86_64 {
            assert!(X86_64_ALLOWED.contains(&number), "x86_64 missing {number}");
        }
        let required_generic = [
            25, 29, 56, 57, 63, 64, 93, 94, 96, 98, 99, 113, 123, 132, 134, 135, 139, 167, 172,
            178, 214, 215, 222, 226, 261, 277, 278,
        ];
        for number in required_generic {
            assert!(
                GENERIC_ALLOWED.contains(&number),
                "generic/aarch64 missing {number}"
            );
        }
        for table in [X86_64_ALLOWED, GENERIC_ALLOWED] {
            for (index, number) in table.iter().enumerate() {
                assert!(
                    !table[..index].contains(number),
                    "duplicate syscall {number} in table"
                );
            }
        }
    }
}
