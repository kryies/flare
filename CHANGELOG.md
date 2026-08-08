# Changelog

本项目的所有重要变更记录于此格式。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/),版本号遵循 [SemVer](https://semver.org/lang/zh-CN/)。

## [1.0.0] - 2026-08-08

首个正式版本。

### Added
- 支持 7 种 HTTP 方法(GET / POST / PUT / PATCH / DELETE / HEAD / OPTIONS)
- 请求构造:Params(自动 `%20` 编码)、Headers、Body
- Body 五种类型:none / form-data / x-www-form-urlencoded / raw / binary
- 文件上传(multipart/form-data、binary)与响应下载(存盘不损坏二进制)
- 多标签页(可重命名 + localStorage 持久化)
- 历史记录(最近 50 条,localStorage)
- 环境变量(`{{name}}` 占位符自动替换)
- 网络选项:HTTP 代理转发、禁用 TLS 证书校验、请求超时
- cURL 一键导入 / 导出
- JSON 语法高亮、响应体复制
- 快捷键(`⌘/Ctrl+Enter` 发送、`+T` 新标签、`+W` 关闭)
- 浏览器风格「标头 / 载荷」双 Tab 视图
- 火焰应用图标
- macOS / Windows / Linux 三平台构建(GitHub Actions 自动发布)
