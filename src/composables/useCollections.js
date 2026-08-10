import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// 收藏请求(文件持久化)
export function useCollections() {
  const collections = ref([]);

  async function loadCollections() {
    try {
      collections.value = JSON.parse(await invoke("load_collections"));
    } catch {
      collections.value = [];
    }
  }
  async function persistCollections() {
    await invoke("save_collections", { data: JSON.stringify(collections.value) });
  }

  function saveCollection(form) {
    const existingIdx = collections.value.findIndex(
      (c) => c.method === form.method && c.url === form.url
    );
    const data = {
      id: existingIdx >= 0 ? collections.value[existingIdx].id : Date.now(),
      method: form.method,
      url: form.url,
      params: form.params.filter((p) => p.key.trim()).map((p) => ({ ...p })),
      headers: form.headers.filter((h) => h.key.trim()).map((h) => ({ ...h })),
      bodyType: form.bodyType,
      rawBody: form.rawBody,
      formData: (form.formData || []).map((p) => ({ ...p })),
      urlencoded: (form.urlencoded || []).filter((p) => p.key.trim()).map((p) => ({ ...p })),
      binaryFile: form.binaryFile || "",
    };
    if (existingIdx >= 0) {
      collections.value[existingIdx] = data;
    } else {
      collections.value.unshift(data);
    }
    persistCollections();
  }

  function removeCollection(item) {
    collections.value = collections.value.filter((c) => c.id !== item.id);
    persistCollections();
  }

  function restoreCollection(item, form) {
    form.method = item.method;
    form.url = item.url;
    form.params = item.params?.length ? item.params.map((p) => ({ ...p })) : [{ key: "", value: "" }];
    form.headers = item.headers?.length ? item.headers.map((h) => ({ ...h })) : [{ key: "", value: "" }];
    form.bodyType = item.bodyType || "none";
    form.rawBody = item.rawBody || "";
    form.formData = item.formData?.map((p) => ({ ...p })) || [];
    form.urlencoded = item.urlencoded?.length ? item.urlencoded.map((p) => ({ ...p })) : [{ key: "", value: "" }];
    form.binaryFile = item.binaryFile || "";
  }

  return { collections, loadCollections, saveCollection, removeCollection, restoreCollection };
}
