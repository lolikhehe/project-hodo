# Hello Gecko

A Firefox/Gecko-style MV2 demo extension for the hodo browser.

hodo honors `browser_specific_settings.gecko.id` as the extension identity. It accepts MV2 `background.scripts`, and maps `browser.*` calls to the same dispatch table as `chrome.*`.

JS execution for backgrounds works via the hodo-js boa bridge (`run_background_scripts`).
