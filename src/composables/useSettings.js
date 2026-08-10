import { reactive, ref, watch, computed } from "vue";

// 全局网络设置 + 主题/布局
export function useSettings() {
  const settings = reactive({
    disableTls: false,
    proxy: "",
    timeoutMs: 0,
  });

  // 主题(dark/light)
  const theme = ref(localStorage.getItem("flare:theme") || "dark");
  watch(theme, (v) => {
    localStorage.setItem("flare:theme", v);
    document.documentElement.className = v === "light" ? "light" : "";
  });

  // 布局(horizontal/vertical)
  const layout = ref(localStorage.getItem("flare:layout") || "horizontal");
  watch(layout, (v) => localStorage.setItem("flare:layout", v));

  // 设置弹窗
  const showSettings = ref(false);

  // 左侧栏宽度(可拖拽)
  const sidebarWidth = ref(Number(localStorage.getItem("flare:sidebar")) || 210);

  // 请求/响应分栏比例
  const splitPercent = ref(Number(localStorage.getItem("flare:split")) || 42);
  const paneLeftStyle = computed(() => ({ flex: `0 0 ${splitPercent.value}%` }));

  // URL 没带协议时自动补 http://
  function ensureScheme(s) {
    if (s && !/^https?:\/\//i.test(s)) return "http://" + s;
    return s;
  }

  return {
    settings,
    theme,
    layout,
    showSettings,
    sidebarWidth,
    splitPercent,
    paneLeftStyle,
    ensureScheme,
  };
}
