<script setup>
import { ref, computed, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";

// form 是 App.vue 传入的 reactive 对象,这里直接读写它的属性。
// 因为是同一个对象引用,改 form.headers 等会立刻反映回 App.vue(响应式自动同步)。
const props = defineProps({
  form: { type: Object, required: true },
  loading: { type: Boolean, default: false },
});
const emit = defineEmits(["send", "save-collection"]);

// 支持的 HTTP 方法
const methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

// 只有这些方法才需要 body 输入框
const hasBody = computed(() =>
  ["POST", "PUT", "PATCH", "DELETE", "OPTIONS"].includes(props.form.method)
);

function addHeader() {
  props.form.headers.push({ key: "", value: "" });
}

function removeHeader(i) {
  props.form.headers.splice(i, 1);
  if (props.form.headers.length === 0) addHeader(); // 至少保留一行
}

function addParam() {
  props.form.params.push({ key: "", value: "" });
}
function removeParam(i) {
  props.form.params.splice(i, 1);
  if (props.form.params.length === 0) addParam(); // 至少保留一行
}

// URL 带 ?query 时,自动拆到 Params 表(像 Postman),避免重复
function syncParamsFromUrl() {
  const idx = props.form.url.indexOf("?");
  if (idx === -1) return;
  const base = props.form.url.slice(0, idx);
  const qs = props.form.url.slice(idx + 1);
  const parsed = [];
  try {
    new URLSearchParams(qs).forEach((v, k) => parsed.push({ key: k, value: v }));
  } catch {}
  if (parsed.length) props.form.params = parsed;
  // URL 保留 ?query(用户可见),发送时 send() 会去掉避免重复
}

// Params → URL 反向同步:编辑 Params 时,URL 的 ?query 自动更新(Postman 双向)
watch(
  () => props.form.params,
  () => {
    const base = (props.form.url || "").split("?")[0];
    const valid = props.form.params.filter((p) => p.key.trim());
    if (valid.length) {
      const qs = valid
        .map((p) => `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value)}`)
        .join("&");
      props.form.url = base + "?" + qs;
    } else {
      props.form.url = base;
    }
  },
  { deep: true }
);
function onSend() {
  syncParamsFromUrl();
  if (!props.form.url.trim()) return;
  emit("send");
}

// —— Body 类型(Postman 风格)——
const bodyTypes = [
  { value: "none", label: "none" },
  { value: "form-data", label: "form-data" },
  { value: "urlencoded", label: "x-www-form..." },
  { value: "raw", label: "raw" },
  { value: "binary", label: "binary" },
];

// 切 body 类型时自动同步 Content-Type:
// raw → application/json(可在 Headers 改成 xml/text 等);binary → application/octet-stream;
// form-data / urlencoded 由 Rust(reqwest)自动加(含 boundary),切过去会清掉手动的避免冲突。
function setBodyType(t) {
  props.form.bodyType = t;
  const idx = props.form.headers.findIndex((h) => h.key.toLowerCase() === "content-type");
  if (t === "raw" || t === "binary") {
    const mime = t === "raw" ? "application/json" : "application/octet-stream";
    if (idx >= 0) {
      props.form.headers[idx].value = mime;
    } else if (props.form.headers[0] && props.form.headers[0].key.trim() === "") {
      props.form.headers[0].key = "Content-Type";
      props.form.headers[0].value = mime;
    } else {
      props.form.headers.push({ key: "Content-Type", value: mime });
    }
  } else if ((t === "form-data" || t === "urlencoded") && idx >= 0) {
    props.form.headers.splice(idx, 1);
  }
}
function addFormPart() {
  props.form.formData.push({ key: "", part_type: "text", text: "", file: "" });
}
function removeFormPart(i) {
  props.form.formData.splice(i, 1);
}
async function pickFormFile(i) {
  const sel = await open({ multiple: false });
  if (sel) props.form.formData[i].file = sel;
}
function addUrlencoded() {
  props.form.urlencoded.push({ key: "", value: "" });
}
function removeUrlencoded(i) {
  props.form.urlencoded.splice(i, 1);
  if (props.form.urlencoded.length === 0) addUrlencoded();
}
async function pickBinary() {
  const sel = await open({ multiple: false });
  if (sel) props.form.binaryFile = sel;
}
function fileName(path) {
  return path.split("/").pop() || path;
}

// —— 复制为 cURL 命令 ——
const copied = ref(false);

// 用单引号包裹字符串(shell 安全),内部单引号用 '"'"' 转义
function shellQuote(s) {
  return "'" + String(s).replace(/'/g, "'\"'\"'") + "'";
}

function buildCurl() {
  const f = props.form;
  const parts = [`curl -X ${f.method}`];

  // URL(把 params 拼进去,encodeURIComponent → %20,和实际发送一致)
  let url = f.url;
  const ps = f.params.filter((p) => p.key.trim());
  if (ps.length) {
    const qs = ps
      .map((p) => `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value)}`)
      .join("&");
    url += (url.includes("?") ? "&" : "?") + qs;
  }
  parts.push(shellQuote(url));

  // headers
  for (const h of f.headers.filter((x) => x.key.trim())) {
    parts.push("-H " + shellQuote(`${h.key}: ${h.value}`));
  }

  // body(只有带 body 的方法且有内容才加)
  const b = f.body.trim();
  if (b && hasBody.value) {
    parts.push("-d " + shellQuote(b));
  }

  return parts.join(" \\\n  ");
}

async function copyCurl() {
  try {
    await navigator.clipboard.writeText(buildCurl());
    copied.value = true;
    setTimeout(() => (copied.value = false), 1500);
  } catch (e) {
    alert("复制失败:" + e);
  }
}

// —— 导入 cURL:粘贴 curl 命令,解析填表 ——
const showImport = ref(false);
const importText = ref("");

// 简单 shell tokenizer(处理单/双引号)
function tokenize(str) {
  const tokens = [];
  let i = 0;
  while (i < str.length) {
    while (i < str.length && /\s/.test(str[i])) i++;
    if (i >= str.length) break;
    const ch = str[i];
    if (ch === "'" || ch === '"') {
      const end = str.indexOf(ch, i + 1);
      if (end === -1) {
        tokens.push(str.slice(i + 1));
        break;
      }
      tokens.push(str.slice(i + 1, end));
      i = end + 1;
    } else {
      let j = i;
      while (j < str.length && !/\s/.test(str[j])) j++;
      tokens.push(str.slice(i, j));
      i = j;
    }
  }
  return tokens;
}

function parseCurl(cmd) {
  const tokens = tokenize(cmd.replace(/\\\n/g, " ").trim());
  const form = {
    method: "GET",
    url: "",
    params: [{ key: "", value: "" }],
    headers: [],
    bodyType: "none",
    rawBody: "",
  };
  let hasBody = false;
  for (let i = tokens[0] === "curl" ? 1 : 0; i < tokens.length; i++) {
    const t = tokens[i];
    if (t === "-X" || t === "--request") {
      form.method = tokens[++i] || form.method;
    } else if (t === "-H" || t === "--header") {
      const h = tokens[++i] || "";
      const idx = h.indexOf(":");
      if (idx > -1) form.headers.push({ key: h.slice(0, idx).trim(), value: h.slice(idx + 1).trim() });
    } else if (["-d", "--data", "--data-raw", "--data-binary", "--data-ascii"].includes(t)) {
      form.rawBody += (form.rawBody ? "&" : "") + (tokens[++i] || "");
      hasBody = true;
    } else if (/^https?:\/\//.test(t)) {
      form.url = t;
    }
  }
  if (hasBody) {
    if (form.method === "GET") form.method = "POST";
    form.bodyType = "raw";
  }
  // URL ?query 拆到 params(params watch 会把 ?query 加回 URL)
  const qi = form.url.indexOf("?");
  if (qi >= 0) {
    const base = form.url.slice(0, qi);
    const qs = form.url.slice(qi + 1);
    const parsed = [];
    try { new URLSearchParams(qs).forEach((v, k) => parsed.push({ key: k, value: v })); } catch {}
    if (parsed.length) form.params = parsed;
    form.url = base;
  }
  if (!form.headers.length) form.headers.push({ key: "", value: "" });
  return form;
}

function importCurl() {
  const parsed = parseCurl(importText.value);
  if (!parsed.url) {
    alert("没解析出 URL,检查 curl 命令");
    return;
  }
  props.form.method = parsed.method;
  props.form.headers = parsed.headers;
  props.form.bodyType = parsed.bodyType;
  props.form.rawBody = parsed.rawBody;
  props.form.url = parsed.url;
  props.form.params = parsed.params; // 最后设:触发 watch 把 params 拼回 URL ?query
  showImport.value = false;
  importText.value = "";
}
</script>

<template>
  <div class="request-panel">
    <!-- 方法 + URL + 发送 -->
    <div class="url-bar">
      <select v-model="form.method" class="method-select">
        <option v-for="m in methods" :key="m" :value="m">{{ m }}</option>
      </select>
      <input
        v-model="form.url"
        type="text"
        placeholder="https://api.example.com/users"
        class="url-input"
        @keydown.enter="onSend"
      />
      <button
        class="send-btn"
        :disabled="loading || !form.url.trim()"
        @click="onSend"
      >
        {{ loading ? "请求中…" : "发送" }}
      </button>
    </div>

    <!-- Params(URL 查询参数,自动拼到 ? 后面并编码) -->
    <div class="section">
      <div class="section-title">
        Params <span class="hint">自动拼到 URL,自动编码</span>
      </div>
      <div class="kv-list">
        <div v-for="(p, i) in form.params" :key="i" class="kv-row">
          <input v-model="p.key" placeholder="key" class="kv-input kv-key" />
          <input v-model="p.value" placeholder="value" class="kv-input kv-val" />
          <button class="icon-btn" title="删除该行" @click="removeParam(i)">✕</button>
        </div>
      </div>
      <button class="add-btn" @click="addParam">+ 添加 Param</button>
    </div>

    <!-- Headers -->
    <div class="section">
      <div class="section-title">Headers</div>
      <div class="kv-list">
        <div v-for="(h, i) in form.headers" :key="i" class="kv-row">
          <input v-model="h.key" placeholder="key" class="kv-input kv-key" />
          <input v-model="h.value" placeholder="value" class="kv-input kv-val" />
          <button class="icon-btn" title="删除该行" @click="removeHeader(i)">✕</button>
        </div>
      </div>
      <button class="add-btn" @click="addHeader">+ 添加 Header</button>
    </div>

    <!-- Body(仅 POST/PUT/PATCH/DELETE/OPTIONS 显示;GET/HEAD 没有 Body)-->
    <div v-if="hasBody" class="section">
      <div class="section-title">
        Body
        <div class="body-type-tabs">
          <button
            v-for="t in bodyTypes"
            :key="t.value"
            class="type-btn"
            :class="{ active: form.bodyType === t.value }"
            @click="setBodyType(t.value)"
          >
            {{ t.label }}
          </button>
        </div>
      </div>

      <div v-if="form.bodyType === 'none'" class="body-hint">该请求不带 body</div>

      <textarea
        v-else-if="form.bodyType === 'raw'"
        v-model="form.rawBody"
        placeholder='{ "hello": "world" }(Content-Type 在 Headers 里设)'
        class="body-input"
        spellcheck="false"
      ></textarea>

      <!-- form-data:键值表,值可选文本或文件 -->
      <template v-else-if="form.bodyType === 'form-data'">
        <div class="kv-list">
          <div v-for="(p, i) in form.formData" :key="i" class="form-row">
            <input v-model="p.key" placeholder="key" class="kv-input kv-key" />
            <select v-model="p.part_type" class="part-type-select" title="值类型">
              <option value="text">文本</option>
              <option value="file">文件</option>
            </select>
            <input v-if="p.part_type === 'text'" v-model="p.text" placeholder="value" class="kv-input kv-val" />
            <button v-else class="kv-input file-pick" @click="pickFormFile(i)">
              {{ p.file ? fileName(p.file) : "选择文件" }}
            </button>
            <button class="icon-btn" title="删除该行" @click="removeFormPart(i)">✕</button>
          </div>
        </div>
        <button class="add-btn" @click="addFormPart">+ 添加一行</button>
      </template>

      <!-- x-www-form-urlencoded:纯文本键值表 -->
      <template v-else-if="form.bodyType === 'urlencoded'">
        <div class="kv-list">
          <div v-for="(p, i) in form.urlencoded" :key="i" class="kv-row">
            <input v-model="p.key" placeholder="key" class="kv-input kv-key" />
            <input v-model="p.value" placeholder="value" class="kv-input kv-val" />
            <button class="icon-btn" title="删除该行" @click="removeUrlencoded(i)">✕</button>
          </div>
        </div>
        <button class="add-btn" @click="addUrlencoded">+ 添加一行</button>
      </template>

      <!-- binary:单个文件 -->
      <template v-else-if="form.bodyType === 'binary'">
        <button class="add-btn" @click="pickBinary">
          {{ form.binaryFile ? "📄 " + fileName(form.binaryFile) : "+ 选择文件" }}
        </button>
        <button v-if="form.binaryFile" class="link-btn" @click="form.binaryFile = ''">移除</button>
      </template>
    </div>

    <!-- 收藏 + cURL 导入 / 导出 -->
    <div class="curl-actions">
      <button class="curl-btn" @click="emit('save-collection')" title="收藏当前请求">
        ⭐ 收藏
      </button>
      <button class="curl-btn" @click="copyCurl" title="把当前请求复制为 curl 命令">
        {{ copied ? "已复制 ✓" : "复制为 cURL" }}
      </button>
      <button class="curl-btn" @click="showImport = true" title="粘贴 curl 命令解析填表">
        导入 cURL
      </button>
    </div>

    <!-- 导入弹窗 -->
    <div v-if="showImport" class="modal-overlay" @click.self="showImport = false">
      <div class="modal">
        <div class="modal-title">导入 cURL 命令</div>
        <textarea
          v-model="importText"
          class="modal-input"
          rows="8"
          placeholder="curl -X POST 'https://api.example.com' -H 'Content-Type: application/json' -d '{&quot;key&quot;:&quot;value&quot;}'"
        ></textarea>
        <div class="modal-actions">
          <button class="curl-btn" @click="showImport = false">取消</button>
          <button class="curl-btn primary" @click="importCurl">导入</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.request-panel {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.url-bar {
  display: flex;
  gap: 8px;
}

.method-select {
  flex: 0 0 120px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
}

.url-input {
  flex: 1;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  font-size: 13px;
}

.send-btn {
  flex: 0 0 auto;
  min-width: 104px;
  padding: 0 26px;
  background: var(--accent);
  color: #fff;
  border: none;
  font-size: 15px;
  font-weight: 600;
}

.curl-btn {
  background: transparent;
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 10px;
  width: 100%;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}
.curl-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: rgba(79, 141, 247, 0.08);
}

.curl-actions {
  display: flex;
  gap: 8px;
}
.curl-actions .curl-btn {
  flex: 1;
}
.curl-btn.primary {
  border-color: var(--accent);
  color: var(--accent);
}

/* 导入弹窗 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.modal {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 18px;
  width: 600px;
  max-width: 90vw;
}
.modal-title {
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--text);
}
.modal-input {
  width: 100%;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 10px;
  color: var(--text);
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  font-size: 13px;
  resize: vertical;
  outline: none;
}
.modal-input:focus {
  border-color: var(--accent);
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}
.send-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}
.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.section-title {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-dim);
  margin-bottom: 8px;
  font-weight: 600;
}
.section-title .hint {
  font-weight: 400;
  text-transform: none;
  letter-spacing: normal;
  font-size: 11px;
  color: var(--text-dim);
  margin-left: 6px;
  opacity: 0.8;
}

.kv-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.kv-row {
  display: flex;
  gap: 6px;
}

.kv-key {
  flex: 0 0 40%;
}
.kv-val {
  flex: 1;
}

.add-btn {
  margin-top: 8px;
  padding: 10px;
  background: transparent;
  color: var(--accent);
  border: 1px dashed var(--border);
  border-radius: 6px;
  width: 100%;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}
.add-btn:hover {
  border-color: var(--accent);
  background: rgba(79, 141, 247, 0.08);
}

.body-input {
  width: 100%;
  min-height: 160px;
  resize: vertical;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  font-size: 13px;
}

/* 共用输入框样式 */
.method-select,
.url-input,
.kv-input,
.body-input {
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 10px 12px;
  color: var(--text);
  outline: none;
}
.method-select:focus,
.url-input:focus,
.kv-input:focus,
.body-input:focus {
  border-color: var(--accent);
}

.icon-btn {
  flex: 0 0 auto;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-dim);
  width: 34px;
}
.icon-btn:hover {
  color: var(--red);
  border-color: var(--red);
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 8px;
}
.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 10px;
  font-size: 13px;
}
.file-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
}

/* Body 类型切换 */
.body-type-tabs {
  display: flex;
  gap: 4px;
  margin-left: auto;
}
.type-btn {
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-dim);
  font-size: 11px;
  padding: 3px 8px;
  cursor: pointer;
  text-transform: none;
  letter-spacing: normal;
  font-weight: 500;
}
.type-btn:hover {
  color: var(--text);
  border-color: var(--text-dim);
}
.type-btn.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.body-hint {
  color: var(--text-dim);
  font-size: 13px;
  padding: 6px 0;
}
.form-row {
  display: flex;
  gap: 6px;
  align-items: center;
}
.part-type-select {
  flex: 0 0 68px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px 6px;
  color: var(--text);
  font-size: 12px;
  cursor: pointer;
  outline: none;
}
.file-pick {
  text-align: left;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.link-btn {
  background: transparent;
  border: none;
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  margin-top: 6px;
  text-decoration: underline;
}
.link-btn:hover {
  color: var(--red);
}
</style>
