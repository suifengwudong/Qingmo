# [Feature] 命令面板（Ctrl+Shift+P）

## 描述
参考 VS Code 的命令面板，提供一个模糊搜索驱动的操作入口，让用户无需记忆菜单位置即可快速执行任意功能：打开文件、切换面板、应用 LLM 模板、切换主题、导出章节等。

## 期望行为
1. **触发**：`Ctrl+Shift+P` 打开命令面板浮窗，覆盖在主界面上方中央。
2. **模糊搜索**：输入关键词（支持拼音首字母或中文），实时过滤命令列表，高亮匹配字符。
3. **执行**：`↑`/`↓` 选择，`Enter` 执行，`Esc` 关闭。
4. **命令集**（初始版，可扩展）：

| 命令 | 操作 |
|------|------|
| 打开项目文件夹 | 触发 `rfd::FileDialog` |
| 新建文件 | 在当前目录创建 `.md` |
| 保存当前文件 | `Ctrl+S` 等效 |
| 切换主题（亮色/暗色）| 切换 `AppTheme` |
| 切换预览模式 | `Ctrl+P` 等效 |
| 导出章节合集 | `export_chapters_merged()` |
| 导出为纯文本 | `export_plain_text()` |
| 全文搜索 | `Ctrl+Shift+F` 等效 |
| 应用模板（短篇）| `apply_template_short()` |
| 应用模板（长篇）| `apply_template_long()` |
| 打开设置 | 显示设置弹窗 |
| 备份项目 | `backup_project()` |

## 实现建议

### 注册表
```rust
pub struct Command {
    pub name: &'static str,           // 显示名（中文）
    pub keywords: &'static [&'static str], // 额外关键词（拼音首字母、英文别名）
    pub action: fn(&mut TextToolApp),
}

pub const COMMANDS: &[Command] = &[
    Command { name: "打开项目文件夹", keywords: &["open", "dkxm"], action: |app| app.open_project() },
    Command { name: "切换主题", keywords: &["theme", "qt", "dark", "light"], action: |app| app.toggle_theme() },
    // … 其他命令
];
```

### 状态字段
```rust
pub(super) show_command_palette: bool,
pub(super) command_palette_query: String,
pub(super) command_palette_selection: usize,
```

### UI 渲染（ui_helpers.rs）
```rust
pub(super) fn draw_command_palette(&mut self, ctx: &Context) {
    if !self.show_command_palette { return; }

    // 模糊过滤
    let query_lower = self.command_palette_query.to_lowercase();
    let filtered: Vec<&Command> = COMMANDS.iter()
        .filter(|cmd| {
            cmd.name.contains(&self.command_palette_query)
                || cmd.keywords.iter().any(|k| k.contains(&query_lower))
        })
        .collect();

    // 键盘导航
    ctx.input(|i| {
        if i.key_pressed(egui::Key::ArrowDown) {
            self.command_palette_selection =
                (self.command_palette_selection + 1).min(filtered.len().saturating_sub(1));
        }
        if i.key_pressed(egui::Key::ArrowUp) && self.command_palette_selection > 0 {
            self.command_palette_selection -= 1;
        }
        if i.key_pressed(egui::Key::Escape) {
            self.show_command_palette = false;
        }
    });

    egui::Window::new("命令面板")
        .anchor(egui::Align2::CENTER_TOP, [0.0, 60.0])
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .min_width(440.0)
        .show(ctx, |ui| {
            let resp = ui.text_edit_singleline(&mut self.command_palette_query);
            resp.request_focus();
            ui.separator();
            for (i, cmd) in filtered.iter().enumerate() {
                let selected = i == self.command_palette_selection;
                if ui.selectable_label(selected, cmd.name).clicked()
                    || (selected && ctx.input(|x| x.key_pressed(egui::Key::Enter)))
                {
                    (cmd.action)(self);
                    self.show_command_palette = false;
                }
            }
        });
}
```

### 相关文件
- `src/app/mod.rs`：`COMMANDS` 常量数组，`Command` 结构体，状态字段
- `src/app/ui_helpers.rs`：`draw_command_palette()` 渲染，`handle_keyboard()` 中添加 `Ctrl+Shift+P`
- `src/app/panel/*.rs`：各面板操作函数暴露为 `pub(super) fn` 供命令注册

## 优先级
🟡 中——显著降低操作摩擦，对功能持续增多后尤为重要；实现轻量，不依赖外部库

## 验收标准
- [ ] `Ctrl+Shift+P` 弹出命令面板
- [ ] 输入关键词可模糊过滤命令列表，结果实时更新
- [ ] `↑`/`↓` 键盘导航，`Enter` 执行，`Esc` 关闭
- [ ] 初始支持 README 中所列 12 条命令
- [ ] 命令注册表设计为可扩展（新命令只需追加数组项）
- [ ] 不引入新的 crate 依赖
