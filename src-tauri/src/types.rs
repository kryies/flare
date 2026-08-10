// 类型定义 + 编码集

use percent_encoding::AsciiSet;
use serde::{Deserialize, Serialize};

/// 查询参数的编码集:unreserved(A-Z a-z 0-9 - . _ ~)不编码,其余编码(空格→%20)。
/// 这是 RFC 3986 percent-encoding 风格,区别于表单编码(application/x-www-form-urlencoded)的空格→+。
pub const QUERY_ENCODE_SET: &AsciiSet = &{
    let set = percent_encoding::CONTROLS
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
    set
};

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
    pub content_type: String,
    pub body_size: u64,
    pub elapsed_ms: u64,
    pub final_url: String,
}
