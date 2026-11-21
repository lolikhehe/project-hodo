# Messages Demo
Gecko-style (MV2) demo: background replies to `browser.runtime.onMessage` with a value read from `browser.storage.local`.
hodo honors `browser_specific_settings.gecko.id`, accepts MV2 `background.scripts`, and maps `browser.*` to the storage dispatch (JS execution works via the hodo-js boa bridge, `run_background_scripts`).
