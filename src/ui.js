export function showMsg(el, msg, type = "info") {
  el.className = "status-msg " + type;
  el.textContent = typeof msg === "string" ? msg : String(msg.message || msg);
  el.style.display = "block";
}

export async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    // 降级
    const ta = document.createElement("textarea");
    ta.value = text;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    ta.remove();
  }
}

// 复制 + 按钮反馈
export function bindCopy(btn, getText) {
  btn.addEventListener("click", async () => {
    const text = getText();
    if (!text || !text.trim()) return;
    await copyText(text);
    btn.textContent = "✓ 已复制";
    btn.classList.add("copied");
    setTimeout(() => {
      btn.textContent = "复制";
      btn.classList.remove("copied");
    }, 1500);
  });
}
