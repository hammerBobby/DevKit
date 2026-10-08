import { invoke } from "../api.js";
import { showMsg, bindCopy, copyText } from "../ui.js";

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function initSqlView() {
  const view = document.getElementById("sql-view");
  view.innerHTML = `
    <div class="view-header">
      <div>
        <div class="view-title">SQL 参数替换</div>
        <div class="view-subtitle">将 SQL 模板中的 ? 占位符按参数列表替换为完整 SQL，INSERT 自动生成 UPDATE</div>
      </div>
    </div>
    <div class="editor-area">
      <div class="toolbar">
        <button class="btn btn-primary" id="sql-run">替换占位符</button>
        <button class="btn" id="sql-sample">示例</button>
        <span style="flex:1"></span>
        <button class="btn btn-danger" id="sql-clear">清空</button>
      </div>
      <div class="status-msg" id="sql-status"></div>

      <!-- 第一行：SQL 模板 + 表结构 DDL（6:4） -->
      <div class="sql-row1">
        <div class="pane">
          <div class="pane-header">
            <span>SQL 模板 + 参数</span>
            <span style="font-weight:400;color:var(--color-text-muted)">格式：SQL, 执行参数：1(String), 张三(String)</span>
          </div>
          <textarea id="sql-input" class="code" spellcheck="false" wrap="soft" placeholder='示例：
SELECT * FROM user WHERE id = ? AND name = ?, 执行参数：1001(Long), 张三(String)

INSERT INTO orders (id, amount, remark) VALUES (?, ?, ?), 执行参数：2001(Long), 99.99(BigDecimal), 已下单(String)'></textarea>
        </div>
        <div class="pane">
          <div class="pane-header">
            <span>表结构 DDL（可选）</span>
            <span style="font-weight:400;color:var(--color-text-muted)">VARCHAR 超长校验</span>
          </div>
          <textarea id="sql-ddl" class="code" spellcheck="false" wrap="soft" placeholder="CREATE TABLE user (
  id varchar(20) NOT NULL,
  name varchar(50),
  remark varchar(100),
  age int
);"></textarea>
        </div>
      </div>

      <!-- 第二行：替换后的 SQL（无 UPDATE 时单栏，有 UPDATE 时双栏） -->
      <div class="sql-row2 single" id="sql-row2">
        <div class="pane">
          <div class="pane-header">
            <span>替换后的 SQL</span>
            <div class="right">
              <button class="mini-btn" id="sql-copy">复制</button>
            </div>
          </div>
          <textarea id="sql-output" class="code" spellcheck="false" wrap="soft" readonly></textarea>
        </div>
        <div class="pane" id="sql-update-pane" style="display:none">
          <div class="pane-header">
            <span>自动生成的 UPDATE</span>
            <div class="right">
              <button class="mini-btn" id="sql-copy-update">复制</button>
            </div>
          </div>
          <textarea id="sql-update-output" class="code" spellcheck="false" wrap="soft" readonly></textarea>
        </div>
      </div>

      <!-- 第三行：仅展示超长字段 -->
      <div class="pane" id="sql-over-pane" style="display:none">
        <div class="pane-header">
          <span id="sql-over-title">超长字段列表</span>
          <span style="font-weight:400;color:var(--color-text-muted)" id="sql-over-summary"></span>
          <div class="right">
            <button class="mini-btn" id="sql-copy-over">复制提示</button>
          </div>
        </div>
        <div id="sql-over-list" class="sql-overflow-list"></div>
      </div>
    </div>
  `;

  const input = view.querySelector("#sql-input");
  const ddlInput = view.querySelector("#sql-ddl");
  const output = view.querySelector("#sql-output");
  const updateOutput = view.querySelector("#sql-update-output");
  const updatePane = view.querySelector("#sql-update-pane");
  const row2 = view.querySelector("#sql-row2");
  const overPane = view.querySelector("#sql-over-pane");
  const overTitle = view.querySelector("#sql-over-title");
  const overSummary = view.querySelector("#sql-over-summary");
  const overList = view.querySelector("#sql-over-list");
  const status = view.querySelector("#sql-status");

  function renderOutput(sql, updateSql) {
    output.value = sql;
    if (updateSql) {
      updateOutput.value = updateSql;
      updatePane.style.display = "";
      row2.classList.remove("single");
    } else {
      updateOutput.value = "";
      updatePane.style.display = "none";
      row2.classList.add("single");
    }
  }

  function renderOverChecks(checks) {
    const over = (checks || []).filter((c) => c.status === "over");
    if (over.length === 0) {
      overPane.style.display = "none";
      overList.innerHTML = "";
      return;
    }
    overList.innerHTML = over
      .map((c) => {
        const al = c.actual_len == null ? "-" : c.actual_len;
        const ml = c.max_len == null ? "-" : c.max_len;
        const dbType = c.db_type || "-";
        return `<div class="sql-over-item">
          <span class="sql-over-mark">❗</span>
          <span class="sql-over-field">${escapeHtml(c.field)}</span>
          <span class="sql-over-type">${escapeHtml(dbType)}</span>
          <span class="sql-over-len">${al} / ${ml}</span>
        </div>`;
      })
      .join("");
    overTitle.textContent = `超长字段列表（${over.length} 个）`;
    overSummary.textContent = `请修正这些字段的值后再执行`;
    overPane.style.display = "";
  }

  function buildOverText(checks) {
    const over = (checks || []).filter((c) => c.status === "over");
    return over
      .map(
        (c) =>
          `字段「${c.field}」值长度 ${c.actual_len} > 限制 ${c.max_len}（字段类型 ${c.db_type}）`
      )
      .join("\n");
  }

  view.querySelector("#sql-run").onclick = async () => {
    const text = input.value.trim();
    if (!text) {
      showMsg(status, "请输入 SQL 模板", "warning");
      return;
    }
    try {
      const ddl = ddlInput.value.trim();
      const r = await invoke("sql_replace", {
        sqlTemplate: text,
        tableStructure: ddl || null,
      });

      renderOutput(r.sql, r.update_sql);
      renderOverChecks(r.field_checks);

      // 自动复制：无 UPDATE 只复制 SQL；有 UPDATE 则 SQL + UPDATE 一起复制
      let copied = false;
      try {
        const copyPayload = r.update_sql ? r.sql + "\n\n" + r.update_sql : r.sql;
        await copyText(copyPayload);
        copied = true;
      } catch {
        copied = false;
      }
      const copyHint = copied ? " · 已自动复制到剪贴板" : "";

      const overText = buildOverText(r.field_checks);

      // 状态提示：超长优先
      if (overText) {
        showMsg(
          status,
          `⚠️ 字段长度校验失败（${overText.split("\n").length} 个超长）\n` +
            overText.split("\n").map((e) => "• " + e).join("\n") +
            copyHint,
          "error"
        );
        return;
      }

      const msgs = [];
      if (r.table_name) msgs.push(`表名：${r.table_name}`);
      if (r.update_sql) msgs.push("已生成 UPDATE 语句");

      if (r.warnings.length) {
        showMsg(status, "替换成功" + copyHint + "\n" + r.warnings.join("\n"), "warning");
        return;
      }
      showMsg(status, "替换成功" + (msgs.length ? " · " + msgs.join(" · ") : "") + copyHint, "success");
    } catch (e) {
      showMsg(status, e, "error");
    }
  };

  bindCopy(view.querySelector("#sql-copy"), () => output.value);
  bindCopy(view.querySelector("#sql-copy-update"), () => updateOutput.value);
  bindCopy(view.querySelector("#sql-copy-over"), () => {
    const items = overList.querySelectorAll(".sql-over-item");
    if (!items.length) return "";
    return Array.from(items)
      .map((el) => {
        const spans = el.querySelectorAll("span");
        return `${spans[1]?.textContent ?? ""} ${spans[3]?.textContent ?? ""}`.trim();
      })
      .filter(Boolean)
      .join("\n");
  });

  view.querySelector("#sql-clear").onclick = () => {
    input.value = "";
    ddlInput.value = "";
    output.value = "";
    updateOutput.value = "";
    updatePane.style.display = "none";
    row2.classList.add("single");
    overPane.style.display = "none";
    overList.innerHTML = "";
    status.style.display = "none";
  };

  view.querySelector("#sql-sample").onclick = () => {
    input.value = `SELECT * FROM orders WHERE status = ? AND amount > ?, 执行参数：1(Integer), 100(BigDecimal)

UPDATE user SET name = ?, age = ? WHERE id = ?, 执行参数：李四(String), 30(Integer), 1001(Long)

INSERT INTO user (id, name, remark) VALUES (?, ?, ?), 执行参数：155699555319834624011(String), 张三(String), 这是一个非常非常非常非常非常非常长的备注内容超过一百个字符用于演示超长字段校验效果的信息内容(String)`;
    ddlInput.value = `CREATE TABLE user (
  id varchar(20) NOT NULL,
  name varchar(50),
  remark varchar(100),
  age int,
  price numeric(10,2)
);`;
  };
}