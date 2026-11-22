# Entity-site fixture

Headless-render regression fixture for two live-site parse/render bugs.

```cmd
cargo run --release -p hodo-shell -- --screenshot examples/entity-site/index.html "%TEMP%\opencode\entity.png"
```

Expected screenshot:

- Every entity is decoded exactly once: `Tom & Jerry`, `"double"` and `'single'`,
  `/search?q=a&b=2&c=3`, literal `<a href="/x">link</a>` and
  `<div class="box">x</div>` text, `ABC` and the grinning-face emoji.
- The red `<noscript>` and `<template>` warnings do **not** appear.
- The final `After noscript/template: still visible` row does appear.
