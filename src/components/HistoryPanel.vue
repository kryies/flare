<script setup>
defineProps({
  history: { type: Array, default: () => [] },
});
const emit = defineEmits(["restore", "clear"]);

function fmtTime(ts) {
  const d = new Date(ts);
  const pad = (n) => String(n).padStart(2, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

// 方法对应的颜色
function methodClass(m) {
  if (m === "GET") return "m-get";
  if (m === "POST") return "m-post";
  if (m === "DELETE") return "m-del";
  if (m === "PUT" || m === "PATCH") return "m-put";
  return "m-other";
}
</script>

<template>
  <details class="history-panel">
    <summary>历史 ({{ history.length }})</summary>
    <div v-if="!history.length" class="hist-empty">发送请求后,最近 50 条会记录在这里</div>
    <template v-else>
      <div class="hist-list">
        <div
          v-for="h in history"
          :key="h.id"
          class="hist-item"
          :title="`${h.method} ${h.url}`"
          @click="emit('restore', h)"
        >
          <span class="hist-method" :class="methodClass(h.method)">{{ h.method }}</span>
          <span class="hist-url">{{ h.url }}</span>
          <span class="hist-time">{{ fmtTime(h.time) }}</span>
        </div>
      </div>
      <button class="hist-clear" @click="emit('clear')">清空历史</button>
    </template>
  </details>
</template>

<style scoped>
.history-panel {
  padding: 8px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
}
.history-panel summary {
  cursor: pointer;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-dim);
  font-weight: 600;
  list-style: none;
}
.history-panel summary::-webkit-details-marker {
  display: none;
}
.history-panel summary::before {
  content: "▸";
  margin-right: 6px;
  display: inline-block;
  transition: transform 0.15s ease;
}
.history-panel[open] > summary::before {
  transform: rotate(90deg);
}

.hist-empty {
  color: var(--text-dim);
  font-size: 12px;
  padding: 8px 0;
}

.hist-list {
  display: flex;
  flex-direction: column;
  max-height: 240px;
  overflow: auto;
  margin-top: 8px;
}

.hist-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.hist-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.hist-method {
  flex: 0 0 50px;
  font-weight: 700;
  font-size: 10px;
  text-align: center;
  padding: 2px 0;
  border-radius: 3px;
}
.m-get {
  color: var(--green);
  background: rgba(63, 185, 80, 0.15);
}
.m-post {
  color: var(--accent);
  background: rgba(79, 141, 247, 0.15);
}
.m-put {
  color: var(--orange);
  background: rgba(210, 153, 34, 0.15);
}
.m-del {
  color: var(--red);
  background: rgba(248, 81, 73, 0.15);
}
.m-other {
  color: var(--text-dim);
  background: rgba(154, 160, 172, 0.15);
}

.hist-url {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  color: var(--text);
}

.hist-time {
  flex: 0 0 auto;
  color: var(--text-dim);
  font-size: 11px;
}

.hist-clear {
  margin-top: 6px;
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  font-size: 11px;
  padding: 4px;
}
.hist-clear:hover {
  color: var(--red);
}
</style>
