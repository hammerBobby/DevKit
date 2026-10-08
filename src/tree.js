// 将 JSON 渲染为可折叠/展开的树形结构

function fmtValue(v) {
  if (v === null) return `<span class="tree-null">null</span>`;
  if (typeof v === "string")
    return `<span class="tree-string">"${escapeHtml(v)}"</span>`;
  if (typeof v === "number")
    return `<span class="tree-number">${v}</span>`;
  if (typeof v === "boolean")
    return `<span class="tree-boolean">${v}</span>`;
  return "";
}

function escapeHtml(s) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function quoteKey(k) {
  return `<span class="tree-key">"${escapeHtml(String(k))}"</span>`;
}

/**
 * 渲染节点
 * @param {*} value 值
 * @param {string|null} key 键名（根节点为 null）
 * @param {boolean} isRoot 是否根节点
 */
function renderNode(value, key, isRoot) {
  const isObj = value !== null && typeof value === "object" && !Array.isArray(value);
  const isArr = Array.isArray(value);

  // 叶子
  if (!isObj && !isArr) {
    if (isRoot) {
      return `<div class="tree-row"><span class="tree-punc">${fmtValue(value)}</span></div>`;
    }
    return `<div class="tree-row"><span class="tree-toggle">&nbsp;</span>${quoteKey(key)}<span class="tree-punc">: </span>${fmtValue(value)}</div>`;
  }

  // 容器节点
  const count = isArr ? value.length : Object.keys(value).length;
  const emptyContent = count === 0;
  const openBrace = isArr ? "[" : "{";
  const closeBrace = isArr ? "]" : "}";

  if (isRoot) {
    if (emptyContent) {
      return `<div class="tree-row"><span class="tree-punc">${openBrace}${closeBrace}</span></div>`;
    }
    const children = isArr
      ? value.map((v, i) => renderNode(v, i, false)).join("")
      : Object.keys(value)
          .map((k) => renderNode(value[k], k, false))
          .join("");
    return `
      <div class="tree-row"><span class="tree-toggle" onclick="this.parentNode.nextElementSibling.hidden=!this.parentNode.nextElementSibling.hidden;this.textContent=this.textContent==='▸'?'▾':'▸'">▾</span><span class="tree-punc">${openBrace}</span></div>
      <div class="tree-node">
        ${children}
      </div>
      <div class="tree-row"><span class="tree-punc">${closeBrace}</span></div>
    `;
  }

  // 带键的容器节点
  if (emptyContent) {
    return `<div class="tree-row"><span class="tree-toggle">&nbsp;</span>${quoteKey(key)}<span class="tree-punc">: ${openBrace}${closeBrace}</span></div>`;
  }

  const children = isArr
    ? value.map((v, i) => renderNode(v, i, false)).join("")
    : Object.keys(value)
        .map((k) => renderNode(value[k], k, false))
        .join("");

  return `
    <div class="tree-row"><span class="tree-toggle" onclick="this.parentNode.nextElementSibling.hidden=!this.parentNode.nextElementSibling.hidden;this.textContent=this.textContent==='▸'?'▾':'▸'">▾</span>${quoteKey(key)}<span class="tree-punc">: ${openBrace}</span><span class="tree-collapsed-hint" style="display:none"></span></div>
    <div class="tree-node">
      ${children}
    </div>
    <div class="tree-row"><span class="tree-punc">${closeBrace}</span></div>
  `;
}

export function renderJsonTree(data, container) {
  container.innerHTML = renderNode(data, null, true);
}
