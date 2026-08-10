import { ref } from "vue";

const HISTORY_KEY = "flare:history";
const HISTORY_MAX = 50;

// 历史记录(localStorage 持久化)
export function useHistory() {
  const history = ref(loadHistory());

  function loadHistory() {
    try {
      return JSON.parse(localStorage.getItem(HISTORY_KEY) || "[]");
    } catch {
      return [];
    }
  }
  function persistHistory() {
    localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value));
  }
  function saveToHistory(data) {
    history.value.unshift({ id: Date.now(), time: Date.now(), ...data });
    if (history.value.length > HISTORY_MAX) history.value.length = HISTORY_MAX;
    persistHistory();
  }
  function removeSingleHistory(item) {
    history.value = history.value.filter((h) => h.id !== item.id);
    persistHistory();
  }
  function clearHistory() {
    history.value = [];
    persistHistory();
  }

  return { history, saveToHistory, removeSingleHistory, clearHistory };
}
