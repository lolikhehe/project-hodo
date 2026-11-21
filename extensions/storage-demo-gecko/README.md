# Storage Demo Gecko
A Firefox/Gecko-style MV2 demo: its background writes and reads `{"theme":"dark"}` via `browser.storage.local.set/get` promise chains.
hodo honors `browser_specific_settings.gecko.id`, accepts MV2 `background.scripts`, and maps `browser.*` to the same storage dispatch table (JS execution on the roadmap).
