// Gecko-style (MV2) - background replies to messages with a stored value

browser.runtime.onMessage.addListener(function (message) {
  return browser.storage.local.get(["reply"]).then(function (result) {
    return result.reply || "no reply stored";
  });
});
