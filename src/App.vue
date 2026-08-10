<script setup>
import { onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import RequestPanel from "./components/RequestPanel.vue";
import ResponsePanel from "./components/ResponsePanel.vue";
import HistoryPanel from "./components/HistoryPanel.vue";
import CollectionPanel from "./components/CollectionPanel.vue";
import { useTabs } from "./composables/useTabs";
import { useHistory } from "./composables/useHistory";
import { useCollections } from "./composables/useCollections";
import { useEnvVars } from "./composables/useEnvVars";
import { useSettings } from "./composables/useSettings";
import { useRequest } from "./composables/useRequest";

// ===== Composables =====
const {
  tabs, activeTabId, activeTab, createTab, closeTab, tabStatus,
  editingTabId, startEdit, finishEdit, loadTabs,
} = useTabs();
const { history, saveToHistory, removeSingleHistory, clearHistory } = useHistory();
const {
  collections, loadCollections, saveCollection: saveColl,
  removeCollection, restoreCollection,
} = useCollections();
const { envVars, applyVars, addVar, removeVar, loadEnvVars } = useEnvVars();
const {
  settings, theme, layout, showSettings,
  sidebarWidth, splitPercent, paneLeftStyle, ensureScheme,
} = useSettings();
const { send: sendReq, downloadResponse: dlResp } = useRequest(
  settings, applyVars, ensureScheme, { saveToHistory }
);

// ===== Bridge functions =====
async function send() { await sendReq(activeTab.value); }
function saveCollection() { if (activeTab.value) saveColl(activeTab.value.form); }
async function downloadResponse() { await dlResp(activeTab.value); }
function handleRestoreCollection(item) {
  if (activeTab.value) restoreCollection(item, activeTab.value.form);
}
function restoreFromHistory(item) {
  const f = activeTab.value?.form;
  if (!f) return;
  f.method = item.method;
  f.url = item.url;
  f.params = item.params?.length ? item.params.map((p) => ({ ...p })) : [{ key: "", value: "" }];
  f.headers = item.headers?.length ? item.headers.map((h) => ({ ...h })) : [{ key: "", value: "" }];
  f.bodyType = item.bodyType || "none";
  f.rawBody = item.rawBody || "";
  f.formData = item.formData?.map((p) => ({ ...p })) || [];
  f.urlencoded = item.urlencoded?.length ? item.urlencoded.map((p) => ({ ...p })) : [{ key: "", value: "" }];
  f.binaryFile = item.binaryFile || "";
}

// ===== Keyboard shortcuts =====
function onKeydown(e) {
  const mod = e.metaKey || e.ctrlKey;
  if (!mod) return;
  const k = e.key.toLowerCase();
  if (k === "enter") { e.preventDefault(); send(); }
  else if (k === "t") { e.preventDefault(); createTab(); }
  else if (k === "w") { e.preventDefault(); closeTab(activeTabId.value); }
}

// ===== Resize handlers =====
function startResize(e) {
  e.preventDefault();
  const onMove = (ev) => { sidebarWidth.value = Math.max(50, Math.min(500, ev.clientX)); };
  const onUp = () => {
    localStorage.setItem("flare:sidebar", String(sidebarWidth.value));
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
  };
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
}
function startSplitResize(e) {
  e.preventDefault();
  const rect = e.target.parentElement.getBoundingClientRect();
  const isVertical = layout.value === "vertical";
  const onMove = (ev) => {
    let pct = isVertical
      ? ((ev.clientY - rect.top) / rect.height) * 100
      : ((ev.clientX - rect.left) / rect.width) * 100;
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

// ===== Lifecycle =====
let unlistenFn = null;
onMounted(async () => {
  loadTabs();
  loadCollections();
  loadEnvVars();
  document.documentElement.className = theme.value === "light" ? "light" : "";
  window.addEventListener("keydown", onKeydown);
  unlistenFn = await listen("open-settings", () => { showSettings.value = true; });
});
onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  if (unlistenFn) unlistenFn();
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

    <!-- 设置弹窗 -->
    <div v-if="showSettings" class="modal-overlay" @click.self="showSettings = false">
      <div class="settings-modal">
        <div class="settings-title">⚙ 设置</div>
        <div class="settings-body">
          <div class="settings-section">通用</div>
          <div class="settings-row">
            <span class="settings-label">外观主题</span>
            <div class="theme-switch">
              <button :class="{ active: theme === 'dark' }" @click="theme = 'dark'">🌙 深色</button>
              <button :class="{ active: theme === 'light' }" @click="theme = 'light'">☀️ 浅色</button>
            </div>
          </div>
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
      <aside class="sidebar" :style="{ flexBasis: sidebarWidth + 'px' }">
        <HistoryPanel :history="history" @restore="restoreFromHistory" @clear="clearHistory" @remove="removeSingleHistory" />
        <CollectionPanel :items="collections" @restore="handleRestoreCollection" @remove="removeCollection" />
        <div class="sidebar-resize" @mousedown="startResize"></div>
      </aside>

      <div class="main-wrapper">
        <div class="tab-bar">
          <div v-for="tab in tabs" :key="tab.id" class="tab-item" :class="{ active: tab.id === activeTabId }" @click="activeTabId = tab.id">
            <span class="tab-status" :class="tabStatus(tab)"></span>
            <input v-if="editingTabId === tab.id" :ref="(el) => el?.focus()" v-model="tab.name" class="tab-name-input" @click.stop @blur="finishEdit(tab)" @keydown.enter="finishEdit(tab)" @keydown.esc="finishEdit(tab)" />
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
            <ResponsePanel :response="activeTab.response" :error="activeTab.error" :loading="activeTab.loading" :sent-request="activeTab.sentRequest" @download="downloadResponse" />
          </section>
        </main>
      </div>
    </div>
  </div>
</template>

<style>
:root {
  --bg: #1e1f25; --bg-elevated: #282a33; --bg-input: #1a1b20;
  --border: #3a3d4a; --text: #e4e6eb; --text-dim: #9aa0ac;
  --accent: #4f8df7; --accent-hover: #6aa0ff;
  --green: #3fb950; --orange: #d29922; --red: #f85149; --blue: #58a6ff;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  font-size: 14px; line-height: 1.5;
  color: var(--text); background-color: var(--bg);
}
:root.light {
  --bg: #f5f6f8; --bg-elevated: #ffffff; --bg-input: #ebedf0;
  --border: #d8dbe0; --text: #1a1b20; --text-dim: #6a7178;
  --accent: #2563eb; --green: #16a34a; --orange: #ca8a04; --red: #dc2626; --blue: #2563eb;
  color: var(--text); background-color: var(--bg);
}
* { box-sizing: border-box; }
html, body, #app { height: 100%; margin: 0; }
.app { display: flex; flex-direction: column; height: 100vh; }
.app-header { display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; background: var(--bg-elevated); border-bottom: 1px solid var(--border); }
.header-left { display: flex; align-items: center; gap: 12px; }
.header-settings { display: flex; align-items: center; gap: 12px; }
.brand-icon { width: 22px; height: 22px; border-radius: 5px; }
.brand { font-weight: 700; font-size: 16px; }
.app-body { flex: 1; display: flex; min-height: 0; }
.main-wrapper { flex: 1; display: flex; flex-direction: column; min-width: 0; }
.sidebar { flex: 0 0 210px; position: relative; display: flex; flex-direction: column; overflow-y: auto; overflow-x: hidden; white-space: nowrap; background: var(--bg-elevated); border-right: 1px solid var(--border); }
.sidebar-resize { position: absolute; top: 0; right: -2px; width: 4px; height: 100%; cursor: col-resize; z-index: 10; }
.sidebar-resize:hover { background: var(--accent); }
.tab-bar { display: flex; align-items: center; gap: 2px; padding: 6px 14px; background: var(--bg-elevated); border-bottom: 1px solid var(--border); overflow-x: auto; overflow-y: hidden; flex-shrink: 0; }
.tab-item { display: flex; align-items: center; gap: 6px; padding: 7px 10px; cursor: pointer; border-radius: 6px; border: 1px solid transparent; color: var(--text-dim); font-size: 13px; max-width: 200px; }
.tab-item:hover { color: var(--text); background: rgba(255,255,255,0.03); }
.tab-item.active { color: var(--text); background: var(--bg-input); border-color: var(--accent); }
.tab-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; cursor: pointer; }
.tab-name-input { background: var(--bg-input); border: 1px solid var(--accent); border-radius: 3px; color: var(--text); font-size: 13px; padding: 1px 4px; outline: none; width: 100px; }
.tab-status { width: 7px; height: 7px; border-radius: 50%; flex: 0 0 auto; background: transparent; }
.tab-status.st-loading { background: var(--orange); animation: tab-pulse 1s infinite; }
.tab-status.st-ok { background: var(--green); }
.tab-status.st-error { background: var(--red); }
.tab-status.st-other { background: var(--text-dim); }
@keyframes tab-pulse { 50% { opacity: 0.3; } }
.tab-close { background: transparent; border: none; color: var(--text-dim); cursor: pointer; padding: 0 2px; font-size: 12px; border-radius: 3px; }
.tab-close:hover { color: var(--red); background: rgba(248,81,73,0.12); }
.tab-new { background: transparent; border: none; color: var(--text-dim); cursor: pointer; font-size: 18px; padding: 4px 10px; border-radius: 4px; }
.tab-new:hover { color: var(--accent); }
.app-main { flex: 1; display: flex; min-height: 0; }
.pane { display: flex; flex-direction: column; min-height: 0; overflow: auto; }
.pane-left { flex: 0 0 42%; border-right: none; }
.pane-right { flex: 1; }
.split-resize { flex: 0 0 1px; cursor: col-resize; background: var(--border); position: relative; }
.split-resize::after { content: ""; position: absolute; inset: 0 -3px; }
.split-resize:hover { background: var(--accent); }
.split-resize.vertical { cursor: row-resize; }
.split-resize.vertical::after { inset: -3px 0; }
.app-main.vertical { flex-direction: column; }
.layout-btn { background: transparent; border: 1px solid var(--border); border-radius: 6px; color: var(--text-dim); padding: 4px 10px; cursor: pointer; }
.layout-btn:hover { border-color: var(--accent); color: var(--accent); }
.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 200; }
.settings-modal { background: var(--bg-elevated); border: 1px solid var(--border); border-radius: 10px; width: 460px; max-width: 90vw; overflow: hidden; }
.settings-title { font-weight: 700; font-size: 15px; padding: 14px 20px; background: var(--bg); border-bottom: 1px solid var(--border); }
.settings-body { padding: 20px; }
.settings-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; }
.settings-label { color: var(--text-dim); font-size: 14px; }
.settings-section { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.08em; color: var(--accent); margin: 18px 0 10px; padding-bottom: 6px; border-bottom: 1px solid var(--border); }
.settings-section:first-of-type { margin-top: 0; }
.settings-input { background: var(--bg-input); border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px; color: var(--text); font-size: 13px; outline: none; width: 200px; }
.settings-input:focus { border-color: var(--accent); }
.settings-input-sm { width: 80px; }
.theme-switch { display: flex; gap: 6px; }
.theme-switch button { background: var(--bg-input); border: 1px solid var(--border); border-radius: 6px; color: var(--text-dim); padding: 6px 14px; font-size: 13px; cursor: pointer; }
.theme-switch button.active { background: var(--accent); border-color: var(--accent); color: #fff; }
.settings-close { margin-top: 8px; width: 100%; background: var(--accent); color: #fff; border: none; border-radius: 6px; padding: 10px; font-size: 14px; font-weight: 600; cursor: pointer; }
.settings-hint { color: var(--text-dim); font-size: 12px; margin-bottom: 14px; }
.var-modal-list { max-height: 300px; overflow-y: auto; margin-bottom: 8px; }
.var-modal-row { display: flex; gap: 6px; margin-bottom: 6px; }
.var-modal-key { flex: 0 0 120px; width: 120px; }
.var-modal-val { flex: 1; width: auto; }
.var-modal-del { flex: 0 0 auto; background: transparent; border: 1px solid var(--border); border-radius: 6px; color: var(--text-dim); cursor: pointer; width: 32px; }
.var-modal-del:hover { color: var(--red); border-color: var(--red); }
.settings-add { width: 100%; background: transparent; border: 1px dashed var(--border); border-radius: 6px; color: var(--accent); padding: 8px; font-size: 13px; cursor: pointer; margin-bottom: 8px; }
.settings-add:hover { border-color: var(--accent); }
</style>
