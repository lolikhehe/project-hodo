# Tabs Demo Chrome
A Chrome-style (MV3) demo: its background service worker opens the hodo GitHub repo via `chrome.tabs.create`.
hodo parses the MV3 chrome dialect, honors the `tabs` permission, and dispatches the TabsCreate API cross-compat (JS execution works via the hodo-js boa bridge, `run_background_scripts`).
