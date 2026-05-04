# [优化] 全文搜索支持大小写不敏感模式

## 描述
当前 `search.rs::search_dir` 使用 `line.contains(query)` 进行精确匹配，区分大小写。
对于中文作者常见的混合大小写英文关键词（如人名、地名），无法找到所有出现位置。
而编辑器内 `FindBar` 已实现了大小写不敏感缓存（`cached_lower`），全文搜索与之存在不一致。

## 期望行为
- 全文搜索面板提供"区分大小写"勾选框（默认关闭）
- 不区分大小写时，搜索 "alice" 也能命中 "Alice"、"ALICE"
- 搜索结果显示的原始行内容不变（仍显示原始大小写）

## 实现建议
- `search_dir` 签名增加 `case_sensitive: bool` 参数
- 不区分大小写时使用 `line.to_lowercase().contains(&query.to_lowercase())`
- `TextToolApp` 增加 `search_case_sensitive: bool` 字段（默认 `false`）
- 在搜索面板 UI 中（`panel/` 或 `ui_helpers.rs`）增加对应勾选框
- `search_dir` 的现有单元测试需覆盖两种模式

## 优先级
🟡 中

## 验收标准
- [ ] 默认（不区分大小写）时，"alice" 能匹配 "Alice"、"ALICE"
- [ ] 勾选"区分大小写"后，"alice" 不匹配 "Alice"
- [ ] `search_dir` 有单元测试覆盖大小写两种模式
- [ ] 搜索结果仍显示原始未修改的行内容
