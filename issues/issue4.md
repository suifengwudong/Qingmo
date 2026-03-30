# [Feature] 编辑器字体大小独立配置及 Ctrl+滚轮缩放

## 描述
目前 Markdown 编辑器（等宽字体）和预览区字号共用同一个设置，无法独立调整。用户在编辑时希望使用更大的字号，但又不想改变预览排版。

## 期望行为
1. **独立字号**：编辑器字号（`editor_font_size`）与预览字号（`preview_font_size`）互相独立，分别在设置中配置。
2. **Ctrl+滚轮快速缩放**：在编辑区滚动时按住 `Ctrl`，字号随滚轮上下调整（每格 ±1pt），松开 Ctrl 恢复普通滚动。
3. **范围限制**：字号范围限定在 8–36pt，防止异常值。
4. **持久化**：字号写入 `MarkdownSettings`，下次启动自动恢复。

## 实现建议

### 数据模型
在 `src/app/models.rs` 的 `MarkdownSettings` 中已有 `editor_font_size` 字段，确认其已被 `panel/novel.rs` 编辑器使用（当前编辑器可能固定使用 `13.0` 或 `preview_font_size`）。

```rust
// MarkdownSettings 中（已有，需核对）：
#[serde(default = "default_editor_font_size")]
pub editor_font_size: f32,
fn default_editor_font_size() -> f32 { 13.0 }
```

### 编辑器渲染应用字号
在 `src/app/panel/novel.rs` 的编辑器 `TextEdit` 构建处替换硬编码字号：

```rust
egui::TextEdit::multiline(&mut content)
    .font(egui::FontId::monospace(self.md_settings.editor_font_size))
    // ...
```

### Ctrl+滚轮缩放
在编辑器所在的 `ui.allocate_rect` 或 `response` 后添加：

```rust
if response.hovered() {
    let scroll_delta = ui.input(|i| {
        if i.modifiers.ctrl { i.smooth_scroll_delta.y } else { 0.0 }
    });
    if scroll_delta != 0.0 {
        self.md_settings.editor_font_size =
            (self.md_settings.editor_font_size + scroll_delta * 0.1).clamp(8.0, 36.0);
        self.config_dirty = true;
    }
}
```

### 设置 UI
在设置弹窗的 Markdown 区块中添加：

```rust
ui.label("编辑器字号");
ui.add(egui::Slider::new(&mut self.md_settings.editor_font_size, 8.0..=36.0)
    .text("pt").step_by(1.0));
```

### 相关文件
- `src/app/models.rs`：`MarkdownSettings.editor_font_size` 字段（已存在，确认默认值）
- `src/app/panel/novel.rs`：编辑器 `TextEdit` 字号引用；Ctrl+滚轮处理
- `src/app/mod.rs`：设置弹窗中添加编辑器字号滑动条

## 优先级
🟡 中——长时间写作时字体舒适度直接影响效率

## 验收标准
- [ ] 设置中编辑器字号与预览字号互相独立
- [ ] Ctrl+滚轮可在编辑区实时缩放字号，范围 8–36pt
- [ ] 字号偏好持久化，重启后自动恢复
- [ ] 普通滚动（无 Ctrl）不受影响
