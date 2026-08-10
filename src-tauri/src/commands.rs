// 所有 Tauri command(send_request / download_response / 文件读写)

use crate::types::*;
use base64::Engine;
use percent_encoding::utf8_percent_encode;
use std::time::{Duration, Instant};
use tauri::Manager;

/// 发送一次 HTTP 请求。
#[tauri::command]
pub async fn send_request(
    method: String,
    url: String,
    params: Vec<Header>,
    headers: Vec<Header>,
    body_type: String,
    raw_body: String,
    form_data: Vec<FormPart>,
    urlencoded: Vec<Header>,
    binary_file: String,
    disable_tls: bool,
    proxy: Option<String>,
    timeout_ms: u64,
) -> Result<ResponseData, String> {
    let mut builder = reqwest::Client::builder().danger_accept_invalid_certs(disable_tls);
    if let Some(p) = proxy.as_deref() {
        if !p.trim().is_empty() {
            builder = builder
                .proxy(reqwest::Proxy::all(p).map_err(|e| format!("代理配置无效: {e}"))?);
        }
    }
    if timeout_ms > 0 {
        builder = builder.timeout(Duration::from_millis(timeout_ms));
    }
    let client = builder
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|e| format!("无效的 HTTP 方法: {e}"))?;

    let mut full_url = url.clone();
    let query_parts: Vec<String> = params
        .iter()
        .filter(|p| !p.key.trim().is_empty())
        .map(|p| {
            let k = utf8_percent_encode(&p.key, QUERY_ENCODE_SET);
            let v = utf8_percent_encode(&p.value, QUERY_ENCODE_SET);
            format!("{k}={v}")
        })
        .collect();
    if !query_parts.is_empty() {
        full_url.push_str(if full_url.contains('?') { "&" } else { "?" });
        full_url.push_str(&query_parts.join("&"));
    }

    let mut req = client.request(method, &full_url);

    for h in &headers {
        if h.key.trim().is_empty() {
            continue;
        }
        req = req.header(h.key.as_str(), h.value.as_str());
    }

    match body_type.as_str() {
        "raw" => {
            if !raw_body.trim().is_empty() {
                req = req.body(raw_body);
            }
        }
        "form-data" => {
            let mut form = reqwest::multipart::Form::new();
            for p in &form_data {
                if p.key.trim().is_empty() {
                    continue;
                }
                if p.part_type == "file" && !p.file.is_empty() {
                    let data = std::fs::read(&p.file)
                        .map_err(|e| format!("读文件失败 {}: {e}", p.file))?;
                    let name = std::path::Path::new(&p.file)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file")
                        .to_string();
                    form = form.part(
                        p.key.clone(),
                        reqwest::multipart::Part::bytes(data).file_name(name),
                    );
                } else {
                    form = form.text(p.key.clone(), p.text.clone());
                }
            }
            req = req.multipart(form);
        }
        "urlencoded" => {
            let pairs: Vec<(String, String)> = urlencoded
                .iter()
                .filter(|p| !p.key.trim().is_empty())
                .map(|p| (p.key.clone(), p.value.clone()))
                .collect();
            if !pairs.is_empty() {
                req = req.form(&pairs);
            }
        }
        "binary" => {
            if !binary_file.is_empty() {
                let data = std::fs::read(&binary_file)
                    .map_err(|e| format!("读文件失败 {}: {e}", binary_file))?;
                req = req.body(data);
            }
        }
        _ => {}
    }

    let start = Instant::now();
    let resp = req.send().await.map_err(|e| format!("请求失败: {e}"))?;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let status = resp.status().as_u16();
    let status_text = resp
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    let resp_headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let ct = content_type.split(';').next().unwrap_or("").trim().to_lowercase();
    let (body, body_size) = if ct.starts_with("image/") {
        let bytes = resp.bytes().await.map_err(|e| format!("读取响应体失败: {e}"))?;
        let n = bytes.len();
        (
            base64::engine::general_purpose::STANDARD.encode(&bytes),
            n as u64,
        )
    } else if ct.starts_with("text/")
        || ct.contains("json")
        || ct.contains("xml")
        || ct.contains("html")
        || ct.contains("javascript")
    {
        let text = resp.text().await.map_err(|e| format!("读取响应体失败: {e}"))?;
        let n = text.len();
        (text, n as u64)
    } else {
        let bytes = resp.bytes().await.map_err(|e| format!("读取响应体失败: {e}"))?;
        (String::new(), bytes.len() as u64)
    };

    Ok(ResponseData {
        status,
        status_text,
        headers: resp_headers,
        body,
        content_type,
        body_size,
        elapsed_ms,
        final_url: full_url,
    })
}

/// 把响应体下载到本地文件。
#[tauri::command]
pub async fn download_response(
    method: String,
    url: String,
    params: Vec<Header>,
    headers: Vec<Header>,
    body: Option<String>,
    disable_tls: bool,
    proxy: Option<String>,
    timeout_ms: u64,
    save_path: String,
) -> Result<u64, String> {
    let mut builder = reqwest::Client::builder().danger_accept_invalid_certs(disable_tls);
    if let Some(p) = proxy.as_deref() {
        if !p.trim().is_empty() {
            builder =
                builder.proxy(reqwest::Proxy::all(p).map_err(|e| format!("代理配置无效: {e}"))?);
        }
    }
    if timeout_ms > 0 {
        builder = builder.timeout(Duration::from_millis(timeout_ms));
    }
    let client = builder
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|e| format!("无效的 HTTP 方法: {e}"))?;

    let mut full_url = url.clone();
    let query_parts: Vec<String> = params
        .iter()
        .filter(|p| !p.key.trim().is_empty())
        .map(|p| {
            let k = utf8_percent_encode(&p.key, QUERY_ENCODE_SET);
            let v = utf8_percent_encode(&p.value, QUERY_ENCODE_SET);
            format!("{k}={v}")
        })
        .collect();
    if !query_parts.is_empty() {
        full_url.push_str(if full_url.contains('?') { "&" } else { "?" });
        full_url.push_str(&query_parts.join("&"));
    }

    let mut req = client.request(method, &full_url);
    for h in &headers {
        if h.key.trim().is_empty() {
            continue;
        }
        req = req.header(h.key.as_str(), h.value.as_str());
    }
    if let Some(b) = body.as_ref() {
        if !b.trim().is_empty() {
            req = req.body(b.clone());
        }
    }

    let resp = req.send().await.map_err(|e| format!("请求失败: {e}"))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("读取响应失败: {e}"))?;
    std::fs::write(&save_path, &bytes).map_err(|e| format!("写文件失败: {e}"))?;
    Ok(bytes.len() as u64)
}

// ===== 文件存储(收藏 / 环境变量)=====

/// 保存收藏列表到 app_data_dir/collections.json
#[tauri::command]
pub fn save_collections(app: tauri::AppHandle, data: String) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("collections.json"), data).map_err(|e| e.to_string())?;
    Ok(())
}

/// 读取收藏列表
#[tauri::command]
pub fn load_collections(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = dir.join("collections.json");
    if path.exists() {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    } else {
        Ok("[]".to_string())
    }
}

/// 保存环境变量到 env_vars.json
#[tauri::command]
pub fn save_env_vars(app: tauri::AppHandle, data: String) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("env_vars.json"), data).map_err(|e| e.to_string())?;
    Ok(())
}

/// 读取环境变量
#[tauri::command]
pub fn load_env_vars(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = dir.join("env_vars.json");
    if path.exists() {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    } else {
        Ok("[]".to_string())
    }
}
