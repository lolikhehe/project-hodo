# Filter list attribution and licensing

This directory vendors two community-maintained filter lists used by hodo's
ad/tracker blocker. They are stored as plain text so `hodo-adblock` can parse
them at startup without a network connection.

## EasyList (advertising)

- Source: <https://easylist-downloads.adblockplus.org/easylist.txt>
- Mirror: <https://raw.githubusercontent.com/easylist/easylist/master/easylist.txt>
- Project home: <https://easylist.to/>
- License: <https://easylist.to/pages/licence.html>
- vendored file: `easylist.txt`
- Retrieved: 2026-11-22

EasyList is free for use and redistribution provided attribution is retained,
as recorded in the license page above. This file is that attribution.

## EasyPrivacy (tracking)

- Source: <https://easylist-downloads.adblockplus.org/easyprivacy.txt>
- Mirror: <https://raw.githubusercontent.com/easylist/easylist/master/easyprivacy.txt>
- Project home: <https://easylist.to/>
- License: <https://easylist.to/pages/licence.html>
- vendored file: `easyprivacy.txt`
- Retrieved: 2026-11-22

EasyPrivacy is published under the same EasyList license and attribution terms.

## Updating

Re-download both files from the source URLs on a schedule (the lists advertise
a 12-hour expiry). Keep this attribution file unchanged. Filter parsing
tolerates new/unsupported syntax: unsupported lines are skipped and counted
rather than failing the whole list.
