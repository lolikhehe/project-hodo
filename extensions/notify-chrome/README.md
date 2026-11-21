# Notify Demo
Chrome-style (MV3) demo: background service worker fires a basic notification on startup and on `chrome.action.onClicked`.
hodo parses the MV3 chrome dialect and honors the `notifications` permission (JS execution works via the hodo-js boa bridge, `run_background_scripts`).
