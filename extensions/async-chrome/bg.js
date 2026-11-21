// MV3 service worker: one dispatch table, async/await and callbacks mixed.

(async function main() {
  await chrome.storage.local.set({ mode: "async", startedAt: Date.now() });
  const { mode } = await chrome.storage.local.get(["mode"]);
  await chrome.storage.session.set({ lastMode: mode });
  console.log("[async-chrome] mode=" + mode);
})().catch(function (error) {
  console.error("[async-chrome] background failed:", error);
});

// Same API, callback style; `chrome.runtime.lastError` is set on failure.
chrome.storage.local.get(["mode"], function (result) {
  chrome.storage.local.set({ callbackMirror: result.mode }, function () {
    if (chrome.runtime.lastError) {
      console.error("[async-chrome] callback failed:", chrome.runtime.lastError);
    }
  });
});

chrome.alarms.onAlarm.addListener(function (alarm) {
  chrome.storage.local.set({ lastAlarm: alarm.name });
});
