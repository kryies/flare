// Flare —— 轻量 HTTP 调试工具的 Rust 后端入口
// 代码拆分:types.rs(类型) + commands.rs(命令) + lib.rs(入口/菜单/测试)

mod commands;
mod types;

use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::send_request,
            commands::download_response,
            commands::save_collections,
            commands::load_collections,
            commands::save_env_vars,
            commands::load_env_vars,
        ])
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem, Submenu};

            let settings_item =
                MenuItem::with_id(app, "settings", "设置...", true, Some("CmdOrCtrl+,"))?;
            let quit_item =
                MenuItem::with_id(app, "quit", "退出 Flare", true, Some("CmdOrCtrl+Q"))?;
            let app_menu =
                Submenu::with_items(app, "Flare", true, &[&settings_item, &quit_item])?;
            let menu = Menu::with_items(app, &[&app_menu])?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "settings" => {
                    let _ = app.emit("open-settings", ());
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use commands::send_request;
    use types::*;

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

    #[test]
    fn query_encoding_percent() {
        use percent_encoding::utf8_percent_encode;
        assert_eq!(
            utf8_percent_encode("a b", types::QUERY_ENCODE_SET).to_string(),
            "a%20b"
        );
        assert_eq!(
            utf8_percent_encode("中", types::QUERY_ENCODE_SET).to_string(),
            "%E4%B8%AD"
        );
        assert_eq!(
            utf8_percent_encode("a&b=c", types::QUERY_ENCODE_SET).to_string(),
            "a%26b%3Dc"
        );
        assert_eq!(
            utf8_percent_encode("a-b_c.d~e", types::QUERY_ENCODE_SET).to_string(),
            "a-b_c.d~e"
        );
    }
}
