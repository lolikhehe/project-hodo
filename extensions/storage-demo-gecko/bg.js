// Gecko-style (MV2) — storage permission + storage.local dispatch in hodo

// Firefox-style `browser.*` APIs return promises; hodo maps them onto the
// same dispatch table as `chrome.*`.
browser.storage.local.set({ theme: "dark" })
  .then(function () {
    console.log("Theme saved to browser.storage.local");
    return browser.storage.local.get(["theme"]);
  })
  .then(function (result) {
    console.log("Theme read back:", result.theme);
  })
  .catch(function (error) {
    console.error("[storage-demo-gecko] background failed:", error);
  });
