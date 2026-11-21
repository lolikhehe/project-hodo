// Gecko-style MV2: every `browser.*` API returns a Promise.

browser.storage.local
  .set({ flavor: "gecko-promise" })
  .then(function () {
    return browser.storage.local.get(["flavor"]);
  })
  .then(function (result) {
    return browser.storage.session.set({ lastFlavor: result.flavor });
  })
  .then(function () {
    return browser.tabs.query({ active: true, currentWindow: true });
  })
  .then(function (tabs) {
    console.log("[promises-gecko] tabs=" + tabs.length);
  })
  .catch(function (error) {
    console.error("[promises-gecko] background failed:", error);
  });

// A returned Promise is a valid onMessage response; first response wins.
browser.runtime.onMessage.addListener(function (message) {
  return browser.storage.local.get(["flavor"]).then(function (result) {
    return result.flavor;
  });
});

// Minimal Port surface used by the upcoming popup wiring.
browser.runtime.onConnect.addListener(function (port) {
  port.onMessage.addListener(function (message) {
    browser.storage.local.set({ portMessage: message });
  });
});
