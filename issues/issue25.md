# [优化] 全文搜索结果高亮匹配词

## 描述
当前全文搜索结果列表中，每行结果以纯文本形式显示，用户无法快速定位命中位置，
尤其在长行内容中难以找到关键词出现的确切位置。

## 期望行为
- 搜索结果列表中，命中的关键词部分以**黄色高亮背景**或**加粗**方式标注
- 高亮支持大小写不敏感模式（与当前 `search_case_sensitive` 标志一致）
- 结果行过长时，显示命中词前后各 30 个字符的摘要（snippet）而非完整行

## 实现建议
- 在 `ui_helpers.rs` 的搜索结果渲染循环中，将原来的 `selectable_label` 替换为
  分段 `egui::Label` + `RichText::background_color` 高亮块
- 新增辅助函数 `make_search_snippet(line: &str, query: &str, case_sensitive: bool, context: usize) -> Vec<(String, bool)>`
  返回 `(片段文本, 是否命中)` 列表
- 该函数可纯逻辑测试，无 GUI 依赖

## 优先级
🟡 中

## 验收标准
- [ ] 搜索结果中命中词背景色与其余文字可视区分
- [ ] 行过长（>80字）时只显示命中词上下文摘要
- [ ] `make_search_snippet` 有单元测试覆盖：普通、CJK、大小写不敏感、多次命中
