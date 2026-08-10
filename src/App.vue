<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import RequestPanel from "./components/RequestPanel.vue";
import ResponsePanel from "./components/ResponsePanel.vue";
import HistoryPanel from "./components/HistoryPanel.vue";
import EnvPanel from "./components/EnvPanel.vue";
import CollectionPanel from "./components/CollectionPanel.vue";

// 创建一个空白请求表单
function makeForm() {
  return {
    method: "GET",
    url: "https://postman-echo.com/get",
    params: [{ key: "", value: "" }],
    headers: [{ key: "", value: "" }],
    bodyType: "none", // none | raw | form-data | urlencoded | binary
    rawBody: "",
    formData: [], // [{ key, part_type: "text"|"file", text, file }]
    urlencoded: [{ key: "", value: "" }],
    binaryFile: "",
  };
}

// 创建一个新标签(一个独立的请求 + 它的响应)
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

// ===== 多标签页 =====
const tabs = reactive([makeTab(1)]);
let nextId = 2;
const activeTabId = ref(1);
const activeTab = computed(() => tabs.find((t) => t.id === activeTabId.value));

function createTab() {
  const id = nextId++;
  tabs.push(makeTab(id));
  activeTabId.value = id;
}

function closeTab(id) {
  if (tabs.length === 1) {
    // 最后一个:清空内容,保持至少有一个标签
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

// 标签上的状态小圆点颜色
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

// 标签重命名(双击标签名编辑)
const editingTabId = ref(null);
function startEdit(id) {
  editingTabId.value = id;
}
function finishEdit(tab) {
  if (!tab.name.trim()) tab.name = `请求 ${tab.id}`; // 空名回退默认
  editingTabId.value = null;
}

// ===== 历史记录(localStorage 持久化)=====
const HISTORY_KEY = "reqman:history";
const HISTORY_MAX = 50;
const history = ref(loadHistory());

// ===== 收藏请求(Collection,文件存储)=====
const collections = ref([]);
async function loadCollections() {
  try { collections.value = JSON.parse(await invoke("load_collections")); } catch { collections.value = []; }
}
async function persistCollections() {
  await invoke("save_collections", { data: JSON.stringify(collections.value) });
}
function saveCollection() {
  const tab = activeTab.value;
  if (!tab) return;
  const f = tab.form;
  // 去重:method + url 相同则更新,不重复添加
  const existingIdx = collections.value.findIndex(
    (c) => c.method === f.method && c.url === f.url
  );
  const data = {
    id: existingIdx >= 0 ? collections.value[existingIdx].id : Date.now(),
    method: f.method,
    url: f.url,
    params: f.params.filter((p) => p.key.trim()).map((p) => ({ ...p })),
    headers: f.headers.filter((h) => h.key.trim()).map((h) => ({ ...h })),
    bodyType: f.bodyType,
    rawBody: f.rawBody,
    formData: (f.formData || []).map((p) => ({ ...p })),
    urlencoded: (f.urlencoded || []).filter((p) => p.key.trim()).map((p) => ({ ...p })),
    binaryFile: f.binaryFile || "",
  };
  if (existingIdx >= 0) {
    collections.value[existingIdx] = data; // 更新已有的
  } else {
    collections.value.unshift(data); // 新增
  }
  persistCollections();
}
function removeCollection(item) {
  collections.value = collections.value.filter((c) => c.id !== item.id);
  persistCollections();
}
function restoreCollection(item) {
  const tab = activeTab.value;
  if (!tab) return;
  tab.form.method = item.method;
  tab.form.url = item.url;
  tab.form.params = item.params?.length ? item.params.map((p) => ({ ...p })) : [{ key: "", value: "" }];
  tab.form.headers = item.headers?.length ? item.headers.map((h) => ({ ...h })) : [{ key: "", value: "" }];
  tab.form.bodyType = item.bodyType || "none";
  tab.form.rawBody = item.rawBody || "";
  tab.form.formData = item.formData?.map((p) => ({ ...p })) || [];
  tab.form.urlencoded = item.urlencoded?.length ? item.urlencoded.map((p) => ({ ...p })) : [{ key: "", value: "" }];
  tab.form.binaryFile = item.binaryFile || "";
}

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
// URL 没带协议时自动补 http://(像 Postman,免得每次手敲)
function ensureScheme(s) {
  if (s && !/^https?:\/\//i.test(s)) return "http://" + s;
  return s;
}
function saveToHistory(tab) {
  history.value.unshift({
    id: Date.now(),
    time: Date.now(),
    method: tab.form.method,
    url: applyVars(tab.form.url),
    params: tab.form.params.filter((p) => p.key.trim()).map((p) => ({ key: applyVars(p.key), value: applyVars(p.value) })),
    headers: tab.form.headers.filter((h) => h.key.trim()).map((h) => ({ key: applyVars(h.key), value: applyVars(h.value) })),
    bodyType: tab.form.bodyType,
    rawBody: tab.form.rawBody,
    formData: tab.form.formData.map((p) => ({ ...p })),
    urlencoded: tab.form.urlencoded.filter((p) => p.key.trim()).map((p) => ({ ...p })),
    binaryFile: tab.form.binaryFile,
  });
  if (history.value.length > HISTORY_MAX) history.value.length = HISTORY_MAX;
  persistHistory();
}
function restoreFromHistory(item) {
  const tab = activeTab.value;
  if (!tab) return;
  tab.form.method = item.method;
  tab.form.url = item.url;
  tab.form.params = item.params.length
    ? item.params.map((p) => ({ ...p }))
    : [{ key: "", value: "" }];
  tab.form.headers = item.headers.length
    ? item.headers.map((h) => ({ ...h }))
    : [{ key: "", value: "" }];
  tab.form.bodyType = item.bodyType || "none";
  tab.form.rawBody = item.rawBody || "";
  tab.form.formData = item.formData ? item.formData.map((p) => ({ ...p })) : [];
  tab.form.urlencoded =
    item.urlencoded && item.urlencoded.length
      ? item.urlencoded.map((p) => ({ ...p }))
      : [{ key: "", value: "" }];
  tab.form.binaryFile = item.binaryFile || "";
}
function clearHistory() {
  history.value = [];
  persistHistory();
}
function removeSingleHistory(item) {
  history.value = history.value.filter((h) => h.id !== item.id);
  persistHistory();
}

// ===== 全局网络设置(禁用证书校验、代理)=====
const settings = reactive({
  disableTls: false,
  proxy: "",
  timeoutMs: 0, // 0 = 不限时
});

// ===== 布局(水平左右 / 垂直上下),localStorage 持久化 =====
const layout = ref(localStorage.getItem("flare:layout") || "horizontal");
watch(layout, (v) => localStorage.setItem("flare:layout", v));

// 主题(dark/light),切换时改 <html> 的 class
const theme = ref(localStorage.getItem("flare:theme") || "dark");
watch(theme, (v) => {
  localStorage.setItem("flare:theme", v);
  document.documentElement.className = v === "light" ? "light" : "";
});
const showSettings = ref(false);
const showVars = ref(false);

// 左侧栏宽度(可拖拽调整,localStorage 持久化)
const sidebarWidth = ref(Number(localStorage.getItem("flare:sidebar")) || 210);
function startResize(e) {
  e.preventDefault();
  const onMove = (ev) => {
    sidebarWidth.value = Math.max(50, Math.min(500, ev.clientX));
  };
  const onUp = () => {
    localStorage.setItem("flare:sidebar", String(sidebarWidth.value));
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
  };
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
}

// 请求/响应分栏宽度(可拖拽)
const splitPercent = ref(Number(localStorage.getItem("flare:split")) || 42);
const paneLeftStyle = computed(() => {
  if (layout.value === "vertical") return { flex: `0 0 ${splitPercent.value}%` };
  return { flex: `0 0 ${splitPercent.value}%` };
});
function startSplitResize(e) {
  e.preventDefault();
  const wrapper = e.target.parentElement;
  const rect = wrapper.getBoundingClientRect();
  const isVertical = layout.value === "vertical";
  const onMove = (ev) => {
    let pct;
    if (isVertical) {
      pct = ((ev.clientY - rect.top) / rect.height) * 100;
    } else {
      pct = ((ev.clientX - rect.left) / rect.width) * 100;
    }
    splitPercent.value = Math.max(10, Math.min(90, pct));
  };
  const onUp = () => {
    localStorage.setItem("flare:split", String(splitPercent.value));
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
  };
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
}

// ===== 环境变量(URL/Headers/Body 里的 {{name}} 发送时替换成值)=====
const envVars = reactive([{ key: "base_url", value: "https://postman-echo.com" }]);
function applyVars(str) {
  if (!str) return str;
  return str.replace(/\{\{(\w+)\}\}/g, (m, k) => {
    const v = envVars.find((x) => x.key === k);
    return v ? v.value : m; // 找不到就保留原样 {{name}}
  });
}
function addVar() {
  envVars.push({ key: "", value: "" });
}
function removeVar(i) {
  envVars.splice(i, 1);
}
async function loadEnvVars() {
  try {
    const data = JSON.parse(await invoke("load_env_vars"));
    if (Array.isArray(data) && data.length) envVars.splice(0, envVars.length, ...data);
  } catch {}
}
async function persistEnvVars() {
  await invoke("save_env_vars", { data: JSON.stringify(envVars) });
}
watch(envVars, () => persistEnvVars(), { deep: true });

// ===== 发送请求 =====
async function send() {
  const tab = activeTab.value;
  if (!tab || tab.loading) return;
  tab.loading = true;
  tab.error = "";
  tab.response = null;

  try {
    // 应用环境变量({{name}} → 值),发送的是替换后的内容
    let url = ensureScheme(applyVars(tab.form.url));
    // URL 含 ?query 时去掉(用 Params 拼,避免重复)
    const qi = url.indexOf("?");
    if (qi >= 0) url = url.slice(0, qi);
    const params = tab.form.params
      .filter((p) => p.key.trim() !== "")
      .map((p) => ({ key: applyVars(p.key), value: applyVars(p.value) }));
    const headers = tab.form.headers
      .filter((h) => h.key.trim() !== "")
      .map((h) => ({ key: applyVars(h.key), value: applyVars(h.value) }));
    // 记录本次发送的请求(右侧"请求预览"显示实际发出的内容)
    tab.sentRequest = { method: tab.form.method, url, params, headers, bodyType: tab.form.bodyType };

    // 记到历史(无论成败,方便重发)
    saveToHistory(tab);

    // invoke 调用 Rust 的 send_request command(按 body 类型带上相应数据)
    tab.response = await invoke("send_request", {
      method: tab.form.method,
      url,
      params,
      headers,
      bodyType: tab.form.bodyType,
      rawBody: tab.form.bodyType === "raw" ? applyVars(tab.form.rawBody) : "",
      formData: tab.form.formData
        .filter((p) => p.key.trim())
        .map((p) => ({ key: applyVars(p.key), part_type: p.part_type, text: applyVars(p.text), file: p.file })),
      urlencoded: tab.form.urlencoded
        .filter((p) => p.key.trim())
        .map((p) => ({ key: applyVars(p.key), value: applyVars(p.value) })),
      binaryFile: tab.form.binaryFile,
      disableTls: settings.disableTls,
      proxy: ensureScheme(settings.proxy.trim()) || null,
      timeoutMs: Number(settings.timeoutMs) || 0,
    });
  } catch (e) {
    // Rust 返回的 Err(String) 会到这里
    tab.error = typeof e === "string" ? e : JSON.stringify(e);
  } finally {
    tab.loading = false;
  }
}

// —— 下载响应到文件 ——
// 从 Content-Disposition 头解析 filename(服务器指定的文件名)
function parseDisposition(headers) {
  const cd = headers.find((h) => h[0].toLowerCase() === "content-disposition")?.[1] || "";
  const m = cd.match(/filename\*?=(?:UTF-8'')?["']?([^"';]+)/i);
  return m ? decodeURIComponent(m[1]) : "";
}
// Content-Type → 文件后缀
function mimeToExt(ct) {
  const base = ct.split(";")[0].trim().toLowerCase();
  const map = {
    "image/png": "png", "image/jpeg": "jpg", "image/gif": "gif",
    "image/svg+xml": "svg", "image/webp": "webp", "image/x-icon": "ico",
    "application/pdf": "pdf", "application/zip": "zip", "application/gzip": "gz",
    "application/json": "json", "application/xml": "xml",
    "text/html": "html", "text/plain": "txt", "text/csv": "csv",
  };
  if (map[base]) return map[base];
  if (base.startsWith("image/")) return base.slice(6);
  if (base.startsWith("text/")) return base.slice(5);
  return ""; // 未知类型(如 application/octet-stream)→ 无后缀
}
// 推断下载文件名,优先级同 Postman / 浏览器(RFC 6266)
function suggestFileName(url, headers = []) {
  // 1. Content-Disposition(服务器指定,最准)
  const disp = parseDisposition(headers);
  if (disp) return disp;
  // 2. URL 路径最后段(带后缀)
  try {
    const seg = new URL(applyVars(url)).pathname.split("/").filter(Boolean).pop();
    if (seg && /\.[a-z0-9]{1,8}$/i.test(seg)) return seg;
  } catch {}
  // 3. Content-Type 推断后缀
  const ct = headers.find((h) => h[0].toLowerCase() === "content-type")?.[1] || "";
  const ext = mimeToExt(ct);
  if (ext) return "response." + ext;
  // 4. 默认兜底
  return "response.bin";
}
async function downloadResponse() {
  const tab = activeTab.value;
  if (!tab) return;
  const path = await save({
    defaultPath: suggestFileName(tab.form.url, tab.response?.headers || []),
    filters: [{ name: "全部文件", extensions: ["*"] }],
  });
  if (!path) return; // 用户取消
  try {
    const n = await invoke("download_response", {
      method: tab.form.method,
      url: ensureScheme(applyVars(tab.form.url)),
      params: tab.form.params.filter((p) => p.key.trim()).map((p) => ({ key: applyVars(p.key), value: applyVars(p.value) })),
      headers: tab.form.headers.filter((h) => h.key.trim()).map((h) => ({ key: applyVars(h.key), value: applyVars(h.value) })),
      body: tab.form.body ? applyVars(tab.form.body) : null,
      disableTls: settings.disableTls,
      proxy: ensureScheme(settings.proxy.trim()) || null,
      timeoutMs: Number(settings.timeoutMs) || 0,
      savePath: path,
    });
    alert(`已保存(${n} 字节):\n${path}`);
  } catch (e) {
    alert("下载失败:" + (typeof e === "string" ? e : JSON.stringify(e)));
  }
}

// —— 全局快捷键(⌘/Ctrl + Enter 发送、+T 新标签、+W 关闭)——
function onKeydown(e) {
  const mod = e.metaKey || e.ctrlKey;
  if (!mod) return;
  const k = e.key.toLowerCase();
  if (k === "enter") {
    e.preventDefault();
    send();
  } else if (k === "t") {
    e.preventDefault();
    createTab();
  } else if (k === "w") {
    e.preventDefault();
    closeTab(activeTabId.value);
  }
}

// —— 多标签持久化(localStorage,重开恢复)——
const TABS_KEY = "reqman:tabs";
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
        response: null,
        error: "",
        loading: false,
        sentRequest: null,
      }))
    );
    nextId = Math.max(...tabs.map((t) => t.id)) + 1;
    activeTabId.value = tabs[0].id;
  } catch {
    /* 解析失败就用默认空标签 */
  }
}
watch(tabs, saveTabs, { deep: true });

let unlistenSettings = null;
onMounted(async () => {
  loadTabs();
  loadCollections();
  loadEnvVars();
  document.documentElement.className = theme.value === "light" ? "light" : "";
  window.addEventListener("keydown", onKeydown);
  unlistenSettings = await listen("open-settings", () => { showSettings.value = true; });
});
onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  if (unlistenSettings) unlistenSettings();
});
</script>

<template>
  <div class="app">
    <header class="app-header">
      <div class="header-left">
        <img src="/flare.svg" class="brand-icon" alt="Flare" />
        <span class="brand">Flare</span>
      </div>
      <div class="header-settings">
        <button
          class="layout-btn"
          @click="layout = layout === 'horizontal' ? 'vertical' : 'horizontal'"
          :title="layout === 'horizontal' ? '切换为上下布局' : '切换为左右布局'"
        >
          <svg v-if="layout === 'horizontal'" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="1" y="2" width="6" height="12" rx="1"/><rect x="9" y="2" width="6" height="12" rx="1"/></svg>
          <svg v-else viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="1" width="12" height="6" rx="1"/><rect x="2" y="9" width="12" height="6" rx="1"/></svg>
        </button>
      </div>
    </header>

    <!-- 设置弹窗(分区:通用 / 网络 / 变量) -->
    <div v-if="showSettings" class="modal-overlay" @click.self="showSettings = false">
      <div class="settings-modal">
        <div class="settings-title">⚙ 设置</div>
        <div class="settings-body">

        <!-- 通用 -->
        <div class="settings-section">通用</div>
        <div class="settings-row">
          <span class="settings-label">外观主题</span>
          <div class="theme-switch">
            <button :class="{ active: theme === 'dark' }" @click="theme = 'dark'">🌙 深色</button>
            <button :class="{ active: theme === 'light' }" @click="theme = 'light'">☀️ 浅色</button>
          </div>
        </div>

        <!-- 网络 -->
        <div class="settings-section">网络</div>
        <div class="settings-row">
          <span class="settings-label">禁用 TLS 证书校验</span>
          <input type="checkbox" v-model="settings.disableTls" />
        </div>
        <div class="settings-row">
          <span class="settings-label">HTTP 代理</span>
          <input class="settings-input" v-model="settings.proxy" placeholder="http://127.0.0.1:8080" />
        </div>
        <div class="settings-row">
          <span class="settings-label">超时(ms,0=不限)</span>
          <input class="settings-input settings-input-sm" type="number" v-model.number="settings.timeoutMs" min="0" step="500" />
        </div>

        <!-- 环境变量 -->
        <div class="settings-section">环境变量</div>
        <div class="settings-hint">在 URL / Params / Headers / Body 里用 <code v-pre>{{name}}</code> 引用,发送时自动替换</div>
        <div class="var-modal-list">
          <div v-for="(v, i) in envVars" :key="i" class="var-modal-row">
            <input v-model="v.key" placeholder="name" class="settings-input var-modal-key" />
            <input v-model="v.value" placeholder="value" class="settings-input var-modal-val" />
            <button class="var-modal-del" @click="removeVar(i)">✕</button>
          </div>
        </div>
        <button class="settings-add" @click="addVar">+ 添加变量</button>

        <button class="settings-close" @click="showSettings = false">完成</button>
        </div>
      </div>
    </div>

    <div class="app-body">
      <!-- 左侧栏:管理(历史/收藏),独立于标签栏 -->
      <aside class="sidebar" :style="{ flexBasis: sidebarWidth + 'px' }">
        <HistoryPanel :history="history" @restore="restoreFromHistory" @clear="clearHistory" @remove="removeSingleHistory" />
        <CollectionPanel :items="collections" @restore="restoreCollection" @remove="removeCollection" />
        <div class="sidebar-resize" @mousedown="startResize"></div>
      </aside>

      <!-- 右侧:标签栏 + 请求/响应(标签栏只在这里,不覆盖左侧) -->
      <div class="main-wrapper">
        <div class="tab-bar">
          <div
            v-for="tab in tabs"
            :key="tab.id"
            class="tab-item"
            :class="{ active: tab.id === activeTabId }"
            @click="activeTabId = tab.id"
          >
            <span class="tab-status" :class="tabStatus(tab)"></span>
            <input
              v-if="editingTabId === tab.id"
              :ref="(el) => el?.focus()"
              v-model="tab.name"
              class="tab-name-input"
              @click.stop
              @blur="finishEdit(tab)"
              @keydown.enter="finishEdit(tab)"
              @keydown.esc="finishEdit(tab)"
            />
            <span v-else class="tab-name" title="双击重命名" @dblclick.stop="startEdit(tab.id)">{{ tab.name }}</span>
            <button class="tab-close" title="关闭标签" @click.stop="closeTab(tab.id)">✕</button>
          </div>
          <button class="tab-new" title="新建标签" @click="createTab">+</button>
        </div>

        <main class="app-main" :class="{ vertical: layout === 'vertical' }" v-if="activeTab">
          <section class="pane pane-left" :class="{ vertical: layout === 'vertical' }" :style="paneLeftStyle">
            <RequestPanel :form="activeTab.form" :loading="activeTab.loading" @send="send" @save-collection="saveCollection" />
          </section>
          <div class="split-resize" :class="{ vertical: layout === 'vertical' }" @mousedown="startSplitResize"></div>
          <section class="pane pane-right" :class="{ vertical: layout === 'vertical' }">
            <ResponsePanel
              :response="activeTab.response"
              :error="activeTab.error"
              :loading="activeTab.loading"
              :sent-request="activeTab.sentRequest"
              @download="downloadResponse"
            />
          </section>
        </main>
      </div>
    </div>
  </div>
</template>

<style>
/* 全局主题变量(深色) */
:root {
  --bg: #1e1f25;
  --bg-elevated: #282a33;
  --bg-input: #1a1b20;
  --border: #3a3d4a;
  --text: #e4e6eb;
  --text-dim: #9aa0ac;
  --accent: #4f8df7;
  --accent-hover: #6aa0ff;
  --green: #3fb950;
  --orange: #d29922;
  --red: #f85149;
  --blue: #58a6ff;

  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
    "Helvetica Neue", Arial, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  color: var(--text);
  background-color: var(--bg);
}

/* 浅色主题 */
:root.light {
  --bg: #f5f6f8;
  --bg-elevated: #ffffff;
  --bg-input: #ebedf0;
  --border: #d8dbe0;
  --text: #1a1b20;
  --text-dim: #6a7178;
  --accent: #2563eb;
  --accent-hover: #1d4ed8;
  --green: #16a34a;
  --orange: #ca8a04;
  --red: #dc2626;
  --blue: #2563eb;
  color: var(--text);
  background-color: var(--bg);
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  height: 100%;
  margin: 0;
}

.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 16px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border);
  -webkit-user-select: none;
  user-select: none;
}
.header-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}
.header-settings {
  display: flex;
  align-items: center;
  gap: 12px;
}
.setting {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--text-dim);
  cursor: pointer;
  -webkit-user-select: none;
  user-select: none;
}
.setting input[type="checkbox"] {
  cursor: pointer;
  accent-color: var(--accent);
}
.proxy-input {
  width: 220px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 5px 10px;
  color: var(--text);
  font-size: 12px;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  outline: none;
}
.proxy-input:focus {
  border-color: var(--accent);
}
.timeout-input {
  width: 64px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 3px 6px;
  color: var(--text);
  font-size: 12px;
  outline: none;
}
.timeout-input:focus {
  border-color: var(--accent);
}

.brand-icon {
  width: 22px;
  height: 22px;
  border-radius: 5px;
}
.brand {
  font-weight: 700;
  font-size: 16px;
}

.subtitle {
  color: var(--text-dim);
  font-size: 13px;
}

/* 主体:左侧栏 + 右侧请求/响应 */
.app-body {
  flex: 1;
  display: flex;
  min-height: 0;
}

/* 右侧:标签栏 + 请求/响应 */
.main-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

/* 左侧栏(历史/收藏) */
.sidebar {
  flex: 0 0 210px;
  position: relative;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  overflow-x: hidden;
  white-space: nowrap;
  background: var(--bg-elevated);
  border-right: 1px solid var(--border);
}
.sidebar-resize {
  position: absolute;
  top: 0;
  right: -2px;
  width: 4px;
  height: 100%;
  cursor: col-resize;
  z-index: 10;
}
.sidebar-resize:hover {
  background: var(--accent);
}

/* 标签栏(顶部水平,保持以前) */
.tab-bar {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 6px 14px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border);
  -webkit-user-select: none;
  user-select: none;
  overflow-x: auto;
  overflow-y: hidden;
  flex-shrink: 0;
}
.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 10px;
  cursor: pointer;
  border-radius: 6px;
  border: 1px solid transparent;
  color: var(--text-dim);
  font-size: 13px;
  max-width: 200px;
}
.tab-item:hover {
  color: var(--text);
  background: rgba(255, 255, 255, 0.03);
}
.tab-item.active {
  color: var(--text);
  background: var(--bg-input);
  border-color: var(--accent);
}
.tab-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}
.tab-name-input {
  background: var(--bg-input);
  border: 1px solid var(--accent);
  border-radius: 3px;
  color: var(--text);
  font-size: 13px;
  padding: 1px 4px;
  outline: none;
  width: 100px;
}
.tab-status {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex: 0 0 auto;
  background: transparent;
}
.tab-status.st-loading {
  background: var(--orange);
  animation: tab-pulse 1s infinite;
}
.tab-status.st-ok {
  background: var(--green);
}
.tab-status.st-error {
  background: var(--red);
}
.tab-status.st-other {
  background: var(--text-dim);
}
@keyframes tab-pulse {
  50% {
    opacity: 0.3;
  }
}
.tab-close {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 0 2px;
  font-size: 12px;
  border-radius: 3px;
}
.tab-close:hover {
  color: var(--red);
  background: rgba(248, 81, 73, 0.12);
}
.tab-new {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  font-size: 18px;
  padding: 4px 10px;
  border-radius: 4px;
}
.tab-new:hover {
  color: var(--accent);
  background: rgba(255, 255, 255, 0.05);
}

/* 右侧主体(请求 + 响应) */
.app-main {
  flex: 1;
  display: flex;
  min-height: 0;
}

.pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: auto;
}

.pane-left {
  flex: 0 0 42%;
  border-right: none;
}

.pane-right {
  flex: 1;
}

/* 请求/响应拖拽条(隐形,hover 才显示) */
.split-resize {
  flex: 0 0 1px;
  cursor: col-resize;
  background: var(--border);
  position: relative;
}
.split-resize::after {
  content: "";
  position: absolute;
  inset: 0 -3px;
}
.split-resize:hover {
  background: var(--accent);
}
.split-resize.vertical {
  cursor: row-resize;
}
.split-resize.vertical::after {
  inset: -3px 0;
}

/* 上下(垂直)布局 */
.app-main.vertical {
  flex-direction: column;
}
.pane-left.vertical {
  border-right: none;
}
.pane-right.vertical {
  flex: 1;
}
.layout-btn {
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-dim);
  font-size: 12px;
  padding: 4px 10px;
  cursor: pointer;
  white-space: nowrap;
}
.layout-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

/* 设置弹窗 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}
.settings-modal {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  width: 460px;
  max-width: 90vw;
  overflow: hidden;
}
.settings-title {
  font-weight: 700;
  font-size: 15px;
  padding: 14px 20px;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
  -webkit-user-select: none;
  user-select: none;
}
.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}
.settings-label {
  color: var(--text-dim);
  font-size: 14px;
}
.settings-body {
  padding: 20px;
}
.settings-section {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
  margin: 18px 0 10px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border);
}
.settings-section:first-of-type {
  margin-top: 0;
}
.settings-input-sm {
  width: 80px;
}
.settings-input {
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 10px;
  color: var(--text);
  font-size: 13px;
  outline: none;
  width: 200px;
}
.settings-input:focus {
  border-color: var(--accent);
}
.theme-switch {
  display: flex;
  gap: 6px;
}
.theme-switch button {
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-dim);
  padding: 6px 14px;
  font-size: 13px;
  cursor: pointer;
}
.theme-switch button.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.settings-close {
  margin-top: 8px;
  width: 100%;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}
.settings-hint {
  color: var(--text-dim);
  font-size: 12px;
  margin-bottom: 14px;
  line-height: 1.5;
}
.var-modal-list {
  max-height: 300px;
  overflow-y: auto;
  margin-bottom: 8px;
}
.var-modal-row {
  display: flex;
  gap: 6px;
  margin-bottom: 6px;
}
.var-modal-key {
  flex: 0 0 120px;
  width: 120px;
}
.var-modal-val {
  flex: 1;
  width: auto;
}
.var-modal-del {
  flex: 0 0 auto;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-dim);
  cursor: pointer;
  width: 32px;
}
.var-modal-del:hover {
  color: var(--red);
  border-color: var(--red);
}
.settings-add {
  width: 100%;
  background: transparent;
  border: 1px dashed var(--border);
  border-radius: 6px;
  color: var(--accent);
  padding: 8px;
  font-size: 13px;
  cursor: pointer;
  margin-bottom: 8px;
}
.settings-add:hover {
  border-color: var(--accent);
}
</style>
