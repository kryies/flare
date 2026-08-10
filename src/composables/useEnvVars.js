import { reactive, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

// 环境变量(全局共享,文件持久化)
export function useEnvVars() {
  const envVars = reactive([{ key: "base_url", value: "https://postman-echo.com" }]);

  // {{name}} → 值,发送时替换
  function applyVars(str) {
    if (!str) return str;
    return str.replace(/\{\{(\w+)\}\}/g, (m, k) => {
      const v = envVars.find((x) => x.key === k);
      return v ? v.value : m;
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

  return { envVars, applyVars, addVar, removeVar, loadEnvVars };
}
