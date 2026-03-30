# [Feature] 编辑器内查找与替换

## 描述
当前仅有 `Ctrl+Shift+F` 全文搜索（跨文件只读），编辑器内没有单文件范围的「查找」与「查找并替换」功能。这是文本编辑器的基础功能，缺失会严重打断改稿流程。

## 期望行为
1. **查找（Ctrl+F）**：在当前 Markdown 编辑器内弹出查找工具栏（类 VS Code 顶部浮窗），高亮匹配项，`Enter`/`Shift+Enter` 向前/向后跳转。
2. **替换（Ctrl+H）**：扩展查找栏，增加「替换为」输入框；「替换（Enter）」替换当前匹配，「全部替换」一次性替换文件内所有匹配。
3. **选项**：支持「区分大小写」和「全词匹配」两个开关；中文内容常用，优先保证 CJK 字符的正确匹配。
4. **退出**：`Esc` 关闭工具栏并清除高亮，焦点返回编辑器。

## 实现建议

### 状态字段（TextToolApp）
```rust
pub(super) find_bar: Option<FindBar>,

pub(super) struct FindBar {
    pub query: String,
    pub replace: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub match_ranges: Vec<(usize, usize)>, // byte ranges in left_content
    pub current_match: usize,
    pub replace_mode: bool,
}
```

### 查找逻辑
```rust
impl FindBar {
    /// Recompute match_ranges from the current query against `text`.
    pub fn refresh_matches(&mut self, text: &str) {
        self.match_ranges.clear();
        if self.query.is_empty() { return; }
        let haystack = if self.case_sensitive { text.to_owned() } else { text.to_lowercase() };
        let needle   = if self.case_sensitive { self.query.clone() } else { self.query.to_lowercase() };
        let mut start = 0;
        while let Some(pos) = haystack[start..].find(&needle) {
            let abs = start + pos;
            // Whole-word check: boundaries must be non-alphanumeric / non-CJK
            if !self.whole_word || is_word_boundary(text, abs, abs + needle.len()) {
                self.match_ranges.push((abs, abs + needle.len()));
            }
            start = abs + needle.len().max(1);
        }
    }
}
```

### UI（浮动工具栏，渲染于 novel.rs 或 ui_helpers.rs）
```rust
fn draw_find_bar(&mut self, ui: &mut Ui) {
    let Some(bar) = &mut self.find_bar else { return; };
    egui::Frame::none()
        .fill(Color32::from_gray(40))
        .rounding(6.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // 查找框
                let resp = ui.text_edit_singleline(&mut bar.query);
                if resp.changed() { bar.refresh_matches(&self.left_content); }
                ui.label(format!("{}/{}", bar.current_match + 1, bar.match_ranges.len()));
                if ui.small_button("▲").clicked() { bar.prev_match(); }
                if ui.small_button("▼").clicked() { bar.next_match(); }
                ui.checkbox(&mut bar.case_sensitive, "Aa");
                ui.checkbox(&mut bar.whole_word, "\\b");
                // 替换框（仅 replace_mode 下显示）
                if bar.replace_mode {
                    ui.text_edit_singleline(&mut bar.replace);
                    if ui.button("替换").clicked() { /* … */ }
                    if ui.button("全部替换").clicked() { /* … */ }
                }
                if ui.small_button("✕").clicked() { self.find_bar = None; }
            });
        });
}
```

### 高亮渲染
在 `build_inline_job` 中，若 `FindBar` 有当前匹配范围且行包含匹配字节偏移，对匹配段应用高亮 `TextFormat`（黄色背景）。

### 相关文件
- `src/app/mod.rs`：`FindBar` 结构体，状态字段 `find_bar`
- `src/app/panel/novel.rs`：绑定 `Ctrl+F` / `Ctrl+H`，渲染浮动查找栏，跳转光标
- `src/app/ui_helpers.rs`：`handle_keyboard()` 中添加 Ctrl+F / Ctrl+H 响应
- `src/app/panel/markdown.rs`：`build_inline_job` 支持高亮匹配段

## 优先级
🔴 高——改稿时「找到所有『他』替换为『她』」是高频操作，缺失严重影响效率

## 验收标准
- [ ] `Ctrl+F` 弹出查找栏，实时高亮当前文件内所有匹配，`Enter`/`Shift+Enter` 跳转
- [ ] `Ctrl+H` 在查找栏基础上展开替换框，支持「替换」和「全部替换」
- [ ] 支持「区分大小写」开关
- [ ] `Esc` 关闭查找栏并清除高亮
- [ ] 替换操作可通过 `Ctrl+Z` 撤销
