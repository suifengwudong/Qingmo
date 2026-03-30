# [Tech Debt] 代码健壮性改进：替换潜在风险 unwrap() 并统一错误处理

## 描述
代码库中存在约 189 处 `unwrap()` 调用，多数位于上下文安全的场景，但部分位于生产路径且依赖隐式不变量，一旦不变量被破坏将直接 `panic`（进程崩溃）。本 issue 跟踪需要改进的高风险点。

## 已知风险点

### 1. `src/app/agent.rs` — LLM Skill 写文件路径处理 ✅ 已修复
**原始代码（WriteFileContentSkill, ~line 750）**：
```rust
std::fs::create_dir_all(canonical_file.parent().unwrap())
```
`canonical_file` 由 `canonicalize` 或 `canonical_parent.join(...)` 构建，理论上必有父目录，但 `unwrap()` 使代码意图不明确，且与周围安全代码风格不一致。

**已修改为**：
```rust
let parent_dir = canonical_file.parent()
    .ok_or_else(|| "文件路径无效，无法解析父目录".to_owned())?;
std::fs::create_dir_all(parent_dir)
    .map_err(|e| format!("创建目录失败: {e}"))?;
```

### 2. `src/app/panel/markdown.rs` — 行内渲染 `render_inline_text`
代码中多处对字符串切片使用字节索引（如 `&text[start..end]`），若切割点恰好在多字节 UTF-8 字符中间，将在运行时 `panic`。

**风险场景**：LLM 生成内容含特殊 Unicode 字符（如 emoji、特殊标点）时，索引计算可能错位。

**建议**：使用 `get(start..end)` 替代直接索引，并 fallback 到全文渲染：
```rust
// 替换 &text[start..end]
let Some(slice) = text.get(start..end) else { return; };
```

### 3. `src/app/sync.rs` — 模板创建部分失败时 UI 未同步 ✅ 已修复
`apply_template_short` / `apply_template_long` 在检测到部分失败后提前返回，但**已成功写入的文件的 UI（文件树、结构树）不会同步**，导致磁盘上存在文件而界面不感知。

**已修改**：将 `sync_struct_from_folders()` + `refresh_tree()` 移到检查之前，并更新错误消息以明确提示残留文件：
```rust
self.sync_struct_from_folders();
self.refresh_tree();
if !errors.is_empty() {
    self.status = format!("模板创建部分失败（已成功创建的文件保留在磁盘）: {}", errors.join("; "));
} else {
    self.status = "已创建短篇模板（单层章节结构）".to_owned();
}
```

> **待办（可选增强）**：如需完全原子化，可在失败时清理已创建文件（rollback），但需权衡实现复杂度。

### 4. `src/app/panel/novel.rs` / `src/app/mod.rs` — 测试代码 `unwrap()`
测试中大量使用 `serde_json::from_str(...).unwrap()` / `std::fs::write(...).unwrap()`，在受限文件系统（CI 沙箱、只读目录）下可能导致测试崩溃而非输出有意义的失败信息。

**建议**：改用 `expect("描述性消息")` 以便调试：
```rust
// 替换
serde_json::from_str(&json).unwrap()
// 为
serde_json::from_str(&json).expect("测试用 JSON 应可正常反序列化")
```

## 不需要修改的安全 `unwrap()`
以下是经过验证安全的 `unwrap()` 调用，不需修改：
- `outline.rs:125` — `new_path.last_mut().unwrap()`（`path.last()` 已经在外层匹配为 `Some`）
- `outline.rs:506, 513` — 均有 `path.is_empty()` 前置守卫
- `markdown.rs:150` — `trimmed.chars().next().unwrap()`（`trim()` + `len() >= 3` 保证非空）
- `agent.rs:376` 等 `path.parent().unwrap()`（路径由 `root.join("Design").join(filename)` 构建，必有父目录）

## 优先级
🟡 中——不影响当前功能，但影响代码可维护性和在极端输入下的稳健性

## 验收标准
- [x] `WriteFileContentSkill` 的目录创建改用 `.ok_or_else(...)?` 模式（已修复）
- [x] 模板创建失败时状态栏消息明确提示残留文件情况，且 UI 同步已创建的文件（已修复）
- [ ] `render_inline_text` 中所有字节索引改用 `.get(start..end)`，对无效索引 gracefully fallback
- [ ] 测试代码中的 `unwrap()` 替换为携带描述的 `expect(...)`
