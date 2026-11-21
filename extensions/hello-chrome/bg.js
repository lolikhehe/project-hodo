// Chrome-style (MV3) — loaded by hodo's cross-compat extension host

chrome.storage.local.set({ greeting: "Hello from a Chrome-style MV3 extension" }, function () {
  console.log("Greeting saved to chrome.storage.local");

  chrome.storage.local.get(["greeting"], function (result) {
    console.log("Greeting read back:", result.greeting);
  });

  chrome.tabs.query({ active: true, currentWindow: true }, function (tabs) {
    if (tabs && tabs.length > 0) {
      console.log("Active tab:", tabs[0].url);
    }
  });
});
