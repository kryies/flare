<script setup>
defineProps({
  vars: { type: Array, default: () => [] },
});
const emit = defineEmits(["add", "remove"]);
</script>

<template>
  <details class="env-panel">
    <summary>变量 ({{ vars.length }})</summary>
    <div class="var-hint">
      在 URL / Params / Headers / Body 里用 <code v-pre>{{name}}</code> 引用,发送时替换
    </div>
    <div class="var-list">
      <div v-for="(v, i) in vars" :key="i" class="var-row">
        <input v-model="v.key" placeholder="name" class="var-key" />
        <input v-model="v.value" placeholder="value" class="var-val" />
        <button class="icon-btn" title="删除" @click="emit('remove', i)">✕</button>
      </div>
    </div>
    <button class="add-btn" @click="emit('add')">+ 添加变量</button>
  </details>
</template>

<style scoped>
.env-panel {
  padding: 8px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
}
.env-panel summary {
  cursor: pointer;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-dim);
  font-weight: 600;
  list-style: none;
}
.env-panel summary::-webkit-details-marker {
  display: none;
}
.env-panel summary::before {
  content: "▸";
  margin-right: 6px;
  display: inline-block;
  transition: transform 0.15s ease;
}
.env-panel[open] > summary::before {
  transform: rotate(90deg);
}
.var-hint {
  color: var(--text-dim);
  font-size: 11px;
  margin: 8px 0;
  line-height: 1.5;
}
.var-hint code {
  background: var(--bg-input);
  padding: 1px 5px;
  border-radius: 3px;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  color: var(--accent);
}
.var-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 200px;
  overflow: auto;
}
.var-row {
  display: flex;
  gap: 6px;
}
.var-key {
  flex: 0 0 35%;
}
.var-val {
  flex: 1;
}
.var-key,
.var-val {
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 5px 8px;
  color: var(--text);
  font-size: 12px;
  outline: none;
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
}
.var-key:focus,
.var-val:focus {
  border-color: var(--accent);
}
.icon-btn {
  flex: 0 0 auto;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-dim);
  width: 28px;
  cursor: pointer;
}
.icon-btn:hover {
  color: var(--red);
  border-color: var(--red);
}
.add-btn {
  margin-top: 6px;
  background: transparent;
  color: var(--accent);
  border: 1px dashed var(--border);
  border-radius: 6px;
  padding: 6px;
  width: 100%;
  font-size: 12px;
  cursor: pointer;
}
.add-btn:hover {
  border-color: var(--accent);
}
</style>
