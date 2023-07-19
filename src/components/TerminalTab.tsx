import { createSignal } from "solid-js";
import TerminalScreen from "./TerminalScreen";

function TerminalTab() {
  const [tabs, setTabs] = createSignal([{ id: 0 }]); // Array of tab objects
  const [activeTab, setActiveTab] = createSignal(0); // Index of the active tab

  const addTab = () => {
    const newTabId = tabs().length;
    setTabs([...tabs(), { id: newTabId }]);
    setActiveTab(newTabId);
  };

  const removeTab = (tabId: number) => {
    const newTabs = tabs().filter((tab) => tab.id !== tabId);
    setTabs(newTabs);

    // Set the active tab to the last tab if the currently active tab is removed
    if (activeTab() === tabId && newTabs.length > 0) {
      setActiveTab(newTabs[newTabs.length - 1].id);
      console.log(activeTab());
    }
  };

  const switchTab = (tabId: number) => {
    setActiveTab(tabId);
    console.log("Switched to tab", tabId);
  };
  console.log("activeTab", activeTab());
  return (
    <div class="bg-gray-700">
      <ul class="flex space-x-2">
        {tabs().map((tab) => (
          <li
            key={tab.id}
            class={`cursor-pointer py-2 px-4 ${
              activeTab() === tab.id ? "bg-gray-300" : "bg-gray-200"
            }`}
            onClick={() => switchTab(tab.id)}
          >
            Tab {tab.id + 1}
            <button
              class="ml-2 text-red-600 hover:text-red-800"
              onClick={() => removeTab(tab.id)}
            >
              X
            </button>
          </li>
        ))}
        <li
          class="cursor-pointer py-2 px-4 bg-gray-200 hover:bg-gray-300"
          onClick={addTab}
        >
          + New Tab
        </li>
      </ul>
      <div class="mt-4">
        {tabs().map((tab) => (
          <div key={tab.id} class={tab.id === activeTab() ? "block" : "hidden"}>
            <TerminalScreen />
          </div>
        ))}
      </div>
    </div>
  );
}

export default TerminalTab;
