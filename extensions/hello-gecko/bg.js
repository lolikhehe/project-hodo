// Gecko-style (MV2, browser_specific_settings) — loaded by hodo's cross-compat extension host

// Firefox-style extensions use the `browser.*` namespace, whose APIs return
// promises. hodo maps these onto the same dispatch table as `chrome.*`.
async function main() {
  await browser.storage.local.set({ lastStarted: Date.now(), greeting: "Hello Gecko" });

  const { greeting, lastStarted } = await browser.storage.local.get(["greeting", "lastStarted"]);
  console.log(`[hello-gecko] ${greeting}, first run at ${lastStarted}`);

  const tabs = await browser.tabs.query({ active: true, currentWindow: true });
  for (const tab of tabs) {
    await browser.storage.local.set({ lastActiveTab: tab.url || null });
  }
}

main().catch((error) => console.error("[hello-gecko] background failed:", error));
