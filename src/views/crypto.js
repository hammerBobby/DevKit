import { invoke } from "../api.js";
import { showMsg, bindCopy } from "../ui.js";

export function initCryptoView() {
  const view = document.getElementById("crypto-view");
  view.innerHTML = `
    <div class="view-header">
      <div>
        <div class="view-title">加密工具</div>
        <div class="view-subtitle">Base64 编解码 · URL 编解码 · MD5 编码</div>
      </div>
    </div>

    <div class="toolbar">
      <button class="btn" id="crypto-clear">清空</button>
    </div>
    <div class="status-msg" id="crypto-status"></div>

    <!-- Tab 切换 -->
    <div class="json-tabs">
      <button class="json-tab active" data-tab="base64">Base64 编解码</button>
      <button class="json-tab" data-tab="url">URL 编解码</button>
      <button class="json-tab" data-tab="md5">MD5 编码</button>
      <button class="json-tab" data-tab="help">说明</button>
    </div>

    <!-- Base64 -->
    <div class="json-tab-body" id="tab-base64">
      <div class="crypto-card" style="flex:1;min-height:0">
        <textarea id="crypto-base64-input" class="code" spellcheck="false" placeholder="输入内容"></textarea>
        <div class="crypto-actions">
          <button class="btn btn-primary" data-crypto-op="base64-encode">编码</button>
          <button class="btn" data-crypto-op="base64-decode">解码</button>
        </div>
        <textarea id="crypto-base64-output" class="code" spellcheck="false" placeholder="结果" readonly></textarea>
        <div class="crypto-actions">
          <button class="mini-btn" id="copy-base64">复制</button>
          <button class="mini-btn" id="swap-base64">上下互换</button>
        </div>
      </div>
    </div>

    <!-- URL -->
    <div class="json-tab-body" id="tab-url" style="display:none">
      <div class="crypto-card" style="flex:1;min-height:0">
        <textarea id="crypto-url-input" class="code" spellcheck="false" placeholder="输入内容"></textarea>
        <div class="crypto-actions">
          <button class="btn btn-primary" data-crypto-op="url-encode">编码</button>
          <button class="btn" data-crypto-op="url-decode">解码</button>
        </div>
        <textarea id="crypto-url-output" class="code" spellcheck="false" placeholder="结果" readonly></textarea>
        <div class="crypto-actions">
          <button class="mini-btn" id="copy-url">复制</button>
          <button class="mini-btn" id="swap-url">上下互换</button>
        </div>
      </div>
    </div>

    <!-- MD5 -->
    <div class="json-tab-body" id="tab-md5" style="display:none">
      <div class="crypto-card" style="flex:1;min-height:0">
        <textarea id="crypto-md5-input" class="code" spellcheck="false" placeholder="输入明文"></textarea>
        <div class="crypto-actions">
          <button class="btn btn-primary" data-crypto-op="md5-encode">MD5（小写）</button>
          <button class="btn" data-crypto-op="md5-encode-upper">MD5（大写）</button>
        </div>
        <textarea id="crypto-md5-output" class="code" spellcheck="false" placeholder="MD5 结果" readonly></textarea>
        <div class="crypto-actions">
          <button class="mini-btn" id="copy-md5">复制</button>
        </div>
      </div>
    </div>

    <!-- 说明 -->
    <div class="json-tab-body" id="tab-help" style="display:none">
      <div class="crypto-card" style="flex:1;min-height:0">
        <div style="color:var(--color-text-secondary);font-size:13px;line-height:1.8;user-select:text">
          <p>• Base64：标准 Base64 编解码，用于数据传输/存储。</p>
          <p>• URL：对特殊字符进行百分号编码。</p>
          <p>• MD5：128 位哈希，常用于校验和、密码摘要（不可逆）。</p>
          <p>• 点击「上下互换」可将结果回填到输入框，便于多步处理。</p>
          <p>• 点击顶部「清空」可清空当前标签页的输入与结果两个文本框。</p>
        </div>
      </div>
    </div>
  `;

  const status = view.querySelector("#crypto-status");

  // ---------- Tab 切换 ----------
  let currentTab = "base64";
  const tabs = view.querySelectorAll(".json-tab");
  const bodies = view.querySelectorAll(".json-tab-body");
  tabs.forEach((tab) => {
    tab.onclick = () => {
      currentTab = tab.dataset.tab;
      tabs.forEach((t) => t.classList.toggle("active", t === tab));
      bodies.forEach((b) => {
        b.style.display = b.id === "tab-" + currentTab ? "" : "none";
      });
    };
  });

  // ---------- 操作命令 ----------
  const cmdMap = {
    "base64-encode": { cmd: "base64_encode", inputId: "crypto-base64-input", outputId: "crypto-base64-output" },
    "base64-decode": { cmd: "base64_decode", inputId: "crypto-base64-input", outputId: "crypto-base64-output" },
    "url-encode": { cmd: "url_encode", inputId: "crypto-url-input", outputId: "crypto-url-output" },
    "url-decode": { cmd: "url_decode", inputId: "crypto-url-input", outputId: "crypto-url-output" },
    "md5-encode": { cmd: "md5_encode", inputId: "crypto-md5-input", outputId: "crypto-md5-output" },
    "md5-encode-upper": { cmd: "md5_encode_upper", inputId: "crypto-md5-input", outputId: "crypto-md5-output" },
  };

  view.querySelectorAll("[data-crypto-op]").forEach((btn) => {
    btn.onclick = async () => {
      const { cmd, inputId, outputId } = cmdMap[btn.dataset.cryptoOp];
      const value = view.querySelector(`#${inputId}`).value;
      if (!value.trim()) {
        showMsg(status, "请输入内容", "warning");
        return;
      }
      try {
        const result = await invoke(cmd, { input: value });
        view.querySelector(`#${outputId}`).value = result;
        showMsg(status, "处理成功", "success");
      } catch (e) {
        showMsg(status, e, "error");
      }
    };
  });

  const bindSwap = (inputId, outputId) => {
    const inEl = view.querySelector(inputId);
    const outEl = view.querySelector(outputId);
    return () => {
      if (outEl.value) {
        inEl.value = outEl.value;
        showMsg(status, "已互换（结果回填到输入框）", "success");
      } else {
        showMsg(status, "暂无可回填的结果", "warning");
      }
    };
  };

  bindCopy(view.querySelector("#copy-base64"), () =>
    view.querySelector("#crypto-base64-output").value
  );
  bindCopy(view.querySelector("#copy-url"), () =>
    view.querySelector("#crypto-url-output").value
  );
  bindCopy(view.querySelector("#copy-md5"), () =>
    view.querySelector("#crypto-md5-output").value
  );

  view.querySelector("#swap-base64").onclick = bindSwap(
    "#crypto-base64-input",
    "#crypto-base64-output"
  );
  view.querySelector("#swap-url").onclick = bindSwap(
    "#crypto-url-input",
    "#crypto-url-output"
  );

  // ---------- 清空：清除当前 Tab 的输入与结果两个文本框 ----------
  const tabBoxes = {
    base64: ["#crypto-base64-input", "#crypto-base64-output"],
    url: ["#crypto-url-input", "#crypto-url-output"],
    md5: ["#crypto-md5-input", "#crypto-md5-output"],
  };
  view.querySelector("#crypto-clear").onclick = () => {
    const boxes = tabBoxes[currentTab];
    if (boxes) {
      boxes.forEach((id) => {
        const el = view.querySelector(id);
        if (el) el.value = "";
      });
      showMsg(status, "已清空当前标签页", "success");
    } else {
      showMsg(status, "当前标签页无可清空内容", "warning");
    }
  };
}
