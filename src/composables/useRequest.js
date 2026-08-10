import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

// 发送请求 + 下载响应
export function useRequest(settings, envApplyVars, ensureScheme, history) {
  async function send(tab) {
    if (!tab || tab.loading) return;
    tab.loading = true;
    tab.error = "";
    tab.response = null;

    try {
      const applyVars = envApplyVars;
      let url = ensureScheme(applyVars(tab.form.url));
      const qi = url.indexOf("?");
      if (qi >= 0) url = url.slice(0, qi);
      const params = tab.form.params
        .filter((p) => p.key.trim() !== "")
        .map((p) => ({ key: applyVars(p.key), value: applyVars(p.value) }));
      const headers = tab.form.headers
        .filter((h) => h.key.trim() !== "")
        .map((h) => ({ key: applyVars(h.key), value: applyVars(h.value) }));

      tab.sentRequest = { method: tab.form.method, url, params, headers, bodyType: tab.form.bodyType };

      history.saveToHistory({
        method: tab.form.method,
        url,
        params,
        headers,
        bodyType: tab.form.bodyType,
        rawBody: tab.form.rawBody,
        formData: tab.form.formData.map((p) => ({ ...p })),
        urlencoded: tab.form.urlencoded.filter((p) => p.key.trim()).map((p) => ({ ...p })),
        binaryFile: tab.form.binaryFile,
      });

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
      tab.error = typeof e === "string" ? e : JSON.stringify(e);
    } finally {
      tab.loading = false;
    }
  }

  function suggestFileName(url, headers = []) {
    const cd = headers.find((h) => h[0].toLowerCase() === "content-disposition")?.[1] || "";
    const m = cd.match(/filename\*?=(?:UTF-8'')?["']?([^"';]+)/i);
    if (m) return decodeURIComponent(m[1]);
    try {
      const seg = new URL(url).pathname.split("/").filter(Boolean).pop();
      if (seg && /\.[a-z0-9]{1,8}$/i.test(seg)) return seg;
    } catch {}
    const ct = headers.find((h) => h[0].toLowerCase() === "content-type")?.[1] || "";
    const base = ct.split(";")[0].trim().toLowerCase();
    const map = {
      "image/png": "png", "image/jpeg": "jpg", "image/gif": "gif",
      "image/svg+xml": "svg", "image/webp": "webp", "image/x-icon": "ico",
      "application/pdf": "pdf", "application/zip": "zip", "application/gzip": "gz",
      "application/json": "json", "application/xml": "xml",
      "text/html": "html", "text/plain": "txt", "text/csv": "csv",
    };
    const ext = map[base] || (base.startsWith("image/") ? base.slice(6) : base.startsWith("text/") ? base.slice(5) : "");
    if (ext) return "response." + ext;
    return "response.bin";
  }

  async function downloadResponse(tab) {
    if (!tab) return;
    const path = await save({
      defaultPath: suggestFileName(tab.form.url, tab.response?.headers || []),
      filters: [{ name: "全部文件", extensions: ["*"] }],
    });
    if (!path) return;
    try {
      const applyVars = envApplyVars;
      const n = await invoke("download_response", {
        method: tab.form.method,
        url: ensureScheme(applyVars(tab.form.url)).split("?")[0],
        params: tab.form.params.filter((p) => p.key.trim()).map((p) => ({ key: applyVars(p.key), value: applyVars(p.value) })),
        headers: tab.form.headers.filter((h) => h.key.trim()).map((h) => ({ key: applyVars(h.key), value: applyVars(h.value) })),
        body: tab.form.rawBody ? applyVars(tab.form.rawBody) : null,
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

  return { send, downloadResponse };
}
