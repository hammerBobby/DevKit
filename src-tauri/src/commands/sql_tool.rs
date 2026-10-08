/// 解析后的参数项
struct Param {
    value: String,
    typ: String,
}

/// 单个字段长度校验结果
#[derive(serde::Serialize, Clone)]
pub struct FieldCheck {
    pub field: String,
    pub db_type: String,
    pub max_len: Option<usize>,
    pub actual_len: Option<usize>,
    pub status: String, // "ok" | "over" | "skip" | "unknown-type"
}

/// SQL 替换结果
#[derive(serde::Serialize)]
pub struct SqlResult {
    pub sql: String,
    pub update_sql: Option<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub table_name: Option<String>,
    pub field_checks: Vec<FieldCheck>,
}

/// SQL 参数替换主函数
/// 输入格式：`SQL模板, 执行参数：1(String), 张三(String), ...`
/// 分隔符支持中文逗号和 ", 执行参数：" 等
/// `table_structure` 可选：CREATE TABLE DDL，用于 INSERT/UPDATE 字段 VARCHAR 长度校验
#[tauri::command]
pub fn sql_replace(
    sql_template: String,
    table_structure: Option<String>,
) -> Result<SqlResult, String> {
    let template = sql_template.trim().to_string();
    if template.is_empty() {
        return Err("请输入 SQL 模板".to_string());
    }

    // 从 SQL 模板中自动分离出内嵌的 CREATE TABLE（支持 SQL; CREATE TABLE 拼在一起）
    let (sql_template_part, embedded_ddl) = split_embedded_create_table(&template);

    // 解析表结构：显式传入的优先，否则用 SQL 模板内嵌的 CREATE TABLE
    let ddl_source = match &table_structure {
        Some(ddl) if !ddl.trim().is_empty() => ddl.as_str(),
        _ => embedded_ddl.as_str(),
    };
    let field_type_map = parse_table_structure(ddl_source);

    // 拆分 SQL 与参数部分
    let (sql_part, param_part) = split_sql_and_params(&sql_template_part);

    let params = parse_params(&param_part)?;

    // 参数数量校验
    let placeholder_count = count_placeholders(&sql_part);
    if params.len() != placeholder_count {
        return Err(format!(
            "参数数量不匹配：SQL 中有 {} 个 ?，但提供了 {} 个参数",
            placeholder_count,
            params.len()
        ));
    }

    // 识别 SQL 类型
    let sql_trim = sql_part.trim_start();
    let lower = sql_trim.to_lowercase();
    let sql_type = if lower.starts_with("insert") {
        "INSERT"
    } else if lower.starts_with("select") {
        "SELECT"
    } else if lower.starts_with("update") {
        "UPDATE"
    } else if lower.starts_with("delete") {
        "DELETE"
    } else {
        "OTHER"
    };

    // 先做基础占位符替换
    let replaced = replace_placeholders(&sql_part, &params)
        .map_err(|e| e.to_string())?;

    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut update_sql: Option<String> = None;
    let mut table_name: Option<String> = None;
    let mut field_checks: Vec<FieldCheck> = Vec::new();

    // INSERT 特殊处理：生成 UPDATE + 字段长度校验
    if sql_type == "INSERT" {
        match handle_insert(&sql_part, &params, &field_type_map, &mut field_checks) {
            Ok(insert_out) => {
                table_name = insert_out.table_name;
                update_sql = insert_out.update_sql;
                warnings = insert_out.warnings;
                errors = insert_out.errors;
            }
            Err(e) => {
                // INSERT 解析失败，退回基础替换结果
                warnings.push(format!("INSERT 增强处理失败（已回退为普通替换）：{}", e));
            }
        }
    } else if sql_type == "UPDATE" {
        // UPDATE：对 SET 字段做长度校验
        if !field_type_map.is_empty() {
            validate_update_fields(&sql_part, &params, &field_type_map, &mut warnings, &mut errors, &mut field_checks);
        }
    }

    Ok(SqlResult {
        sql: replaced,
        update_sql,
        warnings,
        errors,
        table_name,
        field_checks,
    })
}

/// 拆分 SQL 和参数部分
fn split_sql_and_params(template: &str) -> (String, String) {
    // 支持分隔符：", 执行参数："、"，执行参数："、"; 执行参数："、";执行参数："
    for sep in ["，执行参数：", ", 执行参数：", ";执行参数：", ",执行参数：", "；执行参数："] {
        if let Some(idx) = template.find(sep) {
            let sql = template[..idx].trim().to_string();
            let params = template[idx + sep.len()..].trim().to_string();
            return (sql, params);
        }
    }
    // 无分隔符，尝试通过 ", " 后跟参数推断（当 SQL 含 ? 时）
    if template.contains('?') {
        if let Some(idx) = template.find('，') {
            return (template[..idx].trim().to_string(), template[idx + 1..].trim().to_string());
        }
    }
    (template.to_string(), String::new())
}

/// 统计 ? 占位符数量（忽略字符串内的 ?）
fn count_placeholders(sql: &str) -> usize {
    let mut count = 0;
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double {
            // 处理 SQL 转义引号 'xx''yy'
            if i + 1 < chars.len() && chars[i + 1] == '\'' {
                i += 2;
                continue;
            }
            in_single = !in_single;
        } else if c == '"' && !in_single {
            in_double = !in_double;
        } else if c == '?' && !in_single && !in_double {
            count += 1;
        }
        i += 1;
    }
    count
}

/// 替换占位符
fn replace_placeholders(sql: &str, params: &[Param]) -> Result<String, String> {
    let mut result = String::new();
    let mut idx = 0;
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double {
            if i + 1 < chars.len() && chars[i + 1] == '\'' {
                result.push(c);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            in_single = !in_single;
            result.push(c);
        } else if c == '"' && !in_single {
            in_double = !in_double;
            result.push(c);
        } else if c == '?' && !in_single && !in_double {
            if idx >= params.len() {
                return Err(format!("参数不足：第 {} 个 ? 没有对应参数", idx + 1));
            }
            result.push_str(&format_param(&params[idx]));
            idx += 1;
        } else {
            result.push(c);
        }
        i += 1;
    }
    Ok(result)
}

/// 根据类型格式化参数值
fn format_param(p: &Param) -> String {
    let v = p.value.trim();
    if v.to_lowercase() == "null" {
        return "NULL".to_string();
    }
    let t = p.typ.to_lowercase();
    // 字符串类型：加单引号，转义单引号
    let is_string_type = t.contains("string")
        || t.contains("date")
        || t.contains("timestamp")
        || t.contains("char")
        || t.contains("time");
    if is_string_type {
        return format!("'{}'", v.replace('\'', "''"));
    }
    // 数字类型：验证并原样返回
    let is_number_type = t.contains("int")
        || t.contains("long")
        || t.contains("decimal")
        || t.contains("double")
        || t.contains("float")
        || t.contains("number")
        || t.contains("numeric")
        || t.contains("big");
    if is_number_type {
        if !is_numeric(v) {
            return format!("'{}'", v.replace('\'', "''"));
        }
        return v.to_string();
    }
    // 无类型或 Auto：自动判断
    if is_numeric(v) {
        v.to_string()
    } else {
        format!("'{}'", v.replace('\'', "''"))
    }
}

fn is_numeric(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')
        && s.chars().filter(|c| *c == '.').count() <= 1
}

/// 解析参数列表，支持引号和嵌套括号
fn parse_params(param_str: &str) -> Result<Vec<Param>, String> {
    let param_str = param_str.trim();
    if param_str.is_empty() {
        return Ok(Vec::new());
    }

    // 统一中文逗号
    let normalized: String = param_str.chars().map(|c| if c == '，' { ',' } else { c }).collect();
    let items = split_top_level(&normalized, ',')?;

    let mut params = Vec::new();
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let (value, typ) = extract_value_and_type(item);
        params.push(Param { value, typ });
    }
    Ok(params)
}

/// 在顶层（不在引号和括号内）按分隔符分割
fn split_top_level(s: &str, sep: char) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut depth = 0i32;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double {
            in_single = !in_single;
        } else if c == '"' && !in_single {
            in_double = !in_double;
        } else if !in_single && !in_double {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }
        if c == sep && !in_single && !in_double && depth <= 0 {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(c);
        }
        i += 1;
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    Ok(parts)
}

/// 提取参数值和类型：`值(Type)` 或 `值`
fn extract_value_and_type(item: &str) -> (String, String) {
    let item = item.trim();
    // 找最后一对括号作为类型标记
    if let Some(open) = item.rfind('(') {
        if item.ends_with(')') {
            let inner = &item[open + 1..item.len() - 1];
            // 只有当括号内容看起来是类型（不含引号/逗号过多）时才当作类型
            let value_part = item[..open].trim();
            if !inner.contains('\'') && !inner.contains('"') && inner.len() <= 40 {
                let value = strip_quotes(value_part);
                return (value, inner.trim().to_string());
            }
        }
    }
    (strip_quotes(item), "Auto".to_string())
}

/// 去除首尾成对引号
fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    let chars: Vec<char> = s.chars().collect();
    if chars.len() >= 2 {
        let first = chars[0];
        let last = chars[chars.len() - 1];
        if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
}

/// INSERT 增强处理
struct InsertOutput {
    table_name: Option<String>,
    update_sql: Option<String>,
    warnings: Vec<String>,
    errors: Vec<String>,
}

fn handle_insert(
    sql: &str,
    params: &[Param],
    field_type_map: &std::collections::HashMap<String, String>,
    field_checks: &mut Vec<FieldCheck>,
) -> Result<InsertOutput, String> {
    // 解析表名
    let lower = sql.to_lowercase().replace(';', "").replace('\n', " ");
    let compact: String = lower.split_whitespace().collect::<Vec<_>>().join(" ");
    let table_name = parse_insert_table(&compact)?;

    // 解析字段与 values 占位符
    let (columns, values) = parse_insert_columns_values(sql)?;

    // 映射参数到字段
    let mut value_map: Vec<(String, String)> = Vec::new();
    let mut param_idx = 0;
    for (col, v) in columns.iter().zip(values.iter()) {
        if v.trim() == "?" {
            if param_idx >= params.len() {
                break;
            }
            value_map.push((col.clone(), format_param(&params[param_idx])));
            param_idx += 1;
        } else {
            value_map.push((col.clone(), v.trim().to_string()));
        }
    }

    // 生成 UPDATE（基于主键 id）
    let primary = value_map
        .iter()
        .find(|(c, _)| c.to_lowercase() == "id")
        .cloned();

    let update_sql = primary.map(|(pk_col, pk_val)| {
        let set_clauses: Vec<String> = value_map
            .iter()
            .filter(|(c, _)| !c.eq_ignore_ascii_case(&pk_col))
            .map(|(c, v)| format!("{} = {}", c, v))
            .collect();
        let set_line = set_clauses.join(",\n    ");
        format!(
            "UPDATE {} SET\n    {}\nWHERE {} = {};",
            table_name, set_line, pk_col, pk_val
        )
    });

    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    if update_sql.is_none() {
        warnings.push("INSERT 字段中未找到主键「id」，已跳过 UPDATE 语句生成".to_string());
    }

    // 字段长度校验（基于表结构）：逐字段收集检查明细
    if !field_type_map.is_empty() {
        for (col, formatted_val) in &value_map {
            // 去引号（"id" / `id`）以匹配表结构字段名
            let col_key = col
                .trim_matches(|c: char| c == '"' || c == '`')
                .to_lowercase();
            let db_type = field_type_map
                .get(&col_key)
                .cloned()
                .unwrap_or_default();
            let max_len = if db_type.is_empty() {
                None
            } else {
                get_char_max_len(&db_type)
            };
            // 计算实际长度（去除字面量引号、NULL 跳过）
            let raw = formatted_val.trim();
            let (actual_len, status) = if raw.eq_ignore_ascii_case("null") {
                (None, "skip".to_string())
            } else {
                let v = strip_literal_quotes(raw);
                let al = v.chars().count();
                let st = match max_len {
                    Some(m) if al > m => "over",
                    Some(_) => "ok",
                    None => "unknown-type",
                };
                (Some(al), st.to_string())
            };
            // 若超长，加入 errors
            if status == "over" {
                let max = max_len.unwrap_or(0);
                let al = actual_len.unwrap_or(0);
                let val_display = strip_literal_quotes(raw);
                errors.push(format!(
                    "字段「{}」值「{}」长度 {} > 限制 {}（字段类型 {}）",
                    col, val_display, al, max, db_type
                ));
            }
            field_checks.push(FieldCheck {
                field: col.clone(),
                db_type,
                max_len,
                actual_len,
                status,
            });
        }
    }

    Ok(InsertOutput {
        table_name: Some(table_name),
        update_sql,
        warnings,
        errors,
    })
}

fn parse_insert_table(compact_sql: &str) -> Result<String, String> {
    let words: Vec<&str> = compact_sql.split_whitespace().collect();
    // insert into <table>
    let mut i = 0;
    while i < words.len() {
        if words[i].eq_ignore_ascii_case("insert") {
            let mut j = i + 1;
            while j < words.len() && words[j].eq_ignore_ascii_case("into") {
                j += 1;
            }
            if j < words.len() {
                let table = words[j].trim_matches(|c: char| c == '(' || c == '`' || c == '"');
                return Ok(table.to_string());
            }
        }
        i += 1;
    }
    Err("无法识别 INSERT 语句的表名".to_string())
}

fn parse_insert_columns_values(sql: &str) -> Result<(Vec<String>, Vec<String>), String> {
    // 正则提取 insert into t (col1, col2) values (v1, v2)
    let lower_idx = sql.to_lowercase().find("values").ok_or("未找到 VALUES")?;
    let before = &sql[..lower_idx];
    let after = &sql[lower_idx + "values".len()..];

    let col_start = before.rfind('(').ok_or("未找到字段括号")?;
    let col_end = before.rfind(')').ok_or("未找到字段括号闭合")?;
    if col_start >= col_end {
        return Err("字段列表解析失败".to_string());
    }
    let col_str = &before[col_start + 1..col_end];

    let val_start = after.find('(').ok_or("未找到 VALUES 括号")?;
    let val_end = after.rfind(')').ok_or("未找到 VALUES 括号闭合")?;
    if val_start >= val_end {
        return Err("VALUES 列表解析失败".to_string());
    }
    let val_str = &after[val_start + 1..val_end];

    let columns = split_sql_items(col_str)?;
    let values = split_sql_items(val_str)?;

    if columns.len() != values.len() {
        return Err(format!(
            "字段数量（{}）与 VALUES 数量（{}）不匹配",
            columns.len(),
            values.len()
        ));
    }

    // 列名去引号/反引号，便于主键判断与表结构匹配
    let columns: Vec<String> = columns
        .into_iter()
        .map(|c| {
            c.trim()
                .trim_matches(|c: char| c == '"' || c == '`')
                .to_string()
        })
        .collect();

    Ok((columns, values))
}

/// 分割 SQL 项（忽略引号内的逗号和嵌套括号）
fn split_sql_items(s: &str) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut depth = 0i32;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double {
            if i + 1 < chars.len() && chars[i + 1] == '\'' {
                current.push(c);
                current.push(chars[i + 1]);
                i += 2;
                continue;
            }
            in_single = !in_single;
        } else if c == '"' && !in_single {
            in_double = !in_double;
        } else if !in_single && !in_double {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }
        if c == ',' && !in_single && !in_double && depth <= 0 {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(c);
        }
        i += 1;
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    if parts.is_empty() {
        return Err("未解析到有效的项列表".to_string());
    }
    Ok(parts)
}

/// 从模板中分离内嵌的 CREATE TABLE DDL。
/// 输入可能是 `INSERT ...; CREATE TABLE ...` 拼在一起，返回 (SQL部分, DDL部分)。
fn split_embedded_create_table(template: &str) -> (String, String) {
    let lower = template.to_lowercase();
    // 找 create table 关键字位置
    let ct_pos = match lower.find("create table") {
        Some(p) => p,
        None => return (template.to_string(), String::new()),
    };
    let sql_part = template[..ct_pos].trim().to_string();
    let ddl_part = template[ct_pos..].trim().to_string();
    (sql_part, ddl_part)
}

/// 解析 CREATE TABLE DDL，返回 字段名(小写) -> 类型串 的映射
fn parse_table_structure(ddl: &str) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut map = HashMap::new();

    // 逐行处理，去掉 SQL 注释（-- 开头）
    let mut in_paren = false;
    let mut paren_level = 0i32;
    let mut table_body = String::new();

    for raw_line in ddl.lines() {
        let line = match raw_line.find("--") {
            Some(idx) => &raw_line[..idx],
            None => raw_line,
        }
        .trim();
        if line.is_empty() {
            continue;
        }
        if !in_paren {
            if let Some(idx) = line.find('(') {
                in_paren = true;
                table_body.push_str(&line[idx + 1..]);
                table_body.push(' ');
            }
        } else {
            table_body.push_str(line);
            table_body.push(' ');
        }
        if in_paren {
            for ch in line.chars() {
                match ch {
                    '(' => paren_level += 1,
                    ')' => paren_level -= 1,
                    _ => {}
                }
            }
            if paren_level <= 0 {
                break;
            }
        }
    }

    // 按顶层逗号分割
    for token in split_sql_items(&table_body).unwrap_or_default() {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        // 跳过主键、外键、约束等
        if token.to_lowercase().starts_with("primary")
            || token.to_lowercase().starts_with("foreign")
            || token.to_lowercase().starts_with("unique")
            || token.to_lowercase().starts_with("check")
            || token.to_lowercase().starts_with("constraint")
            || token.to_lowercase().starts_with("key")
        {
            continue;
        }
        // 字段名是第一个单词，去掉反引号/双引号
        let mut parts = token.split_whitespace();
        let field = match parts.next() {
            Some(f) => f
                .trim_matches('`')
                .trim_matches('"')
                .to_lowercase(),
            None => continue,
        };
        // 类型可能由多个词组成，如 "character varying(20)"，拼接直到含完整括号
        let mut type_part = String::new();
        while let Some(word) = parts.next() {
            // 若类型已含完整括号（如 timestamp(6)），不再追加后续限定词
            if type_part.contains('(') && type_part.contains(')') {
                break;
            }
            let word_clean = word.trim_matches(|c: char| c == '`' || c == '"').to_string();
            if type_part.is_empty() {
                type_part = word_clean;
            } else {
                type_part.push(' ');
                type_part.push_str(&word_clean);
            }
        }
        if !type_part.is_empty() {
            map.insert(field, type_part);
        }
    }

    map
}

/// 从类型串提取 varchar/char 长度，如 varchar(50)、character varying(50) -> 50
fn get_char_max_len(type_str: &str) -> Option<usize> {
    let lower = type_str.to_lowercase();
    // 统一 character varying / varchar / character / char 带括号的情况
    let patterns = [
        "character varying(",
        "char varying(",
        "varchar(",
        "character(",
        "nvarchar(",
        "char(",
    ];
    for prefix in patterns {
        if lower.starts_with(prefix) {
            let inner = &lower[prefix.len()..];
            let end = inner.find(')')?;
            let num_str = &inner[..end];
            if let Ok(n) = num_str.parse::<usize>() {
                return Some(n);
            }
        }
    }
    None
}

/// 去掉 SQL 字符串字面量的单引号
fn strip_literal_quotes(s: &str) -> &str {
    let s = s.trim();
    let chars: Vec<char> = s.chars().collect();
    if chars.len() >= 2 && chars[0] == '\'' && chars[chars.len() - 1] == '\'' {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// UPDATE 语句 SET 字段长度校验
fn validate_update_fields(
    sql: &str,
    params: &[Param],
    field_type_map: &std::collections::HashMap<String, String>,
    warnings: &mut Vec<String>,
    errors: &mut Vec<String>,
    field_checks: &mut Vec<FieldCheck>,
) {
    // 解析 UPDATE ... SET 之后的字段
    let lower = sql.to_lowercase();
    let set_pos = match lower.find(" set ") {
        Some(p) => p,
        None => return,
    };
    let after = &sql[set_pos + 5..];
    // 找 WHERE 之前的部分作为 SET 子句
    let set_clause = match after.to_lowercase().find(" where ") {
        Some(p) => &after[..p],
        None => after,
    };

    // 分割 SET 项，拿到 字段 和 值（可能带类型标注或占位符）
    let items = match split_sql_items(set_clause) {
        Ok(items) => items,
        Err(_) => return,
    };

    let mut param_idx = 0;
    let mut matched = false;
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        // col = value
        let eq = match item.find('=') {
            Some(p) => p,
            None => continue,
        };
        let field = item[..eq]
            .trim()
            .trim_matches('`')
            .trim_matches('"')
            .to_string();
        let mut raw_value = item[eq + 1..].trim().to_string();

        // 若是占位符，取对应参数
        if raw_value == "?" || raw_value == "?," {
            raw_value = raw_value.trim_matches(',').to_string();
            if raw_value == "?" {
                if param_idx < params.len() {
                    raw_value = format_param(&params[param_idx]);
                    param_idx += 1;
                }
            }
        }

        // 字段长度校验 + 收集明细
        let db_type = field_type_map
            .get(&field.to_lowercase())
            .cloned()
            .unwrap_or_default();
        let max_len = if db_type.is_empty() {
            None
        } else {
            get_char_max_len(&db_type)
        };
        let raw = raw_value.trim();
        let (actual_len, status) = if raw.eq_ignore_ascii_case("null") || raw == "?" {
            (None, "skip".to_string())
        } else {
            let v = strip_literal_quotes(raw);
            let al = v.chars().count();
            let st = match max_len {
                Some(m) if al > m => "over",
                Some(_) => "ok",
                None => "unknown-type",
            };
            (Some(al), st.to_string())
        };
        if status == "over" {
            let max = max_len.unwrap_or(0);
            let al = actual_len.unwrap_or(0);
            let val_display = strip_literal_quotes(raw);
            errors.push(format!(
                "字段「{}」值「{}」长度 {} > 限制 {}（字段类型 {}）",
                field, val_display, al, max, db_type
            ));
            matched = true;
        }
        field_checks.push(FieldCheck {
            field,
            db_type,
            max_len,
            actual_len,
            status,
        });
    }

    if matched {
        warnings.push("已根据表结构校验 UPDATE 字段长度".to_string());
    }
}
