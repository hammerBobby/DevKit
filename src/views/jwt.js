import { invoke } from "../api.js";
import { showMsg, bindCopy } from "../ui.js";

export function initJwtView() {
  const view = document.getElementById("jwt-view");
  view.innerHTML = `
    <div class="view-header">
      <div>
        <div class="view-title">JWT 解析</div>
        <div class="view-subtitle">解析 JWT Token 的 Header、Payload 与校验信息</div>
      </div>
    </div>
    <div class="editor-area">
      <div class="toolbar">
        <button class="btn btn-primary" id="jwt-parse">解析</button>
        <button class="btn" id="jwt-sample">示例</button>
        <span style="flex:1"></span>
        <button class="btn btn-danger" id="jwt-clear">清空</button>
      </div>
      <div class="status-msg" id="jwt-status"></div>
      <div class="jwt-meta" id="jwt-meta"></div>
      <div class="split-editor">
        <div class="pane">
          <div class="pane-header"><span>输入 JWT Token</span></div>
          <textarea id="jwt-input" class="code" spellcheck="false" placeholder="粘贴 JWT Token（header.payload.signature）"></textarea>
        </div>
        <div class="pane">
          <div class="pane-header">
            <span>解析结果</span>
            <div class="right">
              <button class="mini-btn" id="jwt-copy-header">复制Header</button>
              <button class="mini-btn" id="jwt-copy-payload">复制Payload</button>
            </div>
          </div>
          <textarea id="jwt-output" class="code" spellcheck="false" readonly></textarea>
        </div>
      </div>
    </div>
  `;

  const input = view.querySelector("#jwt-input");
  const output = view.querySelector("#jwt-output");
  const meta = view.querySelector("#jwt-meta");
  const status = view.querySelector("#jwt-status");

  let lastHeader = "";
  let lastPayload = "";

  function renderMeta(r) {
    meta.innerHTML = "";
    const timeStr = (t) =>
      t == null ? "-" : new Date(t * 1000).toLocaleString("zh-CN");
    const badge = r.valid
      ? '<span class="jwt-badge valid">✓ 有效</span>'
      : '<span class="jwt-badge invalid">✗ 无效/已过期</span>';
    meta.innerHTML = `
      <div class="jwt-meta-card"><div class="label">签名算法 (alg)</div><div class="value">${r.header?.alg ?? "-"}</div></div>
      <div class="jwt-meta-card"><div class="label">Token 类型 (typ)</div><div class="value">${r.header?.typ ?? "-"}</div></div>
      <div class="jwt-meta-card"><div class="label">签发时间 (iat)</div><div class="value">${timeStr(r.issued_at)}</div></div>
      <div class="jwt-meta-card"><div class="label">过期时间 (exp)</div><div class="value">${timeStr(r.exp)}</div></div>
      <div class="jwt-meta-card"><div class="label">生效时间 (nbf)</div><div class="value">${timeStr(r.not_before)}</div></div>
      <div class="jwt-meta-card"><div class="label">校验状态</div><div class="value">${badge}</div></div>
    `;
  }

  view.querySelector("#jwt-parse").onclick = async () => {
    const token = input.value.trim();
    if (!token) {
      showMsg(status, "请输入 JWT Token", "warning");
      return;
    }
    try {
      const r = await invoke("jwt_decode", { token });
      lastHeader = JSON.stringify(r.header, null, 2);
      lastPayload = JSON.stringify(r.payload, null, 2);
      output.value = `/* Header */\n${lastHeader}\n\n/* Payload */\n${lastPayload}\n\n/* Signature */\n${r.signature}`;
      renderMeta(r);
      if (r.error) {
        showMsg(status, r.error, "error");
      } else if (r.valid) {
        showMsg(status, "解析成功，Token 有效", "success");
      } else {
        showMsg(status, "解析成功，但 Token 已过期或尚未生效", "warning");
      }
    } catch (e) {
      showMsg(status, e, "error");
    }
  };

  bindCopy(view.querySelector("#jwt-copy-header"), () => lastHeader);
  bindCopy(view.querySelector("#jwt-copy-payload"), () => lastPayload);

  view.querySelector("#jwt-clear").onclick = () => {
    input.value = "";
    output.value = "";
    meta.innerHTML = "";
    status.style.display = "none";
  };

  view.querySelector("#jwt-sample").onclick = () => {
    // 一个示例 JWT
    const header = btoa(JSON.stringify({ alg: "HS256", typ: "JWT" }));
    const payload = btoa(
      JSON.stringify({
        sub: "1234567890",
        name: "DevKit User",
        iat: Math.floor(Date.now() / 1000) - 1000,
        exp: Math.floor(Date.now() / 1000) + 3600,
      })
    );
    const sig = "SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    input.value = `${header}.${payload}.${sig}`;
  };
}
