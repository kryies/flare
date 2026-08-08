<div align="center">

# 🔥 Flare

**轻量、跨平台的 HTTP 调试工具** —— 类 Postman,用 Rust + Tauri 构建。

[![Release](https://img.shields.io/github/v/release/kryies/flare?style=flat-square)](https://github.com/kryies/flare/releases)
[![License](https://img.shields.io/github/license/kryies/flare?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue?style=flat-square)](#-下载)

</div>

> ⚡ Flare = **req**uest **man**ager。发一个 HTTP 请求,就像发一颗信号弹。

## ✨ 功能

- **完整请求构造**:7 种 HTTP 方法、Params(自动 URL 编码 `%20`)、Headers、Body
- **Body 五种类型**:`none` / `form-data` / `x-www-form-urlencoded` / `raw` / `binary`(类 Postman)
- **文件上传 / 下载**:multipart 上传、响应体存盘(二进制不损坏)
- **多标签页**:可重命名、关闭,localStorage 持久化(重开恢复)
- **历史记录**:自动保存最近 50 条,一键恢复
- **环境变量**:`{{base_url}}` 占位符,发送时自动替换
- **网络选项**:HTTP 代理(转发 Burp)、禁用 TLS 证书校验(测自签 HTTPS)、请求超时
- **cURL 互转**:一键导入 / 导出 cURL 命令
- **JSON 语法高亮** + 响应体复制
- **快捷键**:`⌘/Ctrl+Enter` 发送、`+T` 新标签、`+W` 关闭
- **浏览器风格界面**:标头 / 载荷双 Tab,请求预览(展示 = 实际发送)
- **跨平台**:macOS、Windows、Linux

## 📥 下载

去 [Releases](https://github.com/kryies/flare/releases) 下载对应平台的安装包:

| 平台 | 文件 |
|------|------|
| Windows | `Flare_<version>_x64-setup.exe` / `.msi` |
| macOS (Apple Silicon) | `Flare_<version>_aarch64.dmg` |
| Linux | `.AppImage` / `.deb` / `.rpm` |

> Windows 首次运行若提示无法验证开发者,**右键 → 打开** 即可。

## 🛠️ 本地开发

需要安装:[Rust](https://rustup.rs)、[Node.js](https://nodejs.org) 18+、系统 C++ 工具链(macOS 用 Xcode CLT,Windows 用 MSVC Build Tools)。

```bash
git clone https://github.com/kryies/flare.git
cd flare
npm install
npm run tauri dev      # 开发模式(热重载)
npm run tauri build    # 打包成安装包
```

## 🔧 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | [Tauri 2](https://tauri.app) |
| 后端 | Rust + [reqwest](https://docs.rs/reqwest)(HTTP)、[tokio](https://tokio.rs)(异步) |
| 前端 | [Vue 3](https://vuejs.org) + [Vite](https://vitejs.dev) |
| TLS | rustls(纯 Rust,无系统 OpenSSL 依赖) |

> 因为复用系统 WebView(macOS WebKit / Windows WebView2 / Linux WebKitGTK),安装包仅约 **13 MB**,远小于 Electron 同类工具。

## 📦 项目结构

```
flare/
├── src/                  # Vue 前端
│   ├── App.vue           # 根组件(标签页/历史/变量/状态)
│   ├── main.js
│   └── components/       # RequestPanel / ResponsePanel / HistoryPanel / EnvPanel
├── src-tauri/            # Rust 后端
│   ├── src/lib.rs        # 核心:send_request / download_response
│   ├── Cargo.toml
│   └── tauri.conf.json
├── .github/workflows/    # CI:三平台自动发布
└── assets/flare-icon.svg # 图标设计源
```

## 🚀 路线图(可能的方向)

- [ ] 重定向跟随开关(Burp 风格看原始 302)
- [ ] Cookie / Set-Cookie 管理
- [ ] 重放重发(Repeater)
- [ ] TLS 证书详情
- [ ] 二进制响应 hex 查看

## 🤝 贡献

欢迎提 Issue 和 PR。提 PR 前请:
1. Fork 并新建分支
2. `npm run tauri dev` 确认改动正常
3. 提交规范遵循 [Conventional Commits](https://www.conventionalcommits.org/)(如 `feat: ...`、`fix: ...`)

## 📄 许可证

[MIT](LICENSE) © kryies
