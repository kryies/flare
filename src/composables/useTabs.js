import { ref, reactive, computed, watch } from "vue";

const TABS_KEY = "flare:tabs";

// 多标签页管理(localStorage 持久化)
export function useTabs() {
  const tabs = reactive([makeTab(1)]);
  let nextId = 2;
  const activeTabId = ref(1);
  const activeTab = computed(() => tabs.find((t) => t.id === activeTabId.value));

  function makeForm() {
    return {
      method: "GET",
      url: "https://postman-echo.com/get",
      params: [{ key: "", value: "" }],
      headers: [{ key: "", value: "" }],
      bodyType: "none",
      rawBody: "",
      formData: [],
      urlencoded: [{ key: "", value: "" }],
      binaryFile: "",
    };
  }

  function makeTab(id) {
    return {
      id,
      name: `请求 ${id}`,
      form: makeForm(),
      response: null,
      error: "",
      loading: false,
      sentRequest: null,
    };
  }

  function createTab() {
    const id = nextId++;
    tabs.push(makeTab(id));
    activeTabId.value = id;
  }

  function closeTab(id) {
    if (tabs.length === 1) {
      Object.assign(tabs[0], makeTab(tabs[0].id));
      return;
    }
    const idx = tabs.findIndex((t) => t.id === id);
    if (idx === -1) return;
    tabs.splice(idx, 1);
    if (activeTabId.value === id) {
      activeTabId.value = tabs[Math.min(idx, tabs.length - 1)].id;
    }
  }

  function tabStatus(tab) {
    if (tab.loading) return "st-loading";
    if (tab.error) return "st-error";
    if (tab.response) {
      const s = tab.response.status;
      if (s >= 200 && s < 300) return "st-ok";
      if (s >= 400) return "st-error";
      return "st-other";
    }
    return "";
  }

  // 编辑标签名
  const editingTabId = ref(null);
  function startEdit(id) {
    editingTabId.value = id;
  }
  function finishEdit(tab) {
    if (!tab.name.trim()) tab.name = `请求 ${tab.id}`;
    editingTabId.value = null;
  }

  // 持久化
  function saveTabs() {
    localStorage.setItem(
      TABS_KEY,
      JSON.stringify(tabs.map((t) => ({ id: t.id, name: t.name, form: t.form })))
    );
  }
  function loadTabs() {
    try {
      const data = JSON.parse(localStorage.getItem(TABS_KEY) || "[]");
      if (!data.length) return;
      tabs.splice(
        0,
        tabs.length,
        ...data.map((t) => ({
          ...makeTab(t.id),
          id: t.id,
          name: t.name,
          form: {
            ...makeForm(),
            ...t.form,
            bodyType: t.form.bodyType || "none",
            formData: t.form.formData || [],
            urlencoded: t.form.urlencoded || [{ key: "", value: "" }],
            binaryFile: t.form.binaryFile || "",
            rawBody: t.form.rawBody || "",
          },
        }))
      );
      nextId = Math.max(...tabs.map((t) => t.id)) + 1;
      activeTabId.value = tabs[0].id;
    } catch {}
  }

  watch(tabs, saveTabs, { deep: true });

  return {
    tabs,
    activeTabId,
    activeTab,
    makeForm,
    createTab,
    closeTab,
    tabStatus,
    editingTabId,
    startEdit,
    finishEdit,
    loadTabs,
  };
}
