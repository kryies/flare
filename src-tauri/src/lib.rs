// reqman —— 轻量 HTTP 请求工具的 Rust 后端
//
// 核心只有一个 Tauri command: send_request。
// 前端通过 invoke("send_request", { ... }) 调用它,由 Rust 在后端真正发起 HTTP 请求。
// 为什么不在前端用 fetch?浏览器 fetch 受 CORS 限制会拦截跨域请求,
// 对一个通用 HTTP 工具是致命的;Rust 的 reqwest 没有这个限制。

use base64::Engine;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 查询参数的编码集:unreserved(A-Z a-z 0-9 - . _ ~)不编码,其余编码(空格→%20)。
/// 这是 RFC 3986 percent-encoding 风格,区别于表单编码(application/x-www-form-urlencoded)的空格→+。
const QUERY_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// 前端传来的单个请求头。serde 会把 JS 对象 { key, value } 反序列化成它。
#[derive(Debug, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
}

/// form-data 的一行:key + 值类型(text/file) + 值(text 文本 或 file 文件路径)
#[derive(Debug, Deserialize)]
pub struct FormPart {
    pub key: String,
    pub part_type: String, // "text" | "file"
    pub text: String,
    pub file: String,
}

/// 返回给前端的响应数据。serde 会把它序列化成 JS 对象。
#[derive(Debug, Serialize)]
pub struct ResponseData {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub content_type: String, // 响应的 Content-Type(决定前端怎么展示:图片预览 / 文本 / 二进制提示)
    pub body_size: u64, // 响应体字节数(二进制时不返回 body,前端显示大小)
    pub elapsed_ms: u64,
    pub final_url: String, // 实际发送的完整 URL(含编码后的查询参数),供前端照抄展示
}

/// 发送一次 HTTP 请求。
///
/// 返回 `Result<ResponseData, String>`:成功给数据,失败把错误转成中文字符串。
/// Tauri 会把 `Err(String)` 变成前端 `invoke().catch(err)` 拿到的拒绝值,
/// 这样前端不用面对 Rust 的错误类型,直接显示字符串即可。
#[tauri::command]
async fn send_request(
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
    // 1. 构建 client。禁用 TLS 证书校验、代理、超时都在这里配。
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

    // 2. 把字符串方法(如 "GET")解析成 reqwest 的 Method 类型。
    let method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|e| format!("无效的 HTTP 方法: {e}"))?;

    // 3. 把查询参数拼进 URL(用 RFC 3986 percent-encoding,空格→%20,
    //    区别于表单编码的 +)。编码在这里做,前端只展示本函数返回的 final_url,
    //    保证「展示的」和「实际发送的」完全是同一个值。
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

    // 4. 叠加请求头,跳过 key 为空的行(前端表格里的空行)。
    for h in &headers {
        if h.key.trim().is_empty() {
            continue;
        }
        req = req.header(h.key.as_str(), h.value.as_str());
    }

    // 5. 根据 body 类型构建请求体(none / raw / form-data / urlencoded / binary)
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
                    form = form.part(p.key.clone(), reqwest::multipart::Part::bytes(data).file_name(name));
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
        _ => {} // none:不带 body
    }

    // 6. 发送!并计时。.await 是 Rust 的异步等待,期间不阻塞线程。
    let start = Instant::now();
    let resp = req.send().await.map_err(|e| format!("请求失败: {e}"))?;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    // 7. 从响应里提取要展示的信息。
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

    // 按 Content-Type 决定怎么取 body:
    //   image/* → base64 编码(前端 <img> 直接预览,不乱码)
    //   文本类(text/、json、xml、html、javascript)→ utf-8 文本
    //   其他二进制(octet-stream、zip 等)→ 不解码,前端显示大小 + 下载
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
        (base64::engine::general_purpose::STANDARD.encode(&bytes), n as u64)
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

/// 把响应体下载到本地文件(响应是文件/二进制时用)。
/// 请求构建和 send_request 一致,但响应直接取 bytes 写文件,不经 String(避免二进制损坏)。
#[tauri::command]
async fn download_response(
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
            builder = builder.proxy(reqwest::Proxy::all(p).map_err(|e| format!("代理配置无效: {e}"))?);
        }
    }
    if timeout_ms > 0 {
        builder = builder.timeout(Duration::from_millis(timeout_ms));
    }
    let client = builder.build().map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

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
    let bytes = resp.bytes().await.map_err(|e| format!("读取响应失败: {e}"))?;
    std::fs::write(&save_path, &bytes).map_err(|e| format!("写文件失败: {e}"))?;
    Ok(bytes.len() as u64)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![send_request, download_response])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证一次正常的 HTTPS GET 能成功返回(顺带验证 rustls-tls 链路通畅),
    /// 并通过 postman-echo 的 header 回显,确认请求头真的被传到了服务端。
    #[tokio::test]
    async fn get_https_returns_200() {
        let resp = send_request(
            "GET".to_string(),
            "https://postman-echo.com/get".to_string(),
            vec![],
            vec![Header {
                key: "X-Reqman-Test".to_string(),
                value: "hello".to_string(),
            }],
            "none".to_string(),
            String::new(),
            vec![],
            vec![],
            String::new(),
            false,
            None,
            0,
        )
        .await
        .expect("请求应成功");
        assert_eq!(resp.status, 200, "期望状态码 200");
        assert!(
            resp.body.to_lowercase().contains("x-reqman-test"),
            "响应体应回显我们发送的自定义 header"
        );
    }

    /// 验证连不上的主机返回 Err(而不是 panic),前端能拿到友好错误字符串。
    #[tokio::test]
    async fn invalid_host_returns_error() {
        let result = send_request(
            "GET".to_string(),
            "https://host-does-not-exist-12345.invalid".to_string(),
            vec![],
            vec![],
            "none".to_string(),
            String::new(),
            vec![],
            vec![],
            String::new(),
            false,
            None,
            0,
        )
        .await;
        assert!(result.is_err(), "无效主机应返回 Err 而非 Ok");
    }

    /// 不联网:验证我们的查询参数编码(QUERY_ENCODE_SET)把空格编成 %20
    /// (RFC 3986 percent-encoding 风格),而不是表单编码的 +。
    #[test]
    fn query_encoding_percent() {
        assert_eq!(utf8_percent_encode("a b", QUERY_ENCODE_SET).to_string(), "a%20b");
        assert_eq!(utf8_percent_encode("中", QUERY_ENCODE_SET).to_string(), "%E4%B8%AD");
        assert_eq!(utf8_percent_encode("a&b=c", QUERY_ENCODE_SET).to_string(), "a%26b%3Dc");
        // unreserved 字符(A-Z a-z 0-9 - . _ ~)不编码
        assert_eq!(
            utf8_percent_encode("a-b_c.d~e", QUERY_ENCODE_SET).to_string(),
            "a-b_c.d~e"
        );
    }
}
