import { initJsonView } from "./views/json.js";
import { initJwtView } from "./views/jwt.js";
import { initCryptoView } from "./views/crypto.js";
import { initSqlView } from "./views/sql.js";

// 导航映射
const navItems = document.querySelectorAll(".nav-item");
const views = {
  json: document.getElementById("json-view"),
  jwt: document.getElementById("jwt-view"),
  crypto: document.getElementById("crypto-view"),
  sql: document.getElementById("sql-view"),
};

function switchView(name) {
  navItems.forEach((item) => item.classList.toggle("active", item.dataset.view === name));
  Object.entries(views).forEach(([key, el]) => {
    el.classList.toggle("active", key === name);
  });
}

navItems.forEach((item) => {
  item.addEventListener("click", () => switchView(item.dataset.view));
});

// 初始化各视图
initJsonView();
initJwtView();
initCryptoView();
initSqlView();

// 默认显示 JSON
switchView("json");
