pub mod commands;

use commands::{
    crypto, json_tool, jwt, sql_tool,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // 加密编码
            crypto::md5_encode,
            crypto::md5_encode_upper,
            crypto::base64_encode,
            crypto::base64_decode,
            crypto::url_encode,
            crypto::url_decode,
            // JWT
            jwt::jwt_decode,
            // JSON
            json_tool::json_format,
            json_tool::json_compact,
            json_tool::json_unescape,
            json_tool::json_escape,
            // SQL
            sql_tool::sql_replace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
