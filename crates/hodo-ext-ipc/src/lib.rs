//! The frozen M3 extension IPC wire format: codec and validation only.
//!
//! No sockets, pipes or processes live here. A frame is
//! `u32_le(N) | [VERSION, serde_json::Message]` where `N` counts the bytes of
//! the bracketed body. Length and depth are checked *before* any allocation
//! that an untrusted peer could influence.

use std::collections::HashSet;
use std::io::{self, Read};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Wire version carried as the first body byte.
pub const VERSION: u8 = 1;
/// Maximum encoded body of one frame.
pub const MAX_MSG_BYTES: usize = hodo_types::limits::MAX_MSG_BYTES;
/// Maximum nesting depth of any JSON value carried in a frame.
pub const MAX_MSG_DEPTH: usize = hodo_types::limits::MAX_MSG_DEPTH;
/// Maximum number of live broker contexts.
pub const MAX_EXT_CONTEXTS: usize = hodo_types::limits::MAX_EXT_CONTEXTS;
/// Maximum number of unanswered calls the worker keeps outstanding.
pub const MAX_OUTSTANDING_CALLS: usize = 256;

/// The only messages that may cross the broker/worker boundary.
///
/// Direction is by convention: `Hello` and `ApiCall` are worker-to-broker,
/// `ApiResponse` and `Shutdown` are broker-to-worker, `Event` is bidirectional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Message {
    /// First frame from the worker, immediately after `env_clear`.
    Hello {
        token: String,
        manifest_digest: String,
    },
    /// A shadow-registry call forwarded by the worker.
    ApiCall {
        id: u64,
        method: String,
        params: Value,
    },
    /// Reply to an `ApiCall`; exactly one of the two payload fields is `Some`.
    ApiResponse {
        id: u64,
        /// Present `null` is a real result; omitting `None` keeps absence distinct.
        #[serde(
            default,
            deserialize_with = "deserialize_some_value",
            skip_serializing_if = "Option::is_none"
        )]
        result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// Browser event, or a worker log/ready notification.
    Event { name: String, payload: Value },
    /// Broker asks the worker to drain and exit cleanly.
    Shutdown,
}

/// Deserialize a *present* field value into `Some`, including JSON `null`.
/// serde's default `Option<Value>` decoding maps `null` to `None`, which would
/// make a valid `result: null` indistinguishable from an absent result.
fn deserialize_some_value<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Value::deserialize(deserializer).map(Some)
}

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("frame length is zero")]
    ZeroLength,
    #[error("frame length {0} exceeds the {MAX_MSG_BYTES} byte cap")]
    TooLarge(usize),
    #[error("frame is truncated")]
    Truncated,
    #[error("frame body is empty")]
    EmptyBody,
    #[error("unsupported protocol version {0}")]
    BadVersion(u8),
    #[error("message nesting exceeds {MAX_MSG_DEPTH}")]
    TooDeep,
    #[error("invalid message: {0}")]
    Invalid(String),
    #[error("too many outstanding calls (cap {MAX_OUTSTANDING_CALLS})")]
    TooManyOutstanding,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Encode one message into a self-delimiting frame.
///
/// Panics only if the caller tries to encode a body larger than
/// [`MAX_MSG_BYTES`]; the wire format cannot represent such a frame.
#[must_use]
pub fn encode(msg: &Message) -> Vec<u8> {
    let mut body = Vec::with_capacity(128);
    body.push(VERSION);
    serde_json::to_writer(&mut body, msg).expect("serializing a Message cannot fail");
    assert!(
        body.len() <= MAX_MSG_BYTES,
        "message body exceeds the IPC cap"
    );
    let mut frame = Vec::with_capacity(body.len() + 4);
    frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
    frame.extend_from_slice(&body);
    frame
}

/// Decode one complete frame (length header included).
///
/// # Errors
/// Rejects a missing/zero/oversized length, a truncated body, an unknown
/// version, an unknown/missing `t` tag, more than [`MAX_MSG_DEPTH`] levels of
/// JSON nesting, and `ApiResponse`s without exactly one of `result`/`error`.
pub fn decode(frame: &[u8]) -> Result<Message, IpcError> {
    if frame.len() < 4 {
        return Err(IpcError::Truncated);
    }
    let len = u32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;
    if len == 0 {
        return Err(IpcError::ZeroLength);
    }
    // The cap is checked against the *advertised* length before any allocation.
    if len > MAX_MSG_BYTES {
        return Err(IpcError::TooLarge(len));
    }
    let body_end = 4usize.checked_add(len).ok_or(IpcError::TooLarge(len))?;
    if frame.len() < body_end {
        return Err(IpcError::Truncated);
    }
    decode_body(&frame[4..body_end])
}

fn decode_body(body: &[u8]) -> Result<Message, IpcError> {
    let (&version, json) = body.split_first().ok_or(IpcError::EmptyBody)?;
    if version != VERSION {
        return Err(IpcError::BadVersion(version));
    }
    // serde's internally-tagged enum rejects a missing or unknown `t`, and its
    // own recursion guard (128) stops pathological JSON before we look at it.
    let msg: Message = serde_json::from_slice(json)?;
    check_message(&msg)?;
    Ok(msg)
}

fn check_message(msg: &Message) -> Result<(), IpcError> {
    if let Message::ApiResponse { result, error, .. } = msg {
        if result.is_some() == error.is_some() {
            return Err(IpcError::Invalid(
                "api response needs exactly one of result and error".to_string(),
            ));
        }
    }
    match msg {
        Message::ApiCall { params, .. }
        | Message::Event {
            payload: params, ..
        } => check_value(params, 0),
        Message::ApiResponse {
            result: Some(params),
            ..
        } => check_value(params, 0),
        Message::Hello { .. } | Message::Shutdown | Message::ApiResponse { .. } => Ok(()),
    }
}

fn check_value(value: &Value, depth: usize) -> Result<(), IpcError> {
    if depth > MAX_MSG_DEPTH {
        return Err(IpcError::TooDeep);
    }
    match value {
        Value::Array(items) => {
            for item in items {
                check_value(item, depth + 1)?;
            }
        }
        Value::Object(map) => {
            for (_, item) in map {
                check_value(item, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Buffered reader over any `Read` source. The advertised length is capped
/// *before* the body buffer is allocated, so a hostile peer cannot make us
/// reserve 4 GiB.
pub struct FrameReader<R: Read> {
    inner: R,
}

impl<R: Read> FrameReader<R> {
    #[must_use]
    pub fn new(inner: R) -> Self {
        Self { inner }
    }

    /// Read the next frame. `Ok(None)` means a clean EOF before a length header.
    ///
    /// # Errors
    /// Propagates I/O errors and protocol errors from [`decode_body`].
    pub fn read_frame(&mut self) -> Result<Option<Message>, IpcError> {
        let mut header = [0u8; 4];
        match self.inner.read_exact(&mut header) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(err) => return Err(err.into()),
        }
        let len = u32::from_le_bytes(header) as usize;
        if len == 0 {
            return Err(IpcError::ZeroLength);
        }
        if len > MAX_MSG_BYTES {
            return Err(IpcError::TooLarge(len));
        }
        let mut body = vec![0u8; len];
        self.inner.read_exact(&mut body)?;
        decode_body(&body).map(Some)
    }

    /// Give the underlying reader back.
    #[must_use]
    pub fn into_inner(self) -> R {
        self.inner
    }
}

/// Tracks the worker-side handshake state.
#[derive(Debug, Default)]
pub struct HandshakeState {
    saw_hello: bool,
    ready: bool,
}

impl HandshakeState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a message and reject a repeated `Hello`.
    ///
    /// # Errors
    /// Returns [`IpcError::Invalid`] on a duplicate `Hello`.
    pub fn observe(&mut self, msg: &Message) -> Result<(), IpcError> {
        if matches!(msg, Message::Hello { .. }) {
            if self.saw_hello {
                return Err(IpcError::Invalid("duplicate hello".to_string()));
            }
            self.saw_hello = true;
        }
        Ok(())
    }

    /// Mark the ready event as received.
    pub fn mark_ready(&mut self) {
        self.ready = true;
    }

    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    #[must_use]
    pub fn saw_hello(&self) -> bool {
        self.saw_hello
    }

    /// Before `__hodo_ready`, only `Hello` may travel worker-to-broker.
    ///
    /// # Errors
    /// Returns [`IpcError::Invalid`] for a message received ahead of the
    /// handshake.
    pub fn validate_worker_frame(&self, msg: &Message) -> Result<(), IpcError> {
        if !self.ready && !matches!(msg, Message::Hello { .. }) {
            return Err(IpcError::Invalid(
                "worker frame received before handshake".to_string(),
            ));
        }
        Ok(())
    }
}

/// Bounds the number of unanswered calls a worker holds at once.
#[derive(Debug, Default)]
pub struct PendingCalls {
    ids: HashSet<u64>,
}

impl PendingCalls {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// # Errors
    /// Returns [`IpcError::TooManyOutstanding`] at the cap, and
    /// [`IpcError::Invalid`] if the id is already pending.
    pub fn insert(&mut self, id: u64) -> Result<(), IpcError> {
        if !self.ids.insert(id) {
            return Err(IpcError::Invalid(format!("duplicate call id {id}")));
        }
        if self.ids.len() > MAX_OUTSTANDING_CALLS {
            self.ids.remove(&id);
            return Err(IpcError::TooManyOutstanding);
        }
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> bool {
        self.ids.remove(&id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    fn hello() -> Message {
        Message::Hello {
            token: "tok".to_string(),
            manifest_digest: "abcdef".to_string(),
        }
    }

    #[test]
    fn roundtrip_every_variant() {
        let messages = [
            hello(),
            Message::ApiCall {
                id: 7,
                method: "storage.local.set".to_string(),
                params: json!({"k": 1}),
            },
            Message::ApiResponse {
                id: 7,
                result: Some(json!({"ok": true})),
                error: None,
            },
            Message::ApiResponse {
                id: 8,
                result: None,
                error: Some("nope".to_string()),
            },
            Message::Event {
                name: "__hodo_ready".to_string(),
                payload: json!({"allowed_namespaces": ["storage"]}),
            },
            Message::Shutdown,
        ];
        for msg in messages {
            let frame = encode(&msg);
            assert_eq!(decode(&frame).unwrap(), msg);
        }
    }

    #[test]
    fn wire_tag_and_version_are_visible() {
        let frame = encode(&Message::Shutdown);
        assert_eq!(&frame[4..], b"\x01{\"t\":\"shutdown\"}");
    }

    #[test]
    fn oversized_length_is_rejected_before_allocating() {
        let huge = (MAX_MSG_BYTES as u32) + 1;
        let frame = huge.to_le_bytes();
        assert!(matches!(decode(&frame), Err(IpcError::TooLarge(n)) if n == huge as usize));
        let mut reader = FrameReader::new(Cursor::new(frame.to_vec()));
        assert!(matches!(reader.read_frame(), Err(IpcError::TooLarge(n)) if n == huge as usize));
    }

    #[test]
    fn zero_length_and_version_only_body_are_rejected() {
        assert!(matches!(decode(&[0, 0, 0, 0]), Err(IpcError::ZeroLength)));
        // Header plus one body byte: the version byte is 0, which is invalid.
        assert!(matches!(
            decode(&[1, 0, 0, 0, 0]),
            Err(IpcError::BadVersion(0))
        ));
        assert!(matches!(decode(&[1, 0, 0, 0]), Err(IpcError::Truncated)));
    }

    #[test]
    fn bad_version_is_rejected() {
        let mut frame = encode(&Message::Shutdown);
        frame[4] = 2;
        assert!(matches!(decode(&frame), Err(IpcError::BadVersion(2))));
    }

    #[test]
    fn truncated_frames_are_rejected() {
        assert!(matches!(decode(&[1, 0, 0]), Err(IpcError::Truncated)));
        let mut frame = encode(&hello());
        frame.pop();
        assert!(matches!(decode(&frame), Err(IpcError::Truncated)));
    }

    #[test]
    fn unknown_tag_is_rejected() {
        let json = br#"{"t":"bogus"}"#;
        let mut frame = Vec::new();
        frame.extend_from_slice(&((json.len() + 1) as u32).to_le_bytes());
        frame.push(VERSION);
        frame.extend_from_slice(json);
        assert!(matches!(decode(&frame), Err(IpcError::Json(_))));
    }

    #[test]
    fn depth_over_64_is_rejected() {
        let mut nested = json!(1);
        for _ in 0..=MAX_MSG_DEPTH {
            nested = json!([nested]);
        }
        let frame = encode(&Message::Event {
            name: "probe".to_string(),
            payload: nested,
        });
        assert!(matches!(decode(&frame), Err(IpcError::TooDeep)));
    }

    #[test]
    fn api_response_requires_exactly_one_payload() {
        let both = Message::ApiResponse {
            id: 1,
            result: Some(json!(1)),
            error: Some("e".to_string()),
        };
        assert!(decode(&encode(&both)).is_err());
        let neither = Message::ApiResponse {
            id: 1,
            result: None,
            error: None,
        };
        assert!(decode(&encode(&neither)).is_err());
        let ok = Message::ApiResponse {
            id: 1,
            result: Some(json!(1)),
            error: None,
        };
        assert_eq!(decode(&encode(&ok)).unwrap(), ok);
    }

    fn frame_from_json(json: &[u8]) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.extend_from_slice(&((json.len() + 1) as u32).to_le_bytes());
        frame.push(VERSION);
        frame.extend_from_slice(json);
        frame
    }

    #[test]
    fn null_result_is_present_and_accepted() {
        let reply = Message::ApiResponse {
            id: 9,
            result: Some(Value::Null),
            error: None,
        };
        let frame = encode(&reply);
        assert_eq!(
            std::str::from_utf8(&frame[5..]).unwrap(),
            r#"{"t":"api_response","id":9,"result":null}"#
        );
        // decode calls check_message; a null result must survive as Some(Null).
        assert_eq!(decode(&frame).unwrap(), reply);
    }

    #[test]
    fn error_only_response_is_accepted() {
        let reply = Message::ApiResponse {
            id: 10,
            result: None,
            error: Some("denied".to_string()),
        };
        let frame = encode(&reply);
        assert_eq!(
            std::str::from_utf8(&frame[5..]).unwrap(),
            r#"{"t":"api_response","id":10,"error":"denied"}"#
        );
        assert_eq!(decode(&frame).unwrap(), reply);
    }

    #[test]
    fn both_or_neither_payload_wire_frames_are_rejected() {
        let both = frame_from_json(br#"{"t":"api_response","id":1,"result":1,"error":"e"}"#);
        assert!(matches!(decode(&both), Err(IpcError::Invalid(_))));
        let neither = frame_from_json(br#"{"t":"api_response","id":1}"#);
        assert!(matches!(decode(&neither), Err(IpcError::Invalid(_))));
        // A present null result paired with an error is still both-fields.
        let both_null =
            frame_from_json(br#"{"t":"api_response","id":1,"result":null,"error":"e"}"#);
        assert!(matches!(decode(&both_null), Err(IpcError::Invalid(_))));
    }

    #[test]
    fn storage_local_set_roundtrips_through_ipc() {
        let call = Message::ApiCall {
            id: 42,
            method: "storage.local.set".to_string(),
            params: json!({"k": 1}),
        };
        match decode(&encode(&call)).unwrap() {
            Message::ApiCall { id, method, params } => {
                assert_eq!(id, 42);
                assert_eq!(method, "storage.local.set");
                assert_eq!(params, json!({"k": 1}));
            }
            other => panic!("expected api call, got {other:?}"),
        }

        // storage.local.set dispatches to ApiResult::Ok(Value::Null), which the
        // broker sends as `result: null` -- the frame the old decoder rejected.
        let reply = Message::ApiResponse {
            id: 42,
            result: Some(Value::Null),
            error: None,
        };
        let decoded = decode(&encode(&reply)).unwrap();
        assert_eq!(decoded, reply);
        assert!(matches!(
            decoded,
            Message::ApiResponse {
                result: Some(Value::Null),
                error: None,
                ..
            }
        ));
    }

    #[test]
    fn frame_reader_reads_a_stream_of_frames() {
        let mut stream = encode(&hello());
        stream.extend_from_slice(&encode(&Message::Shutdown));
        let mut reader = FrameReader::new(Cursor::new(stream));
        assert_eq!(reader.read_frame().unwrap(), Some(hello()));
        assert_eq!(reader.read_frame().unwrap(), Some(Message::Shutdown));
        assert_eq!(reader.read_frame().unwrap(), None);
    }

    #[test]
    fn duplicate_hello_is_detected() {
        let mut state = HandshakeState::new();
        assert!(state.observe(&hello()).is_ok());
        assert!(state.saw_hello());
        assert!(state.observe(&hello()).is_err());
    }

    #[test]
    fn pre_handshake_frames_are_rejected_until_ready() {
        let mut state = HandshakeState::new();
        let call = Message::ApiCall {
            id: 1,
            method: "tabs.list".to_string(),
            params: json!([]),
        };
        assert!(state.validate_worker_frame(&call).is_err());
        assert!(state.validate_worker_frame(&hello()).is_ok());
        state.mark_ready();
        assert!(state.is_ready());
        assert!(state.validate_worker_frame(&call).is_ok());
    }

    #[test]
    fn pending_calls_cap_is_enforced() {
        let mut pending = PendingCalls::new();
        for id in 0..MAX_OUTSTANDING_CALLS as u64 {
            assert!(pending.insert(id).is_ok());
        }
        assert!(matches!(
            pending.insert(MAX_OUTSTANDING_CALLS as u64),
            Err(IpcError::TooManyOutstanding)
        ));
        assert!(pending.remove(0));
        assert!(pending.insert(MAX_OUTSTANDING_CALLS as u64).is_ok());
        assert_eq!(pending.len(), MAX_OUTSTANDING_CALLS);
    }
}
