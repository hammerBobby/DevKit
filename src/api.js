// 封装 Tauri invoke。
// 在 Tauri 环境用 window.__TAURI__.core.invoke；
// 在纯浏览器预览环境（非 Tauri）降级为 JS 实现，便于 UI 预览。

const isTauri = !!(window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke);

// ---- 浏览器降级实现（供预览/测试） ----
function b64Encode(str) {
  try {
    return btoa(unescape(encodeURIComponent(str)));
  } catch {
    return btoa(str);
  }
}
function b64Decode(str) {
  try {
    return decodeURIComponent(escape(atob(str.trim())));
  } catch (e) {
    return atob(str.trim());
  }
}
function md5Hex(input, upper) {
  // 简易 MD5 纯 JS 实现（RFC 1321）
  function toUTF8Array(str) {
    const out = [];
    for (let i = 0; i < str.length; i++) {
      let c = str.charCodeAt(i);
      if (c < 0x80) out.push(c);
      else if (c < 0x800) {
        out.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f));
      } else if (c < 0xd800 || c >= 0xe000) {
        out.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
      } else {
        i++;
        c = 0x10000 + (((c & 0x3ff) << 10) | (str.charCodeAt(i) & 0x3ff));
        out.push(
          0xf0 | (c >> 18),
          0x80 | ((c >> 12) & 0x3f),
          0x80 | ((c >> 6) & 0x3f),
          0x80 | (c & 0x3f)
        );
      }
    }
    return out;
  }
  const bytes = toUTF8Array(input);
  const len = bytes.length;
  const bitLen = len * 8;
  // 填充
  const padded = bytes.slice();
  padded.push(0x80);
  while (padded.length % 64 !== 56) padded.push(0);
  // 追加长度（64位，小端）
  for (let i = 0; i < 8; i++) padded.push((bitLen / Math.pow(2, 8 * i)) & 0xff);

  const K = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a,
    0xa8304613, 0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
    0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340,
    0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8,
    0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
    0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
    0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92,
    0xffeff47d, 0x85845dd1, 0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
    0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
  ];
  const S = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
  ];
  const rnd = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    1, 6, 11, 0, 5, 10, 15, 4, 9, 14, 3, 8, 13, 2, 7, 12,
    5, 8, 11, 14, 1, 4, 7, 10, 13, 0, 3, 6, 9, 12, 15, 2,
    0, 7, 14, 5, 12, 3, 10, 1, 8, 15, 6, 13, 4, 11, 2, 9];

  let a0 = 0x67452301, b0 = 0xefcdab89, c0 = 0x98badcfe, d0 = 0x10325476;

  const add = (a, b) => (a + b) & 0xffffffff;
  const rotl = (x, c) => ((x << c) | (x >>> (32 - c))) & 0xffffffff;

  for (let i = 0; i < padded.length; i += 64) {
    const M = [];
    for (let j = 0; j < 16; j++) {
      M[j] = padded[i + j * 4] |
        (padded[i + j * 4 + 1] << 8) |
        (padded[i + j * 4 + 2] << 16) |
        (padded[i + j * 4 + 3] << 24);
    }
    let A = a0, B = b0, C = c0, D = d0;
    for (let k = 0; k < 64; k++) {
      let F, g;
      if (k < 16) { F = (B & C) | (~B & D); g = k; }
      else if (k < 32) { F = (D & B) | (~D & C); g = (5 * k + 1) % 16; }
      else if (k < 48) { F = B ^ C ^ D; g = (3 * k + 5) % 16; }
      else { F = C ^ (B | ~D); g = (7 * k) % 16; }
      const tmp = D;
      D = C;
      C = B;
      B = add(B, rotl(add(add(A, F), add(K[k], M[g])), S[k]));
      A = tmp;
    }
    a0 = add(a0, A); b0 = add(b0, B); c0 = add(c0, C); d0 = add(d0, D);
  }

  const toHex = (n) => {
    let s = "";
    for (let i = 0; i < 4; i++) {
      s += ((n >> (i * 8 + 4)) & 0xf).toString(16);
      s += ((n >> (i * 8)) & 0xf).toString(16);
    }
    return s;
  };
  let hex = toHex(a0) + toHex(b0) + toHex(c0) + toHex(d0);
  return upper ? hex.toUpperCase() : hex;
}

function base64UrlDecode(seg) {
  let s = seg.replace(/-/g, "+").replace(/_/g, "/");
  while (s.length % 4 !== 0) s += "=";
  return JSON.parse(b64Decode(s));
}

// 浏览器端 JSON 工具（与 Rust 命令行为对齐）
function sanitizeJsonCtrl(input) {
  let out = "", inStr = false;
  for (let i = 0; i < input.length; i++) {
    const c = input[i];
    if (c === '"') {
      if (inStr && input[i + 1] === '"') { out += '""'; i++; continue; }
      inStr = !inStr; out += c; continue;
    }
    if (inStr && c.charCodeAt(0) < 0x20) {
      out += "\\u" + c.charCodeAt(0).toString(16).padStart(4, "0");
      continue;
    }
    out += c;
  }
  return out;
}
function jsJsonFormat(input) {
  return JSON.stringify(JSON.parse(sanitizeJsonCtrl(input)), null, 2);
}
function jsJsonCompact(input) {
  return JSON.stringify(JSON.parse(sanitizeJsonCtrl(input)));
}
function jsJsonEscape(input) {
  // 整体作为字符串值序列化 -> 带引号 + 内部转义，与 jsJsonUnescape 互逆
  return JSON.stringify(input);
}
function jsJsonUnescape(input) {
  const s = input.trim();
  // 1) 先尝试作为 JSON 字符串字面量解析
  try {
    const unescaped = JSON.parse(s);
    if (typeof unescaped === "string") {
      try { return JSON.stringify(JSON.parse(unescaped), null, 2); } catch { return unescaped; }
    }
  } catch {}
  // 2) 回退：手动反转义常见转义序列（无需外层引号）
  const unescaped = s.replace(/\\(["\\/bfnrt])/g, (m, p) =>
    ({ '"': '"', "\\": "\\", "/": "/", b: "\b", f: "\f", n: "\n", r: "\r", t: "\t" }[p] ?? m)
  ).replace(/\\u([0-9a-fA-F]{4})/g, (_, h) => String.fromCharCode(parseInt(h, 16)));
  try { return JSON.stringify(JSON.parse(unescaped), null, 2); } catch { return unescaped; }
}

// SQL 替换（浏览器降级，与 Rust 行为对齐）
function jsSqlReplace(template, tableStructure) {
  function splitTop(s, sep) {
    const parts = [];
    let cur = "";
    let inS = false, inD = false, depth = 0;
    for (let i = 0; i < s.length; i++) {
      const c = s[i];
      if (c === "'" && !inD) inS = !inS;
      else if (c === '"' && !inS) inD = !inD;
      else if (!inS && !inD) {
        if (c === "(" || c === "[" || c === "{") depth++;
        else if (c === ")" || c === "]" || c === "}") depth--;
      }
      if (c === sep && !inS && !inD && depth <= 0) { parts.push(cur.trim()); cur = ""; }
      else cur += c;
    }
    if (cur.trim()) parts.push(cur.trim());
    return parts;
  }
  function isNum(s) {
    return s !== "" && /^-?\d*\.?\d+$/.test(s) && (s.match(/\./g) || []).length <= 1;
  }
  function fmtParam(v, t) {
    v = v.trim();
    if (v.toLowerCase() === "null") return "NULL";
    const tl = (t || "").toLowerCase();
    const strType = /string|date|timestamp|char|time/.test(tl);
    const numType = /int|long|decimal|double|float|number|numeric|big/.test(tl);
    if (strType) return "'" + v.replace(/'/g, "''") + "'";
    if (numType) return isNum(v) ? v : "'" + v.replace(/'/g, "''") + "'";
    return isNum(v) ? v : "'" + v.replace(/'/g, "''") + "'";
  }
  function countQ(sql) {
    let n = 0, inS = false, inD = false;
    for (let i = 0; i < sql.length; i++) {
      const c = sql[i];
      if (c === "'" && !inD) { if (sql[i + 1] === "'") { i++; continue; } inS = !inS; }
      else if (c === '"' && !inS) inD = !inD;
      else if (c === "?" && !inS && !inD) n++;
    }
    return n;
  }
  function replaceQ(sql, params) {
    let out = "", idx = 0, inS = false, inD = false;
    for (let i = 0; i < sql.length; i++) {
      const c = sql[i];
      if (c === "'" && !inD) { if (sql[i + 1] === "'") { out += "''"; i++; continue; } inS = !inS; out += c; }
      else if (c === '"' && !inS) { inD = !inD; out += c; }
      else if (c === "?" && !inS && !inD) { out += fmtParam(params[idx].value, params[idx].type); idx++; }
      else out += c;
    }
    return out;
  }
  function stripQuotes(s) {
    s = s.trim();
    if (s.length >= 2 && ((s[0] === '"' && s[s.length - 1] === '"') || (s[0] === "'" && s[s.length - 1] === "'"))) {
      return s.slice(1, -1);
    }
    return s;
  }
  function extractItem(item) {
    item = item.trim();
    const open = item.lastIndexOf("(");
    if (open !== -1 && item.endsWith(")")) {
      const inner = item.slice(open + 1, -1);
      const valPart = item.slice(0, open).trim();
      if (!inner.includes("'") && !inner.includes('"') && inner.length <= 40) {
        return { value: stripQuotes(valPart), type: inner.trim() };
      }
    }
    return { value: stripQuotes(item), type: "Auto" };
  }

  // ---- 表结构解析与字段长度校验 ----
  function parseTableStructure(ddl) {
    const map = {};
    if (!ddl || !ddl.trim()) return map;
    let body = "";
    ddl.split("\n").forEach((rawLine) => {
      const line = rawLine.replace(/--.*$/, "").trim();
      if (!line) return;
      body += line + " ";
    });
    const parenStart = body.indexOf("(");
    const parenEnd = body.lastIndexOf(")");
    if (parenStart === -1 || parenEnd === -1 || parenEnd <= parenStart) return map;
    const inner = body.slice(parenStart + 1, parenEnd);
    splitTop(inner, ",").forEach((token) => {
      token = token.trim();
      if (!token) return;
      if (/^(primary|foreign|unique|check|constraint|key)/i.test(token)) return;
      const parts = token.split(/\s+/);
      if (parts.length >= 2) {
        const field = parts[0].replace(/[`"]/g, "").toLowerCase();
        // 类型可能是多词：character varying(20)
        let typeStr = parts[1].replace(/[`"]/g, "");
        for (let i = 2; i < parts.length; i++) {
          // 若类型已含完整括号（如 timestamp(6)），不再追加后续限定词
          if (typeStr.includes("(") && typeStr.includes(")")) break;
          typeStr += " " + parts[i].replace(/[`"]/g, "");
        }
        map[field] = typeStr;
      }
    });
    return map;
  }
  function getCharMaxLen(typeStr) {
    const lower = (typeStr || "").toLowerCase();
    let m = lower.match(/character\s+varying\((\d+)\)/);
    if (m) return parseInt(m[1], 10);
    m = lower.match(/char\s+varying\((\d+)\)/);
    if (m) return parseInt(m[1], 10);
    m = lower.match(/varchar\((\d+)\)/);
    if (m) return parseInt(m[1], 10);
    m = lower.match(/nvarchar\((\d+)\)/);
    if (m) return parseInt(m[1], 10);
    m = lower.match(/character\((\d+)\)/);
    if (m) return parseInt(m[1], 10);
    m = lower.match(/char\((\d+)\)/);
    if (m) return parseInt(m[1], 10);
    return null;
  }
  function stripLitQuotes(s) {
    s = s.trim();
    if (s.length >= 2 && s[0] === "'" && s[s.length - 1] === "'") return s.slice(1, -1);
    return s;
  }
  function validateField(field, formattedVal, typeMap) {
    const dbType = typeMap[field.toLowerCase()];
    if (!dbType) return null;
    const maxLen = getCharMaxLen(dbType);
    if (maxLen == null) return null;
    const raw = formattedVal.trim();
    if (raw.toLowerCase() === "null") return null;
    const val = stripLitQuotes(raw);
    const actual = [...val].length;
    if (actual > maxLen) {
      return `字段「${field}」值「${val}」长度 ${actual} > 限制 ${maxLen}（字段类型 ${dbType}）`;
    }
    return null;
  }

  template = (template || "").trim();
  if (!template) throw new Error("请输入 SQL 模板");
  // 从模板中分离内嵌的 CREATE TABLE（支持 INSERT; CREATE TABLE 拼在一起）
  const ctIdx = template.toLowerCase().indexOf("create table");
  let sqlTemplatePart = template;
  if (ctIdx !== -1) {
    sqlTemplatePart = template.slice(0, ctIdx).trim();
    const embeddedDdl = template.slice(ctIdx).trim();
    if (!tableStructure || !tableStructure.trim()) {
      tableStructure = embeddedDdl;
    }
  }
  const typeMap = parseTableStructure(tableStructure);
  let sqlPart = sqlTemplatePart, paramPart = "";
  const seps = ["，执行参数：", ", 执行参数：", ";执行参数：", ",执行参数：", "；执行参数："];
  for (const sep of seps) {
    const idx = template.indexOf(sep);
    if (idx !== -1) { sqlPart = template.slice(0, idx).trim(); paramPart = template.slice(idx + sep.length).trim(); break; }
  }
  const params = paramPart ? splitTop(paramPart.replace(/，/g, ","), ",").filter((x) => x.trim()).map(extractItem) : [];
  const qCount = countQ(sqlPart);
  if (params.length !== qCount) {
    throw new Error(`参数数量不匹配：SQL 中有 ${qCount} 个 ?，但提供了 ${params.length} 个参数`);
  }
  const sql = replaceQ(sqlPart, params);
  const lower = sqlPart.trim().toLowerCase();
  const warnings = [];
  const errors = [];
  const fieldChecks = [];
  let updateSql = null;
  let tableName = null;
  function makeCheck(field, rawVal) {
    // 去引号以匹配表结构字段名
    const fieldKey = field.replace(/^[`"]+|[`"]+$/g, "").toLowerCase();
    const dbType = typeMap[fieldKey] || "";
    const maxLen = dbType ? getCharMaxLen(dbType) : null;
    const r = (rawVal || "").trim();
    let actualLen = null, status = "skip";
    if (r.toLowerCase() === "null" || r === "?") {
      status = "skip";
    } else {
      const v = stripLitQuotes(r);
      actualLen = [...v].length;
      if (maxLen == null) status = "unknown-type";
      else if (actualLen > maxLen) status = "over";
      else status = "ok";
    }
    return { field, db_type: dbType, max_len: maxLen, actual_len: actualLen, status };
  }
  if (lower.startsWith("insert")) {
    try {
      const tMatch = sqlPart.trim().match(/insert\s+into\s+([\w.`"]+)/i);
      tableName = tMatch ? tMatch[1].replace(/[`"]/g, "") : null;
      const colsMatch = sqlPart.match(/\(([\s\S]*?)\)\s*values\s*\(([\s\S]*?)\)/i);
      if (colsMatch) {
        const cols = splitTop(colsMatch[1], ",").map((c) => c.trim().replace(/"/g, ""));
        const vals = splitTop(colsMatch[2], ",").map((c) => c.trim());
        let pairsIdx = 0;
        const pairs = cols.map((c, i) => [c, vals[i] === "?" ? fmtParam(params[pairsIdx++], "") : vals[i]]);
        // 字段长度校验 + 明细
        pairs.forEach(([c, v]) => {
          const chk = makeCheck(c, v);
          fieldChecks.push(chk);
          if (chk.status === "over") {
            const val = stripLitQuotes(v.trim());
            errors.push(`字段「${c}」值「${val}」长度 ${chk.actual_len} > 限制 ${chk.max_len}（字段类型 ${chk.db_type}）`);
          }
        });
        const pk = pairs.find(([c]) => c.toLowerCase() === "id");
        if (pk) {
          const setClauses = pairs.filter(([c]) => c.toLowerCase() !== "id").map(([c, v]) => `${c} = ${v}`);
          updateSql = `UPDATE ${tableName} SET\n    ${setClauses.join(",\n    ")}\nWHERE ${pk[0]} = ${pk[1]};`;
        } else {
          warnings.push("INSERT 字段中未找到主键「id」，已跳过 UPDATE 语句生成");
        }
      }
    } catch (e) {
      warnings.push("INSERT 增强处理失败（已回退为普通替换）：" + e.message);
    }
  } else if (lower.startsWith("update") && Object.keys(typeMap).length) {
    const setPos = lower.indexOf(" set ");
    if (setPos !== -1) {
      let after = sqlPart.slice(setPos + 5);
      const wherePos = after.toLowerCase().indexOf(" where ");
      if (wherePos !== -1) after = after.slice(0, wherePos);
      splitTop(after, ",").forEach((item) => {
        item = item.trim();
        const eq = item.indexOf("=");
        if (eq === -1) return;
        const field = item.slice(0, eq).trim().replace(/[`"]/g, "");
        const rawVal = item.slice(eq + 1).trim();
        const chk = makeCheck(field, rawVal);
        fieldChecks.push(chk);
        if (chk.status === "over") {
          const val = stripLitQuotes(rawVal);
          errors.push(`字段「${field}」值「${val}」长度 ${chk.actual_len} > 限制 ${chk.max_len}（字段类型 ${chk.db_type}）`);
        }
      });
    }
  }
  return { sql, update_sql: updateSql, warnings, errors, table_name: tableName, field_checks: fieldChecks };
}

// 命令分发：优先 Rust，降级 JS
const cmdHandlers = {
  md5_encode: (args) => md5Hex(args.input, false),
  md5_encode_upper: (args) => md5Hex(args.input, true),
  base64_encode: (args) => b64Encode(args.input),
  base64_decode: (args) => b64Decode(args.input),
  url_encode: (args) => encodeURIComponent(args.input),
  url_decode: (args) => decodeURIComponent(args.input),
  json_format: (args) => jsJsonFormat(args.input),
  json_compact: (args) => jsJsonCompact(args.input),
  json_escape: (args) => jsJsonEscape(args.input),
  json_unescape: (args) => jsJsonUnescape(args.input),
  sql_replace: (args) => jsSqlReplace(args.sqlTemplate, args.tableStructure),
  jwt_decode: (args) => {
    const parts = args.token.trim().split(".");
    if (parts.length !== 3) throw new Error("不是有效的 JWT 格式");
    const header = base64UrlDecode(parts[0]);
    const payload = base64UrlDecode(parts[1]);
    const now = Math.floor(Date.now() / 1000);
    const exp = payload.exp != null ? Number(payload.exp) : null;
    const nbf = payload.nbf != null ? Number(payload.nbf) : null;
    const iat = payload.iat != null ? Number(payload.iat) : null;
    const valid = !(exp != null && now >= exp) && !(nbf != null && now < nbf);
    return {
      header, payload, signature: parts[2],
      expired: exp != null ? now >= exp : null,
      exp, not_before: nbf, issued_at: iat, valid, error: null,
    };
  },
};

async function invoke(cmd, args = {}) {
  if (isTauri) {
    return window.__TAURI__.core.invoke(cmd, args);
  }
  const handler = cmdHandlers[cmd];
  if (handler) {
    return handler(args);
  }
  throw new Error(`未知命令：${cmd}`);
}

export { invoke, isTauri };
