# [Tech Debt] 查找栏性能与 UX 改进

## 描述
代码审查在 `FindBar` 及 `draw_find_bar_ui` 实现中发现以下可改进点：

## 已知问题

### 1. 大文件下的性能问题
**位置**：`src/app/mod.rs` — `FindBar::refresh_matches`

**问题**：非大小写区分模式下每次刷新都对整篇内容调用 `to_lowercase()`，对于数万字的长篇小说（每章 5–10 万字）会产生完整字符串拷贝，在用户高速输入时（每次按键触发一次刷新）可能导致帧率抖动。

**建议**：
```rust
// 缓存小写版本，仅在内容或 case_sensitive 变化时重建
pub lowercase_content_cache: Option<String>,
pub content_snapshot_len: usize, // 用于检测内容是否发生变化
```
或使用 `memchr`/`aho-corasick`（已是 egui 依赖，可通过 `Cargo.toml` 直接引用）进行高效子串搜索。

### 2. 字节→字符偏移转换的时间复杂度
**位置**：`src/app/ui_helpers.rs` — `select_current_match`

**问题**：
```rust
let start_char = content[..start_byte].chars().count();
let end_char   = content[..end_byte].chars().count();
```
每次跳转到匹配项时从字符串开头遍历到匹配位置，时间复杂度 O(n)。对于文档末尾的匹配，在 5 万字文档中需要遍历约 15 万字节。

**建议**：在 `refresh_matches` 中同时计算并缓存各匹配项的 char-offset 范围：
```rust
pub struct MatchRange {
    pub byte_start: usize,
    pub byte_end: usize,
    pub char_start: usize, // 仅在需要时填充
    pub char_end: usize,
}
```

### 3. 编辑器未自动滚动到匹配位置
**位置**：`select_current_match` 通过 `TextEditState` 更新了光标选择范围，但 egui 0.29 的 `TextEdit` 不会自动将选中区域滚动到视口内。

**当前现象**：用户按 ▼/▲ 后光标在编辑器内正确移动，但如果匹配项不在当前视口内，用户无法看到高亮效果，需要手动滚动。

**建议**：
- 检查 egui 是否提供 `scroll_to_cursor` 接口（egui 0.30+ 的 `TextEdit` 新增了此选项）
- 或在跳转时将匹配所在行号估算后手动设置 `ScrollArea` 偏移量

## 优先级
🟡 中——功能可用但大文件下体验欠佳

## 验收标准
- [ ] 在 5 万字文档中每次按键后查找响应时间 < 16ms（不产生明显卡顿）
- [ ] 按 ▼/▲ 跳转时编辑器自动滚动使匹配项可见
- [ ] 字节→字符转换复杂度降低（缓存或预计算）
