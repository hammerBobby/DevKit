use std::array;

use gpui_kit::{
    component::{
        button::{Button, ButtonVariants},
        input::{Editor, EditorState, Textarea, TextareaState},
        label::Label,
        list::ListItem,
        status_bar::StatusBar,
        tab::{Tab, TabBar},
        tree::{self, TreeItem, TreeState},
        ActiveTheme, Selectable, TitleBar, h_flex, v_flex,
    },
    *,
};

use crate::commands::{crypto, json_tool, jwt, sql_tool};

const JSON_SAMPLE: &str = r#"{"name":"DevKit","version":"0.3.0","tools":["JSON","JWT","加密","SQL"],"ok":true}"#;
const JWT_SAMPLE: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkRldktpdCIsImlhdCI6MTcwMDAwMDAwMCwiZXhwIjoxOTAwMDAwMDAwfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
const SQL_SAMPLE: &str = "INSERT INTO user (id, name, age, phone) VALUES (?, ?, ?, ?), 执行参数：1001(String), 张三(String), 28(Integer), 13800138000(String)";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Json,
    Jwt,
    Crypto,
    Sql,
}

struct Editors {
    json_input: Entity<TextareaState>,
    json_output: Entity<EditorState>,
    jwt_input: Entity<TextareaState>,
    jwt_output: Entity<TextareaState>,
    crypto_inputs: [Entity<TextareaState>; 3],
    crypto_outputs: [Entity<TextareaState>; 3],
    sql_template: Entity<TextareaState>,
    sql_ddl: Entity<TextareaState>,
    sql_output: Entity<TextareaState>,
    json_tree: Entity<TreeState>,
}

struct DevKit {
    tool: Tool,
    status: String,
    json_tree_view: bool,
    crypto_tab: usize,
    jwt_primary: String,
    sql_primary: String,
    editors: Editors,
}

impl DevKit {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editors = Editors {
            json_input: new_textarea(JSON_SAMPLE, "输入 JSON", window, cx),
            json_output: new_json_editor(window, cx),
            jwt_input: new_textarea("", "输入 JWT Token", window, cx),
            jwt_output: new_textarea("", "解析结果", window, cx),
            crypto_inputs: array::from_fn(|_| new_textarea("", "输入内容", window, cx)),
            crypto_outputs: array::from_fn(|_| new_textarea("", "处理结果", window, cx)),
            sql_template: new_textarea("", "输入 SQL 模板与参数", window, cx),
            sql_ddl: new_textarea("", "可选：CREATE TABLE 语句", window, cx),
            sql_output: new_textarea("", "替换结果", window, cx),
            json_tree: cx.new(|cx| TreeState::new(cx)),
        };

        Self {
            tool: Tool::Json,
            status: "就绪 · 输入内容后点击按钮处理".to_string(),
            json_tree_view: false,
            crypto_tab: 0,
            jwt_primary: String::new(),
            sql_primary: String::new(),
            editors,
        }
    }

    fn set_status(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.status = text.into();
        cx.notify();
    }

    fn read(editor: &Entity<TextareaState>, cx: &App) -> String {
        editor.read(cx).value().to_string()
    }

    fn write(editor: &Entity<TextareaState>, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        editor.update(cx, |state, cx| state.set_value(value.to_string(), window, cx));
    }

    fn read_editor(editor: &Entity<EditorState>, cx: &App) -> String {
        editor.read(cx).value().to_string()
    }

    fn write_editor(editor: &Entity<EditorState>, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        editor.update(cx, |state, cx| state.set_value(value.to_string(), window, cx));
    }

    fn set_output(&self, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        Self::write_editor(&self.editors.json_output, value, window, cx);
    }

    fn auto_copy(&mut self, value: &str, cx: &mut Context<Self>) {
        if value.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        if !self.status.ends_with(" · 已自动复制") {
            self.status.push_str(" · 已自动复制");
        }
        cx.notify();
    }

    fn refresh_json_tree(&self, value: &str, cx: &mut Context<Self>) {
        let items = match serde_json::from_str::<serde_json::Value>(value) {
            Ok(root) => json_tree_items(&root, "$", 0),
            Err(_) => Vec::new(),
        };
        self.editors
            .json_tree
            .update(cx, |state, cx| state.set_items(items, cx));
    }

    fn json_format_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = Self::read(&self.editors.json_input, cx);
        match json_tool::json_format(input) {
            Ok(value) => {
                self.set_output(&value, window, cx);
                self.refresh_json_tree(&value, cx);
                self.set_status("JSON 已格式化", cx);
                self.auto_copy(&value, cx);
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    fn json_compact_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = Self::read(&self.editors.json_input, cx);
        match json_tool::json_compact(input) {
            Ok(value) => {
                self.set_output(&value, window, cx);
                self.refresh_json_tree(&value, cx);
                self.set_status("JSON 已压缩", cx);
                self.auto_copy(&value, cx);
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    fn json_escape_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = Self::read(&self.editors.json_input, cx);
        let value = json_tool::json_escape(input).unwrap_or_else(|error| error);
        self.set_output(&value, window, cx);
        self.set_status("JSON 已转义", cx);
        self.auto_copy(&value, cx);
    }

    fn json_unescape_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = Self::read(&self.editors.json_input, cx);
        match json_tool::json_unescape(input) {
            Ok(value) => {
                self.set_output(&value, window, cx);
                self.refresh_json_tree(&value, cx);
                self.set_status("JSON 已去转义", cx);
                self.auto_copy(&value, cx);
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    fn json_tree_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.json_tree_view {
            let input = Self::read(&self.editors.json_input, cx);
            match json_tool::json_format(input) {
                Ok(value) => {
                    self.set_output(&value, window, cx);
                    self.refresh_json_tree(&value, cx);
                }
                Err(error) => {
                    self.set_status(format!("请先格式化有效 JSON：{error}"), cx);
                    return;
                }
            }
        }
        self.json_tree_view = !self.json_tree_view;
        self.set_status(
            if self.json_tree_view {
                "树形视图 · 点击节点展开"
            } else {
                "文本视图"
            },
            cx,
        );
    }

    fn jwt_decode_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = Self::read(&self.editors.jwt_input, cx);
        match jwt::jwt_decode(input) {
            Ok(result) => {
                let value = format_jwt(&result);
                Self::write(&self.editors.jwt_output, &value, window, cx);
                self.jwt_primary = value.clone();
                self.set_status(
                    if result.valid {
                        "JWT 解析成功 · 时效有效"
                    } else {
                        "JWT 解析成功 · 已过期或未生效"
                    },
                    cx,
                );
                self.auto_copy(&value, cx);
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    fn crypto_action(&mut self, encode: bool, window: &mut Window, cx: &mut Context<Self>) {
        let index = self.crypto_tab;
        if index >= 3 {
            return;
        }
        let input = Self::read(&self.editors.crypto_inputs[index], cx);
        let result = match (index, encode) {
            (0, true) => crypto::base64_encode(input),
            (0, false) => crypto::base64_decode(input),
            (1, true) => crypto::url_encode(input),
            (1, false) => crypto::url_decode(input),
            (2, true) => crypto::md5_encode(input),
            (2, false) => crypto::md5_encode_upper(input),
            _ => return,
        };
        match result {
            Ok(value) => {
                Self::write(&self.editors.crypto_outputs[index], &value, window, cx);
                self.set_status("处理完成", cx);
                self.auto_copy(&value, cx);
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    fn crypto_swap_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let index = self.crypto_tab;
        if index >= 3 {
            return;
        }
        let output = Self::read(&self.editors.crypto_outputs[index], cx);
        let input = Self::read(&self.editors.crypto_inputs[index], cx);
        Self::write(&self.editors.crypto_inputs[index], &output, window, cx);
        Self::write(&self.editors.crypto_outputs[index], &input, window, cx);
        self.set_status("已将结果回填到输入框", cx);
        let swapped = Self::read(&self.editors.crypto_inputs[index], cx);
        self.auto_copy(&swapped, cx);
    }

    fn sql_replace_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let template = Self::read(&self.editors.sql_template, cx);
        let ddl = Self::read(&self.editors.sql_ddl, cx);
        let structure = if ddl.trim().is_empty() { None } else { Some(ddl) };
        match sql_tool::sql_replace(template, structure) {
            Ok(result) => {
                let value = format_sql(&result);
                Self::write(&self.editors.sql_output, &value, window, cx);
                self.sql_primary = result.sql.clone();
                self.set_status("SQL 参数已替换", cx);
                self.auto_copy(&result.sql, cx);
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    fn copy_action(&mut self, value: &str, cx: &mut Context<Self>) {
        if value.is_empty() {
            self.set_status("没有可复制的内容", cx);
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
        self.set_status(format!("已复制 {} 个字符", value.chars().count()), cx);
    }

    fn sidebar(&self, cx: &Context<Self>) -> impl IntoElement {
        let item = |id: &'static str, label: &'static str, tool: Tool, cx: &Context<Self>| {
            Button::new(id)
                .label(label)
                .w_full()
                .selected(self.tool == tool)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.tool = tool;
                    this.set_status(format!("已切换到 {}", tool_name(tool)), cx);
                }))
        };

        v_flex()
            .w(px(136.))
            .flex_shrink_0()
            .h_full()
            .gap(px(4.))
            .p(px(10.))
            .bg(cx.theme().sidebar)
            .border_r_1()
            .border_color(cx.theme().border)
            .child(
                Label::new("工具")
                    .text_size(px(12.))
                    .text_color(cx.theme().muted_foreground)
                    .px(px(8.))
                    .pb(px(6.)),
            )
            .child(item("nav-json", "JSON", Tool::Json, cx))
            .child(item("nav-jwt", "JWT", Tool::Jwt, cx))
            .child(item("nav-crypto", "加密", Tool::Crypto, cx))
            .child(item("nav-sql", "SQL", Tool::Sql, cx))
            .child(div().flex_1())
            .child(
                Label::new("本地计算 · 不上传")
                    .text_size(px(11.))
                    .text_color(cx.theme().muted_foreground)
                    .px(px(8.)),
            )
    }

    fn body(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.tool {
            Tool::Json => self.json_body(cx),
            Tool::Jwt => self.jwt_body(cx),
            Tool::Crypto => self.crypto_body(cx),
            Tool::Sql => self.sql_body(cx),
        }
    }

    fn json_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let output: AnyElement = if self.json_tree_view {
            panel_frame("JSON 树", cx)
                .child(
                    tree::tree(&self.editors.json_tree, |_, entry, _, _, _| {
                        ListItem::new(entry.item().id.clone())
                            .child(Label::new(entry.item().label.clone()).id(entry.item().id.clone()))
                    })
                    .flex_1()
                    .min_h(px(160.))
                    .h_full(),
                )
                .into_any_element()
        } else {
            code_panel("输出", &self.editors.json_output, true, cx).into_any_element()
        };

        v_flex()
            .flex_1()
            .min_h(px(0.))
            .gap(px(10.))
            .child(
                h_flex()
                    .gap(px(6.))
                    .flex_wrap()
                    .child(action_button(
                        "json-format",
                        "格式化",
                        true,
                        cx.listener(|this, _, window, cx| this.json_format_action(window, cx)),
                    ))
                    .child(action_button(
                        "json-compact",
                        "压缩",
                        false,
                        cx.listener(|this, _, window, cx| this.json_compact_action(window, cx)),
                    ))
                    .child(action_button(
                        "json-escape",
                        "转义",
                        false,
                        cx.listener(|this, _, window, cx| this.json_escape_action(window, cx)),
                    ))
                    .child(action_button(
                        "json-unescape",
                        "去转义",
                        false,
                        cx.listener(|this, _, window, cx| this.json_unescape_action(window, cx)),
                    ))
                    .child(action_button(
                        "json-tree",
                        if self.json_tree_view { "树形 ✓" } else { "树形" },
                        false,
                        cx.listener(|this, _, window, cx| this.json_tree_action(window, cx)),
                    ))
                    .child(action_button(
                        "json-copy",
                        "复制结果",
                        false,
                        cx.listener(|this, _, _, cx| {
                            let value = Self::read_editor(&this.editors.json_output, cx);
                            this.copy_action(&value, cx);
                        }),
                    ))
                    .child(action_button(
                        "json-sample",
                        "示例",
                        false,
                        cx.listener(|this, _, window, cx| {
                            Self::write(&this.editors.json_input, JSON_SAMPLE, window, cx);
                            Self::write_editor(&this.editors.json_output, "", window, cx);
                            this.json_tree_view = false;
                            this.set_status("已载入 JSON 示例", cx);
                        }),
                    ))
                    .child(action_button(
                        "json-clear",
                        "清空",
                        false,
                        cx.listener(|this, _, window, cx| {
                            Self::write(&this.editors.json_input, "", window, cx);
                            Self::write_editor(&this.editors.json_output, "", window, cx);
                            this.json_tree_view = false;
                            this.set_status("已清空 JSON", cx);
                        }),
                    )),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .items_stretch()
                    .gap(px(10.))
                    .child(editor_panel(
                        "输入",
                        &self.editors.json_input,
                        false,
                        cx,
                    ))
                    .child(output),
            )
            .into_any_element()
    }

    fn jwt_body(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .flex_1()
            .min_h(px(0.))
            .gap(px(10.))
            .child(
                h_flex()
                    .gap(px(6.))
                    .child(action_button(
                        "jwt-decode",
                        "解析 Token",
                        true,
                        cx.listener(|this, _, window, cx| this.jwt_decode_action(window, cx)),
                    ))
                    .child(action_button(
                        "jwt-copy",
                        "复制结果",
                        false,
                        cx.listener(|this, _, _, cx| {
                            this.copy_action(&this.jwt_primary.clone(), cx);
                        }),
                    ))
                    .child(action_button(
                        "jwt-sample",
                        "示例",
                        false,
                        cx.listener(|this, _, window, cx| {
                            Self::write(&this.editors.jwt_input, JWT_SAMPLE, window, cx);
                            Self::write(&this.editors.jwt_output, "", window, cx);
                            this.jwt_primary.clear();
                            this.set_status("已载入 JWT 示例", cx);
                        }),
                    ))
                    .child(action_button(
                        "jwt-clear",
                        "清空",
                        false,
                        cx.listener(|this, _, window, cx| {
                            Self::write(&this.editors.jwt_input, "", window, cx);
                            Self::write(&this.editors.jwt_output, "", window, cx);
                            this.jwt_primary.clear();
                            this.set_status("已清空 JWT", cx);
                        }),
                    )),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .items_stretch()
                    .gap(px(10.))
                    .child(editor_panel(
                        "Token 输入",
                        &self.editors.jwt_input,
                        false,
                        cx,
                    ))
                    .child(editor_panel(
                        "解析结果",
                        &self.editors.jwt_output,
                        true,
                        cx,
                    )),
            )
            .into_any_element()
    }

    fn crypto_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let index = self.crypto_tab;
        let tabs = TabBar::new("crypto-tabs")
            .segmented()
            .selected_index(index)
            .children([
                Tab::new().label("Base64"),
                Tab::new().label("URL 编解码"),
                Tab::new().label("MD5"),
                Tab::new().label("说明"),
            ])
            .on_click(cx.listener(|this, index: &usize, _, cx| {
                this.crypto_tab = *index;
                if *index < 3 {
                    this.set_status(format!("当前：{}", crypto_tab_name(*index)), cx);
                }
            }));

        let content: AnyElement = if index == 3 {
            v_flex()
                .gap(px(8.))
                .p(px(16.))
                .child(Label::new("Base64：编码 / 解码 UTF-8 文本"))
                .child(Label::new("URL：percent 编码 / 解码"))
                .child(Label::new("MD5：生成 32 位小写摘要，点击“大写 MD5”切换"))
                .child(Label::new("上下互换：把结果回填到输入框，便于连续处理"))
                .child(Label::new("所有计算均在本地完成，不上传任何数据"))
                .into_any_element()
        } else {
            h_flex()
                .flex_1()
                .min_h(px(0.))
                .items_stretch()
                .gap(px(10.))
                .child(editor_panel(
                    "输入",
                    &self.editors.crypto_inputs[index],
                    false,
                    cx,
                ))
                .child(editor_panel(
                    "输出",
                    &self.editors.crypto_outputs[index],
                    true,
                    cx,
                ))
                .into_any_element()
        };

        v_flex()
            .flex_1()
            .min_h(px(0.))
            .gap(px(10.))
            .child(tabs)
            .child(
                h_flex()
                    .gap(px(6.))
                    .child(action_button(
                        "crypto-encode",
                        "编码 / 生成",
                        true,
                        cx.listener(|this, _, window, cx| this.crypto_action(true, window, cx)),
                    ))
                    .child(action_button(
                        "crypto-decode",
                        if index == 2 { "大写 MD5" } else { "解码" },
                        false,
                        cx.listener(|this, _, window, cx| this.crypto_action(false, window, cx)),
                    ))
                    .child(action_button(
                        "crypto-swap",
                        "上下互换",
                        false,
                        cx.listener(|this, _, window, cx| this.crypto_swap_action(window, cx)),
                    ))
                    .child(action_button(
                        "crypto-copy",
                        "复制结果",
                        false,
                        cx.listener(|this, _, _, cx| {
                            if this.crypto_tab < 3 {
                                let value = Self::read(
                                    &this.editors.crypto_outputs[this.crypto_tab],
                                    cx,
                                );
                                this.copy_action(&value, cx);
                            }
                        }),
                    ))
                    .child(action_button(
                        "crypto-clear",
                        "清空",
                        false,
                        cx.listener(|this, _, window, cx| {
                            if this.crypto_tab < 3 {
                                let index = this.crypto_tab;
                                Self::write(&this.editors.crypto_inputs[index], "", window, cx);
                                Self::write(&this.editors.crypto_outputs[index], "", window, cx);
                                this.set_status("已清空当前页", cx);
                            }
                        }),
                    )),
            )
            .child(content)
            .into_any_element()
    }

    fn sql_body(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .flex_1()
            .min_h(px(0.))
            .gap(px(10.))
            .child(
                h_flex()
                    .gap(px(6.))
                    .child(action_button(
                        "sql-replace",
                        "替换参数",
                        true,
                        cx.listener(|this, _, window, cx| this.sql_replace_action(window, cx)),
                    ))
                    .child(action_button(
                        "sql-copy",
                        "复制 SQL",
                        false,
                        cx.listener(|this, _, _, cx| {
                            this.copy_action(&this.sql_primary.clone(), cx);
                        }),
                    ))
                    .child(action_button(
                        "sql-sample",
                        "示例",
                        false,
                        cx.listener(|this, _, window, cx| {
                            Self::write(&this.editors.sql_template, SQL_SAMPLE, window, cx);
                            Self::write(&this.editors.sql_ddl, "", window, cx);
                            Self::write(&this.editors.sql_output, "", window, cx);
                            this.sql_primary.clear();
                            this.set_status("已载入 SQL 示例，点击替换参数查看结果", cx);
                        }),
                    ))
                    .child(action_button(
                        "sql-clear",
                        "清空",
                        false,
                        cx.listener(|this, _, window, cx| {
                            Self::write(&this.editors.sql_template, "", window, cx);
                            Self::write(&this.editors.sql_ddl, "", window, cx);
                            Self::write(&this.editors.sql_output, "", window, cx);
                            this.sql_primary.clear();
                            this.set_status("已清空 SQL", cx);
                        }),
                    )),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .items_stretch()
                    .gap(px(10.))
                    .child(
                        panel_frame("SQL 模板 / 执行参数", cx)
                            .child(
                                Textarea::new(&self.editors.sql_template)
                                    .readonly(false)
                                    .w_full()
                                    .flex_1()
                                    .min_h(px(0.))
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_size(cx.theme().mono_font_size),
                            )
                            .child(
                                Label::new("表结构 DDL（可选）")
                                    .text_size(px(12.))
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                Textarea::new(&self.editors.sql_ddl)
                                    .h(px(90.))
                                    .w_full()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_size(cx.theme().mono_font_size),
                            ),
                    )
                    .child(editor_panel(
                        "替换结果",
                        &self.editors.sql_output,
                        true,
                        cx,
                    )),
            )
            .into_any_element()
    }
}

impl Render for DevKit {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                TitleBar::new().child(
                    Label::new("DevKit · 开发工具箱")
                        .text_size(px(13.))
                        .text_color(cx.theme().foreground),
                ),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .items_stretch()
                    .child(self.sidebar(cx))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w(px(0.))
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap(px(10.))
                                    .px(px(14.))
                                    .pt(px(10.))
                                    .pb(px(4.))
                                    .child(
                                        Label::new(tool_name(self.tool))
                                            .text_size(px(16.))
                                            .text_color(cx.theme().foreground),
                                    )
                                    .child(
                                        Label::new(tool_hint(self.tool))
                                            .text_size(px(12.))
                                            .text_color(cx.theme().muted_foreground),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_h(px(0.))
                                    .p(px(12.))
                                    .child(self.body(cx)),
                            ),
                    ),
            )
            .child(
                StatusBar::new()
                    .left(Label::new(format!("✓ {}", self.status)))
                    .right(Label::new("v0.3.0 · GPUI Kit")),
            )
    }
}

pub fn run() {
    gpui_kit::component::set_locale("zh-CN");

    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
        gpui_kit::init(cx);

        let window_closed = cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        });
        std::mem::forget(window_closed);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(60.), px(40.)),
                size: size(px(1200.), px(820.)),
            })),
            window_min_size: Some(size(px(900.), px(600.))),
            titlebar: Some(TitlebarOptions {
                title: Some("DevKit".into()),
                ..TitleBar::title_bar_options()
            }),
            ..TitleBar::window_options()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| DevKit::new(window, cx))
        })
        .expect("failed to open DevKit window");
    });
}

fn new_json_editor(window: &mut Window, cx: &mut App) -> Entity<EditorState> {
    cx.new(|cx| {
        EditorState::new(window, cx)
            .language("json")
            .line_number(false)
            .soft_wrap(true)
    })
}

fn new_textarea(
    value: &str,
    placeholder: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TextareaState> {
    cx.new(|cx| {
        let mut state = TextareaState::new(window, cx)
            .rows(10)
            .placeholder(placeholder)
            .soft_wrap(true);
        state.set_value(value.to_string(), window, cx);
        state
    })
}

fn action_button(
    id: &'static str,
    label: &'static str,
    primary: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Button {
    let button = Button::new(id).label(label);
    let button = if primary { button.primary() } else { button };
    button.on_click(on_click)
}

fn panel_frame(title: &'static str, cx: &App) -> Div {
    v_flex()
        .flex_1()
        .min_h(px(0.))
        .gap(px(6.))
        .p(px(10.))
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
        .rounded(px(8.))
        .child(
            Label::new(title)
                .text_size(px(12.))
                .text_color(cx.theme().muted_foreground),
        )
}

fn editor_panel(
    title: &'static str,
    editor: &Entity<TextareaState>,
    readonly: bool,
    cx: &App,
) -> impl IntoElement {
    panel_frame(title, cx).child(
        Textarea::new(editor)
            .readonly(readonly)
            .w_full()
            .flex_1()
            .min_h(px(0.))
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size),
    )
}

fn code_panel(
    title: &'static str,
    editor: &Entity<EditorState>,
    readonly: bool,
    cx: &App,
) -> impl IntoElement {
    panel_frame(title, cx).child(
        Editor::new(editor)
            .readonly(readonly)
            .w_full()
            .flex_1()
            .min_h(px(0.))
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size),
    )
}

fn tool_name(tool: Tool) -> &'static str {
    match tool {
        Tool::Json => "JSON 工具",
        Tool::Jwt => "JWT 解析",
        Tool::Crypto => "加密编码",
        Tool::Sql => "SQL 替换",
    }
}

fn tool_hint(tool: Tool) -> &'static str {
    match tool {
        Tool::Json => "格式化 / 压缩 / 转义 / 树形浏览",
        Tool::Jwt => "支持 Bearer、Basic 前缀，仅解析不验签",
        Tool::Crypto => "Base64 / URL / MD5 本地计算",
        Tool::Sql => "按类型替换 ? 并生成 UPDATE",
    }
}

fn crypto_tab_name(index: usize) -> &'static str {
    match index {
        0 => "Base64",
        1 => "URL 编解码",
        2 => "MD5",
        _ => "说明",
    }
}

fn format_jwt(result: &jwt::JwtResult) -> String {
    let pretty = |value: &serde_json::Value| {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    };
    let time = |value: Option<i64>| value.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
    let mut lines = vec![
        format!(
            "有效状态：{}",
            if result.valid { "有效" } else { "无效/已过期" }
        ),
        format!("过期时间 exp：{}", time(result.exp)),
        format!("生效时间 nbf：{}", time(result.not_before)),
        format!("签发时间 iat：{}", time(result.issued_at)),
        format!("签名：{}", result.signature),
    ];
    if let Some(error) = &result.error {
        lines.push(format!("错误：{error}"));
    }
    lines.push("--- Header ---".to_string());
    lines.push(pretty(&result.header));
    lines.push("--- Payload ---".to_string());
    lines.push(pretty(&result.payload));
    lines.join("\n")
}

fn format_sql(result: &sql_tool::SqlResult) -> String {
    let mut lines = vec![format!("SQL:\n{}", result.sql)];
    if let Some(update) = &result.update_sql {
        lines.push(format!("\n自动生成的 UPDATE:\n{update}"));
    }
    if let Some(table) = &result.table_name {
        lines.push(format!("\n表名：{table}"));
    }
    if !result.warnings.is_empty() {
        lines.push("\n警告：".to_string());
        lines.extend(result.warnings.iter().map(|w| format!("- {w}")));
    }
    if !result.errors.is_empty() {
        lines.push("\n错误：".to_string());
        lines.extend(result.errors.iter().map(|e| format!("- {e}")));
    }
    if !result.field_checks.is_empty() {
        lines.push("\n字段检查：".to_string());
        for check in &result.field_checks {
            lines.push(format!(
                "- {} ({}) 最大 {} / 实际 {} → {}",
                check.field,
                check.db_type,
                check.max_len.map(|v| v.to_string()).unwrap_or_else(|| "?".into()),
                check.actual_len.map(|v| v.to_string()).unwrap_or_else(|| "?".into()),
                check.status
            ));
        }
    }
    lines.join("\n")
}

fn json_tree_items(value: &serde_json::Value, path: &str, depth: usize) -> Vec<TreeItem> {
    const MAX_DEPTH: usize = 8;
    const MAX_CHILDREN: usize = 64;
    if depth >= MAX_DEPTH {
        return Vec::new();
    }
    match value {
        serde_json::Value::Object(map) => map
            .iter()
            .take(MAX_CHILDREN)
            .map(|(key, child)| {
                let child_path = format!("{path}.{key}");
                TreeItem::new(path_id(&child_path), format!("{}: {}", key, brief(child)))
                    .expanded(depth == 0)
                    .children(json_tree_items(child, &child_path, depth + 1))
            })
            .collect(),
        serde_json::Value::Array(items) => items
            .iter()
            .take(MAX_CHILDREN)
            .enumerate()
            .map(|(index, child)| {
                let child_path = format!("{path}[{index}]");
                TreeItem::new(path_id(&child_path), format!("[{index}] {}", brief(child)))
                    .expanded(depth == 0)
                    .children(json_tree_items(child, &child_path, depth + 1))
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn brief(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::String(v) => format!("\"{v}\""),
        serde_json::Value::Object(v) => format!("{{{} 项}}", v.len()),
        serde_json::Value::Array(v) => format!("[{} 项]", v.len()),
    }
}

fn path_id(path: &str) -> SharedString {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in path.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:x}").into()
}
