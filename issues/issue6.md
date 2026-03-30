# [Feature] 导出增强：纯文本（.txt）导出与 PDF 打印

## 描述
当前「导出章节合集」仅支持导出为单一 Markdown 文件。许多读者/编辑需要纯文本或 PDF 格式，应在现有基础上补充这两种导出路径。

## 期望行为
1. **纯文本导出（`.txt`）**：去除 Markdown 语法标记（`#`、`**`、`*`、`` ` `` 等），输出干净的纯文字内容，换行段落保留。
2. **PDF 导出**：调用操作系统打印对话框，将 Markdown 渲染为 HTML 后交由 OS 打印管道（Windows：`ShellExecute("print", ...)`；Linux/macOS：`lp` 命令），不引入 PDF 渲染库。
3. **UI 入口**：「文件」菜单中将「导出章节合集」拆分为子菜单：「导出为 Markdown」/ 「导出为纯文本」/ 「打印/导出 PDF」。

## 实现建议

### 纯文本转换（新增自由函数）
```rust
/// 将 Markdown 内容转换为纯文本，去除标记符号
pub fn markdown_to_plain_text(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    for line in md.lines() {
        // 去除 ATX 标题前缀 (#, ##, …)
        let line = line.trim_start_matches('#').trim_start();
        // 去除粗体/斜体标记
        let line = line.replace("**", "").replace('*', "").replace('`', "");
        // 去除引用 >
        let line = line.trim_start_matches('>').trim_start();
        out.push_str(line);
        out.push('\n');
    }
    out
}
```

### 导出流程（src/app/mod.rs）
```rust
// 纯文本导出
fn export_plain_text(&mut self) {
    if let Some(save_path) = rfd::FileDialog::new()
        .add_filter("纯文本", &["txt"])
        .save_file()
    {
        let combined = self.collect_chapters_content(); // 复用已有逻辑
        let plain = markdown_to_plain_text(&combined);
        if let Err(e) = std::fs::write(&save_path, plain) {
            self.status = format!("导出失败: {e}");
        } else {
            self.status = format!("已导出纯文本: {}", save_path.display());
        }
    }
}
```

### 相关文件
- `src/app/mod.rs`：菜单栏「文件」菜单，`export_plain_text()` 方法，`export_pdf()` 方法
- `src/app/file_manager.rs`：可封装 `markdown_to_plain_text` 工具函数

## 优先级
🟢 低——锦上添花，核心写作流程不依赖此功能

## 验收标准
- [ ] 「文件」菜单新增「导出为纯文本（.txt）」选项，输出干净无标记的文字
- [ ] PDF 导出调用系统打印对话框（至少 Windows 平台可用）
- [ ] 导出失败时状态栏显示错误信息
- [ ] 未新增任何 crate 依赖（PDF 通过 OS 打印管道实现）
