# [Bug / Tech Debt] 纯文本导出质量：Markdown 语法清除不完整

## 描述
`markdown_to_plain_text`（`src/app/search.rs`）在处理以下 Markdown 语法时输出不干净：

## 已知遗漏/问题

### 1. 有序列表与无序列表前缀残留
```markdown
- 第一项
* 第二项
+ 第三项
1. 有序一
10. 有序十
```
当前代码仅删除 `*`（单个星号），`-` 和 `+` 及 `1.` 数字前缀**不会被移除**，导出的纯文本包含 `- 第一项`、`1. 有序一` 等前缀。

**建议**：
```rust
// 去除无序列表前缀 (- / * / +)
let line = if let Some(rest) = line.strip_prefix("- ")
    .or_else(|| line.strip_prefix("* "))
    .or_else(|| line.strip_prefix("+ "))
{ rest } else { line };
// 去除有序列表前缀 (1. / 10. 等)
let line = if let Some(pos) = line.find(". ") {
    let num = &line[..pos];
    if num.chars().all(|c| c.is_ascii_digit()) && !num.is_empty() {
        &line[pos + 2..]
    } else { line }
} else { line };
```

### 2. Setext 风格标题下划线行残留
```markdown
章节标题
========
小节标题
--------
```
Setext 风格的 `===` 和 `---` 分隔行会直接输出到纯文本中，不美观。

**建议**：检测仅由 `=` 或 `-` 组成的行（长度 ≥ 3）并跳过输出。

### 3. HTML 注释与原始 HTML 标签
部分 Markdown 文档中含有 `<!-- 注释 -->` 或 `<br>` 等 HTML 标签，当前不做任何处理。

**建议**：用正则或简单的状态机跳过 `<...>` 标签和 `<!-- ... -->` 注释（可用简单字节扫描，无需正则 crate）。

### 4. 链接语法 `[text](url)` 仅保留 text
当前代码不处理链接语法，输出 `[链接文字](https://example.com)`。

**建议**：将 `[text](url)` 替换为 `text`（保留可读部分）。

## 相关文件
- `src/app/search.rs`：`markdown_to_plain_text` 函数
- `src/app/search.rs`：`tests::test_markdown_to_plain_text_*` 测试

## 优先级
🟢 低——现有功能可用，仅在导出质量上有瑕疵；需先修复列表前缀（最常见场景）

## 验收标准
- [ ] 有序/无序列表前缀在纯文本中不出现
- [ ] Setext 标题下划线行不出现在输出中
- [ ] 链接语法 `[text](url)` 输出为 `text`
- [ ] 对应单元测试覆盖以上所有场景
