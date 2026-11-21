# Async Chrome

MV3 demo for the hodo extension bridge. The background service worker mixes
`await chrome.storage.*` (Promise style) with the classic callback style over
the same dispatch table, and listens for `chrome.alarms.onAlarm`.
