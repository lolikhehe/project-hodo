// Chrome-style (MV3) - background service worker demo of notifications

chrome.notifications.create({
  type: "basic",
  title: "Notify Demo",
  message: "Hello from a hodo extension notification"
});

chrome.action.onClicked.addListener(function (tab) {
  chrome.notifications.create({
    type: "basic",
    title: "Action clicked",
    message: "You clicked the Notify Demo action button"
  });
});
