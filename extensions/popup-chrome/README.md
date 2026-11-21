# Popup Demo
Chrome-style (MV3) demo with a static `popup.html` popup and 16/48px PNG icons declared via `action.default_popup` and `icons`.
hodo parses the MV3 chrome dialect and records the popup + icon paths from the manifest (JS execution works via the hodo-js boa bridge, `run_background_scripts`).
