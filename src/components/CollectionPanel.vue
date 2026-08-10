<script setup>
defineProps({
  items: { type: Array, default: () => [] },
});
const emit = defineEmits(["restore", "remove"]);
function methodClass(m) {
  if (m === "GET") return "m-get";
  if (m === "POST") return "m-post";
  if (m === "DELETE") return "m-del";
  if (m === "PUT" || m === "PATCH") return "m-put";
  return "m-other";
}
</script>

<template>
  <details class="collection-panel">
    <summary>收藏 ({{ items.length }})</summary>
    <div v-if="!items.length" class="empty"></div>
    <div v-else class="list">
      <div
        v-for="item in items"
        :key="item.id"
        class="item"
        :title="`${item.method} ${item.url}`"
        @click="emit('restore', item)"
      >
        <span class="method" :class="methodClass(item.method)">{{ item.method }}</span>
        <span class="name">{{ item.url }}</span>
        <button class="del" title="删除" @click.stop="emit('remove', item)">✕</button>
      </div>
    </div>
  </details>
</template>

<style scoped>
.collection-panel {
  padding: 8px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
}
.collection-panel summary {
  cursor: pointer;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-dim);
  font-weight: 600;
  list-style: none;
}
.collection-panel summary::-webkit-details-marker { display: none; }
.collection-panel summary::before {
  content: "▸";
  margin-right: 6px;
  display: inline-block;
  transition: transform 0.15s ease;
}
.collection-panel[open] > summary::before { transform: rotate(90deg); }
.empty { color: var(--text-dim); font-size: 12px; padding: 8px 0; }
.list { display: flex; flex-direction: column; max-height: 240px; overflow: auto; margin-top: 8px; }
.item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.item:hover { background: rgba(255, 255, 255, 0.05); }
.method {
  flex: 0 0 50px;
  font-weight: 700;
  font-size: 10px;
  text-align: center;
  padding: 2px 0;
  border-radius: 3px;
}
.m-get { color: var(--green); background: rgba(63, 185, 80, 0.15); }
.m-post { color: var(--accent); background: rgba(79, 141, 247, 0.15); }
.m-put { color: var(--orange); background: rgba(210, 153, 34, 0.15); }
.m-del { color: var(--red); background: rgba(248, 81, 73, 0.15); }
.m-other { color: var(--text-dim); background: rgba(154, 160, 172, 0.15); }
.name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  color: var(--text);
}
.del {
  flex: 0 0 auto;
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  font-size: 11px;
  padding: 0 2px;
  opacity: 0;
}
.item:hover .del { opacity: 1; }
.del:hover { color: var(--red); }
</style>
