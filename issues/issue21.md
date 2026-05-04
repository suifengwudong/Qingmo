# [修复] export_chapters_merged 仍使用旧目录路径 Content/

## 描述
`search.rs` 中的 `export_chapters_merged` 函数在存储布局从 `Content/` 迁移到 `chapters/`
之后未同步更新，导致"导出章节合集"功能无法找到任何 Markdown 章节文件，导出结果为空。

## 期望行为
- 点击"导出章节合集…"时，正确读取 `<project>/chapters/*.md` 中的所有章节
- 导出结果与 `export_plain_text`（已正确使用 `chapters/`）保持一致

## 根因
`export_chapters_merged` 第一行使用 `root.join("Content")`，应改为 `root.join("chapters")`。

## 实现建议
- 在 `search.rs::export_chapters_merged` 中，将 `root.join("Content")` 改为 `root.join("chapters")`

## 优先级
🔴 高（功能完全失效）

## 验收标准
- [x] `export_chapters_merged` 读取 `chapters/` 目录
- [x] 与 `export_plain_text` 行为对齐，均使用 `chapters/`
