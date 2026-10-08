use base64::{engine::general_purpose::STANDARD, Engine};
use md5::Md5;
use md5::Digest;

/// MD5 编码（小写十六进制）
#[tauri::command]
pub fn md5_encode(input: String) -> Result<String, String> {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// MD5 编码（大写十六进制）
#[tauri::command]
pub fn md5_encode_upper(input: String) -> Result<String, String> {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:X}", result))
}

/// Base64 编码
#[tauri::command]
pub fn base64_encode(input: String) -> Result<String, String> {
    Ok(STANDARD.encode(input.as_bytes()))
}

/// Base64 解码
#[tauri::command]
pub fn base64_decode(input: String) -> Result<String, String> {
    STANDARD
        .decode(input.trim())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|e| format!("Base64 解码失败：{}", e))
}

/// URL 编码
#[tauri::command]
pub fn url_encode(input: String) -> Result<String, String> {
    Ok(urlencoding::encode(&input).into_owned())
}

/// URL 解码
#[tauri::command]
pub fn url_decode(input: String) -> Result<String, String> {
    urlencoding::decode(&input)
        .map(|c| c.into_owned())
        .map_err(|e| format!("URL 解码失败：{}", e))
}
