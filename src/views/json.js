import { invoke } from "../api.js";
import { showMsg, copyText } from "../ui.js";
import { renderJsonTree } from "../tree.js";

export function initJsonView() {
  const view = document.getElementById("json-view");
  view.innerHTML = `
    <div class="view-header">
      <div>
        <div class="view-title">JSON 工具</div>
        <div class="view-subtitle">格式化/压缩 · 转义/去转义</div>
      </div>
    </div>

    <div class="toolbar">
      <button class="btn" id="json-sample">示例</button>
      <button class="btn" id="json-clear">清空</button>
    </div>
    <div class="status-msg" id="json-status"></div>

    <div class="split-editor">
      <!-- 左：输入 -->
      <div class="pane">
        <div class="pane-header">
          <span>输入 JSON</span>
          <div class="right">
            <button class="mini-btn btn-primary" data-act="format">格式化</button>
            <button class="mini-btn" data-act="compact">压缩</button>
            <button class="mini-btn" data-act="escape">转义</button>
            <button class="mini-btn" data-act="unescape">去除转义</button>
            <button class="mini-btn" data-act="copy">复制</button>
          </div>
        </div>
        <textarea id="json-input" class="code" spellcheck="false" wrap="soft" placeholder='粘贴 JSON，例如：{"name":"DevKit","tags":["dev","tool"]}'></textarea>
      </div>

      <!-- 右：输出 -->
      <div class="pane">
        <div class="pane-header">
          <span>输出</span>
          <div class="right">
            <button class="mini-btn btn-primary" data-act="format">格式化</button>
            <button class="mini-btn" data-act="compact">压缩</button>
            <button class="mini-btn" data-act="escape">转义</button>
            <button class="mini-btn" data-act="unescape">去除转义</button>
            <button class="mini-btn" data-act="tree">🌳 树形</button>
            <button class="mini-btn" data-act="copy">复制</button>
          </div>
        </div>
        <div class="tree-wrap" id="json-output-tree" style="display:none"></div>
        <textarea id="json-output" class="code" spellcheck="false" wrap="soft" readonly style="display:none"></textarea>
      </div>
    </div>
  `;

  const status = view.querySelector("#json-status");
  const inTa = view.querySelector("#json-input");
  const outTa = view.querySelector("#json-output");
  const outTree = view.querySelector("#json-output-tree");

  // 就地处理：读取本框内容，结果写回本框
  async function runInPlace(ta, treeWrap, cmd, title, asTree) {
    status.style.display = "none";
    const text = ta.value.trim();
    if (!text) {
      showMsg(status, "请先输入内容", "warning");
      return;
    }
    try {
      const result = await invoke(cmd, { input: text });
      if (asTree && treeWrap) {
        try {
          treeWrap.innerHTML = "";
          renderJsonTree(JSON.parse(result), treeWrap);
          ta.value = result; // 树形时仍保留文本，便于复制
          treeWrap.style.display = "";
          ta.style.display = "none";
          showMsg(status, `${title}成功`, "success");
          return;
        } catch {
          /* 解析失败则退化为文本 */
        }
      }
      if (treeWrap) {
        treeWrap.style.display = "none";
        treeWrap.innerHTML = "";
      }
      ta.style.display = "";
      ta.value = result;
      showMsg(status, `${title}成功`, "success");
    } catch (e) {
      showMsg(status, e, "error");
    }
  }

  // 左框 格式化/压缩：读取输入 → 写入右侧输出（保持原逻辑）
  async function fmtToOutput(cmd, title, asTree) {
    status.style.display = "none";
    const text = inTa.value.trim();
    if (!text) {
      showMsg(status, "请先输入 JSON", "warning");
      return;
    }
    try {
      const result = await invoke(cmd, { input: text });
      if (asTree) {
        try {
          outTree.innerHTML = "";
          renderJsonTree(JSON.parse(result), outTree);
          outTa.value = result;
          outTree.style.display = "";
          outTa.style.display = "none";
          showMsg(status, `${title}成功`, "success");
          return;
        } catch {
          /* 退化为文本 */
        }
      }
      outTree.style.display = "none";
      outTree.innerHTML = "";
      outTa.style.display = "";
      outTa.value = result;
      showMsg(status, `${title}成功`, "success");
    } catch (e) {
      showMsg(status, e, "error");
    }
  }

  async function copyFrom(ta, treeWrap) {
    const target = (ta.value || "").trim() ? ta.value : treeWrap?.innerText;
    if (!target || !target.trim()) {
      showMsg(status, "暂无可复制的内容", "warning");
      return;
    }
    await copyText(target);
    showMsg(status, "已复制到剪贴板", "success");
  }

  // 左框
  const inBox = inTa.closest(".pane");
  inBox.querySelector('[data-act="format"]').onclick = () =>
    fmtToOutput("json_format", "已格式化", true);
  inBox.querySelector('[data-act="compact"]').onclick = () =>
    fmtToOutput("json_compact", "已压缩", false);
  inBox.querySelector('[data-act="escape"]').onclick = () =>
    runInPlace(inTa, null, "json_escape", "已转义", false);
  inBox.querySelector('[data-act="unescape"]').onclick = () =>
    runInPlace(inTa, null, "json_unescape", "已去除转义", false);
  inBox.querySelector('[data-act="copy"]').onclick = () => copyFrom(inTa, null);

  // 右框（全部就地处理本框）
  const outBox = outTa.closest(".pane");
  outBox.querySelector('[data-act="format"]').onclick = () =>
    runInPlace(outTa, outTree, "json_format", "已格式化", true);
  outBox.querySelector('[data-act="compact"]').onclick = () =>
    runInPlace(outTa, outTree, "json_compact", "已压缩", false);
  outBox.querySelector('[data-act="escape"]').onclick = () =>
    runInPlace(outTa, outTree, "json_escape", "已转义", false);
  outBox.querySelector('[data-act="unescape"]').onclick = () =>
    runInPlace(outTa, outTree, "json_unescape", "已去除转义", false);
  outBox.querySelector('[data-act="tree"]').onclick = () => {
    status.style.display = "none";
    const raw = outTa.value.trim();
    if (!raw) {
      showMsg(status, "输出区没有内容", "warning");
      return;
    }
    try {
      const data = JSON.parse(raw);
      outTree.innerHTML = "";
      renderJsonTree(data, outTree);
      outTa.value = JSON.stringify(data, null, 2);
      outTree.style.display = "";
      outTa.style.display = "none";
      showMsg(status, "已切换为树形", "success");
    } catch (e) {
      showMsg(status, `JSON 解析失败：${e.message}`, "error");
    }
  };
  outBox.querySelector('[data-act="copy"]').onclick = () => copyFrom(outTa, outTree);

  // 示例 / 清空
  view.querySelector("#json-sample").onclick = () => {
    inTa.value = JSON.stringify(
      {
        name: "DevKit",
        version: "0.1.0",
        tags: ["dev", "tool", "json"],
        config: { theme: "dark", count: 5 },
        active: true,
        list: [1, "two", false, { deep: ["a", { b: 2 }] }],
      },
      null,
      2
    );
    showMsg(status, "已加载示例到输入框", "success");
  };
  view.querySelector("#json-clear").onclick = () => {
    inTa.value = "";
    outTa.value = "";
    outTree.innerHTML = "";
    outTree.style.display = "none";
    outTa.style.display = "";
    status.style.display = "none";
  };
}
