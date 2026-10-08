use serde_json::Value;

/// JSON 校验 + 格式化（缩进 2 空格）
#[tauri::command]
pub fn json_format(input: String) -> Result<String, String> {
    let sanitized = sanitize_json_control_chars(&input);
    let v: Value = serde_json::from_str(&sanitized)
        .map_err(|e| format!("JSON 解析失败：{}", e))?;
    serde_json::to_string_pretty(&v).map_err(|e| format!("JSON 格式化失败：{}", e))
}

/// JSON 压缩（去除空白）
#[tauri::command]
pub fn json_compact(input: String) -> Result<String, String> {
    let sanitized = sanitize_json_control_chars(&input);
    let v: Value = serde_json::from_str(&sanitized)
        .map_err(|e| format!("JSON 解析失败：{}", e))?;
    serde_json::to_string(&v).map_err(|e| format!("JSON 压缩失败：{}", e))
}

/// 把 JSON 字符串字面量内的字面控制字符（U+0000-U+001F）转义为 \uXXXX，
/// 以便 serde_json 能解析（合规 JSON 不允许字符串内含字面控制字符）。
fn sanitize_json_control_chars(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    let mut in_string = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        // 检测转义的双引号 ""
        if c == '"' {
            if in_string && chars.peek() == Some(&'"') {
                out.push('"');
                out.push('"');
                chars.next();
                continue;
            }
            in_string = !in_string;
            out.push(c);
            continue;
        }
        if in_string && (c as u32) < 0x20 {
            out.push_str(&format!("\\u{:04x}", c as u32));
            continue;
        }
        out.push(c);
    }
    out
}

/// 转义：把输入内容整体作为 JSON 字符串值序列化，
/// 即在最外层加引号，并把内部所有 " \ 换行等特殊字符转义。
/// 示例：{"a":1}  ->  "{\"a\":1}"
#[tauri::command]
pub fn json_escape(input: String) -> Result<String, String> {
    // serde_json 序列化 String 即为带引号、内部转义的 JSON 字符串字面量
    serde_json::to_string(&input).map_err(|e| format!("转义失败：{}", e))
}

/// 去除转义：支持两种输入。
/// 1. JSON 字符串字面量（如 "{\"a\":1}"）→ 严格解析回字符串，再尝试格式化为 JSON
/// 2. 含转义序列的任意文本（如 {"a":"b"} 的片段、含 \" \\n \\t 的字符串）→ 手动反转义
/// 示例 1："{\"a\":1}"  ->  {"a":1}
/// 示例 2：{\"a\":1}     ->  {"a":1}
#[tauri::command]
pub fn json_unescape(input: String) -> Result<String, String> {
    let s = input.trim();

    // 1) 先尝试作为 JSON 字符串字面量解析
    if let Ok(unescaped) = serde_json::from_str::<String>(s) {
        return match serde_json::from_str::<Value>(&unescaped) {
            Ok(v) => serde_json::to_string_pretty(&v).map_err(|e| format!("JSON 格式化失败：{}", e)),
            Err(_) => Ok(unescaped),
        };
    }

    // 2) 回退：手动反转义常见转义序列（无需外层引号）
    let unescaped = manual_unescape(s);
    match serde_json::from_str::<Value>(&unescaped) {
        Ok(v) => serde_json::to_string_pretty(&v).map_err(|e| format!("JSON 格式化失败：{}", e)),
        Err(_) => Ok(unescaped),
    }
}

/// 手动反转义：处理 \" \\ \/ \n \r \t \b \f \uXXXX
fn manual_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000C}'),
            Some('u') => {
                let mut hex = String::new();
                for _ in 0..4 {
                    match chars.next() {
                        Some(h) => hex.push(h),
                        None => break,
                    }
                }
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                } else {
                    out.push('\\');
                    out.push('u');
                    out.push_str(&hex);
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => {
                out.push('\\');
            }
        }
    }
    out
}
