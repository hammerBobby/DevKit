use serde_json::Value;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

/// JWT 解析结果
#[derive(serde::Serialize)]
pub struct JwtResult {
    pub header: Value,
    pub payload: Value,
    pub signature: String,
    pub expired: Option<bool>,
    pub exp: Option<i64>,
    pub not_before: Option<i64>,
    pub issued_at: Option<i64>,
    pub valid: bool,
    pub error: Option<String>,
}

/// 解码 base64url 段并解析为 JSON
fn decode_segment(segment: &str) -> Result<Value, String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(segment.trim())
        .map_err(|e| format!("Base64URL 解码失败：{}", e))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("JSON 解析失败：{}", e))
}

/// 解析 JWT token
#[tauri::command]
pub fn jwt_decode(token: String) -> Result<JwtResult, String> {
    let token = token.trim();
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("不是有效的 JWT 格式，需为 header.payload.signature 三段结构".to_string());
    }

    let header = match decode_segment(parts[0]) {
        Ok(v) => v,
        Err(e) => {
            return Ok(JwtResult {
                header: Value::Null,
                payload: Value::Null,
                signature: String::new(),
                expired: None,
                exp: None,
                not_before: None,
                issued_at: None,
                valid: false,
                error: Some(format!("Header 解析失败：{}", e)),
            })
        }
    };

    let payload = match decode_segment(parts[1]) {
        Ok(v) => v,
        Err(e) => {
            return Ok(JwtResult {
                header,
                payload: Value::Null,
                signature: parts[2].to_string(),
                expired: None,
                exp: None,
                not_before: None,
                issued_at: None,
                valid: false,
                error: Some(format!("Payload 解析失败：{}", e)),
            })
        }
    };

    let now = chrono_epoch_now();

    // 提取时间戳声明（JWT 使用秒）
    let exp = payload.get("exp").and_then(extract_i64);
    let nbf = payload.get("nbf").and_then(extract_i64);
    let iat = payload.get("iat").and_then(extract_i64);

    let expired = exp.map(|e| now >= e);
    let not_yet_valid = nbf.map(|n| now < n);
    let valid = expired != Some(true) && not_yet_valid != Some(true);

    Ok(JwtResult {
        header,
        payload,
        signature: parts[2].to_string(),
        expired,
        exp,
        not_before: nbf,
        issued_at: iat,
        valid,
        error: None,
    })
}

fn extract_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    }
}

/// 获取当前 Unix 时间戳（秒）。为避免额外依赖，使用系统时间近似。
fn chrono_epoch_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
