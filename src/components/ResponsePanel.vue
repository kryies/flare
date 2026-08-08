<script setup>
import { ref, computed } from "vue";

const props = defineProps({
  response: { type: Object, default: null },
  error: { type: String, default: "" },
  loading: { type: Boolean, default: false },
  sentRequest: { type: Object, default: null },
});
const emit = defineEmits(["download"]);

// 当前激活的 tab:标头 / 载荷(仿浏览器)
const activeTab = ref("headers");

// 状态码对应的颜色类(2xx 绿 / 3xx 蓝 / 4xx 橙 / 5xx 红)
function statusClass(s) {
  if (s >= 200 && s < 300) return "s-2xx";
  if (s >= 300 && s < 400) return "s-3xx";
  if (s >= 400 && s < 500) return "s-4xx";
  if (s >= 500) return "s-5xx";
  return "s-other";
}

// 响应体格式化成缩进 JSON;不是 JSON 就原样
const formattedBody = computed(() => {
  if (!props.response) return "";
  const raw = props.response.body;
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
});

const isJson = computed(() => {
  if (!props.response) return false;
  try {
    JSON.parse(props.response.body);
    return true;
  } catch {
    return false;
  }
});

// —— 请求体:JSON 自动美化 + 「查看解析 / 查看源码」切换(仿浏览器 Payload)——
const bodyViewMode = ref("parsed");
const bodyIsJson = computed(() => {
  if (!props.sentRequest || !props.sentRequest.body) return false;
  try {
    JSON.parse(props.sentRequest.body);
    return true;
  } catch {
    return false;
  }
});
const displayBody = computed(() => {
  const raw = props.sentRequest?.body || "";
  if (bodyViewMode.value === "source" || !bodyIsJson.value) return raw;
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
});

// 常规区里状态码那一格显示的文字
const statusText = computed(() => {
  if (props.error) return "失败";
  if (props.loading) return "请求中…";
  if (props.response) return `${props.response.status} ${props.response.status_text}`;
  return "—";
});

// 请求中(还没收到响应)时,前端按 %20 规则拼一个近似 URL 用于展示。
const fullUrl = computed(() => {
  const base = props.sentRequest?.url || "";
  const params = props.sentRequest?.params || [];
  if (!params.length) return base;
  const qs = params
    .map((p) => `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value)}`)
    .join("&");
  const sep = base.includes("?") ? "&" : "?";
  return base + sep + qs;
});

// 常规区显示的请求 URL:响应回来后用 Rust 返回的真实 final_url(100% 准确);
// 请求中用前端拼的 fullUrl 近似展示(空格同样 %20)。
const displayUrl = computed(() => props.response?.final_url || fullUrl.value);

// —— 响应体复制 ——
const respCopied = ref(false);
async function copyRespBody() {
  if (!props.response) return;
  try {
    await navigator.clipboard.writeText(props.response.body);
    respCopied.value = true;
    setTimeout(() => (respCopied.value = false), 1500);
  } catch (e) {
    alert("复制失败:" + e);
  }
}

// —— JSON 语法高亮(先 HTML 转义再着色,v-html 安全)——
function escapeHtml(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
const JSON_TOKEN_RE =
  /("(?:\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(?:\s*:)?|\b(?:true|false)\b|\bnull\b|-?\d+\.?\d*(?:[eE][+-]?\d+)?)/g;
function highlightJson(raw) {
  let text = raw;
  try {
    text = JSON.stringify(JSON.parse(raw), null, 2); // 合法 JSON:先格式化
  } catch {
    return escapeHtml(raw); // 非 JSON:原样转义
  }
  return escapeHtml(text).replace(JSON_TOKEN_RE, (m) => {
    let cls = "json-number";
    if (m.startsWith('"')) cls = m.trimEnd().endsWith(":") ? "json-key" : "json-string";
    else if (m === "true" || m === "false") cls = "json-bool";
    else if (m === "null") cls = "json-null";
    return `<span class="${cls}">${m}</span>`;
  });
}
const respBodyHtml = computed(() => (props.response ? highlightJson(props.response.body) : ""));
</script>

<template>
  <div class="response-panel">
    <!-- 空状态(从没发送过)-->
    <div v-if="!sentRequest" class="empty">
      <p>填写左侧请求并点「发送」,结果会显示在这里。</p>
    </div>

    <template v-else>
      <!-- tab 栏(仿浏览器)-->
      <div class="tabs">
        <button :class="{ active: activeTab === 'headers' }" @click="activeTab = 'headers'">标头</button>
        <button :class="{ active: activeTab === 'payload' }" @click="activeTab = 'payload'">载荷</button>
      </div>

      <!-- 错误(置顶,切到哪个 tab 都能看到)-->
      <div v-if="error" class="error-box">
        <div class="error-title">⚠ 请求失败</div>
        <pre class="error-msg">{{ error }}</pre>
      </div>

      <!-- ============ 标头 tab ============ -->
      <div v-show="activeTab === 'headers'" class="tab-content">
        <!-- 常规 -->
        <details class="resp-headers" open>
          <summary>常规</summary>
          <table class="hdr-table">
            <tbody>
              <tr>
                <td class="hdr-key">请求 URL</td>
                <td class="hdr-val">{{ displayUrl }}</td>
              </tr>
              <tr>
                <td class="hdr-key">请求方法</td>
                <td class="hdr-val">{{ sentRequest.method }}</td>
              </tr>
              <tr>
                <td class="hdr-key">状态码</td>
                <td class="hdr-val">
                  <span class="status-badge" :class="response ? statusClass(response.status) : 's-other'">
                    {{ statusText }}
                  </span>
                  <span v-if="response" class="elapsed">{{ response.elapsed_ms }} ms</span>
                </td>
              </tr>
            </tbody>
          </table>
        </details>

        <!-- 请求头 -->
        <details v-if="sentRequest.headers.length" class="resp-headers" open>
          <summary>请求头 ({{ sentRequest.headers.length }})</summary>
          <table class="hdr-table">
            <tbody>
              <tr v-for="(h, i) in sentRequest.headers" :key="'qh' + i">
                <td class="hdr-key">{{ h.key }}</td>
                <td class="hdr-val">{{ h.value }}</td>
              </tr>
            </tbody>
          </table>
        </details>

        <!-- 响应头(有响应才显示)-->
        <details v-if="response" class="resp-headers" open>
          <summary>响应头 ({{ response.headers.length }})</summary>
          <table class="hdr-table">
            <tbody>
              <tr v-for="(h, i) in response.headers" :key="'rh' + i">
                <td class="hdr-key">{{ h[0] }}</td>
                <td class="hdr-val">{{ h[1] }}</td>
              </tr>
            </tbody>
          </table>
        </details>

        <!-- 响应体(有响应才显示)-->
        <div v-if="response" class="body-section">
          <div class="section-title">
            响应体
            <span v-if="isJson" class="tag">JSON</span>
            <button class="copy-btn" @click="copyRespBody">
              {{ respCopied ? "已复制 ✓" : "复制" }}
            </button>
            <button class="copy-btn" @click="emit('download')">下载</button>
          </div>
          <pre class="code-pre" v-html="respBodyHtml"></pre>
        </div>
      </div>

      <!-- ============ 载荷 tab ============ -->
      <div v-show="activeTab === 'payload'" class="tab-content">
        <div v-if="!sentRequest.params.length && !sentRequest.body" class="req-none">
          该请求没有查询参数或请求体。
        </div>

        <!-- 查询参数 -->
        <details v-if="sentRequest.params.length" class="resp-headers" open>
          <summary>查询参数 ({{ sentRequest.params.length }})</summary>
          <table class="hdr-table">
            <tbody>
              <tr v-for="(p, i) in sentRequest.params" :key="'p' + i">
                <td class="hdr-key">{{ p.key }}</td>
                <td class="hdr-val">{{ p.value }}</td>
              </tr>
            </tbody>
          </table>
        </details>

        <!-- 请求体(带「查看解析 / 查看源码」切换)-->
        <details v-if="sentRequest.body" class="resp-headers" open>
          <summary>
            请求体
            <span v-if="bodyIsJson" class="view-toggle">
              <a v-if="bodyViewMode !== 'parsed'" @click.stop.prevent="bodyViewMode = 'parsed'">查看解析</a>
              <span v-else class="active">查看解析</span>
              <span class="vsep">|</span>
              <a v-if="bodyViewMode !== 'source'" @click.stop.prevent="bodyViewMode = 'source'">查看源码</a>
              <span v-else class="active">查看源码</span>
            </span>
          </summary>
          <pre class="code-pre">{{ displayBody }}</pre>
        </details>
      </div>
    </template>
  </div>
</template>

<style scoped>
.response-panel {
  padding: 14px;
  height: 100%;
  overflow: auto;
}

.empty {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-dim);
}

/* tab 栏(仿浏览器)*/
.tabs {
  display: flex;
  border-bottom: 1px solid var(--border);
  margin-bottom: 16px;
}
.tabs button {
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-dim);
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  margin-bottom: -1px;
}
.tabs button:hover {
  color: var(--text);
}
.tabs button.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.tab-content {
  display: flex;
  flex-direction: column;
}

/* 错误 */
.error-box {
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid var(--red);
  border-radius: 8px;
  padding: 14px;
  margin-bottom: 16px;
}
.error-title {
  color: var(--red);
  font-weight: 600;
  margin-bottom: 8px;
}
.error-msg {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  font-size: 13px;
}

/* 状态码徽章 */
.status-badge {
  display: inline-block;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}
.s-2xx { background: rgba(63, 185, 80, 0.15); color: var(--green); }
.s-3xx { background: rgba(88, 166, 255, 0.15); color: var(--blue); }
.s-4xx { background: rgba(210, 153, 34, 0.15); color: var(--orange); }
.s-5xx { background: rgba(248, 81, 73, 0.15); color: var(--red); }
.s-other { background: rgba(154, 160, 172, 0.15); color: var(--text-dim); }

.elapsed { color: var(--text-dim); font-size: 12px; margin-left: 8px; }
.tag {
  color: var(--text-dim);
  font-size: 11px;
  background: var(--bg-elevated);
  padding: 1px 6px;
  border-radius: 3px;
  margin-left: 6px;
}

/* 折叠区块(通用,响应头/请求头/查询参数/常规/请求体都用)*/
.resp-headers { margin-bottom: 16px; }
.resp-headers summary {
  cursor: pointer;
  display: flex;
  align-items: center;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-dim);
  font-weight: 600;
  margin-bottom: 8px;
  list-style: none;
}
/* 去掉 Safari/Chrome 默认箭头,用自定义 */
.resp-headers summary::-webkit-details-marker {
  display: none;
}
.resp-headers summary::before {
  content: "▸";
  margin-right: 6px;
  color: var(--text-dim);
  transition: transform 0.15s ease;
}
/* 展开时箭头旋转 90° 指向下方 */
.resp-headers[open] > summary::before {
  transform: rotate(90deg);
}

/* 通用 key/value 表格 */
.hdr-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
.hdr-table td {
  padding: 4px 8px;
  border-bottom: 1px solid var(--border);
  vertical-align: top;
  word-break: break-word;
}
.hdr-key {
  color: var(--text-dim);
  width: 35%;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
}
.hdr-val {
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
}

/* 请求体的「查看解析 / 查看源码」切换 */
.view-toggle {
  margin-left: auto;
  font-weight: 400;
  text-transform: none;
  letter-spacing: normal;
  font-size: 11px;
}
.view-toggle a { color: var(--accent); cursor: pointer; text-decoration: none; }
.view-toggle a:hover { text-decoration: underline; }
.view-toggle .active { color: var(--text-dim); font-weight: 600; }
.view-toggle .vsep { color: var(--text-dim); margin: 0 4px; }

/* 区块标题(响应体)*/
.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-dim);
  margin-bottom: 8px;
  font-weight: 600;
}
.copy-btn {
  margin-left: auto;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-dim);
  font-size: 11px;
  padding: 2px 8px;
  cursor: pointer;
  text-transform: none;
  letter-spacing: normal;
}
.copy-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}
/* JSON 语法高亮颜色 */
.code-pre :deep(.json-key) { color: #79c0ff; }
.code-pre :deep(.json-string) { color: #a5d6ff; }
.code-pre :deep(.json-number) { color: #f0883e; }
.code-pre :deep(.json-bool) { color: #d2a8ff; }
.code-pre :deep(.json-null) { color: var(--text-dim); }

/* 代码块(响应体 / 请求体共用)*/
.code-pre {
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 12px;
  margin: 0;
  overflow: auto;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 50vh;
}
.body-section { margin-bottom: 16px; }

.req-none { color: var(--text-dim); font-size: 13px; padding: 8px 0; }
</style>
