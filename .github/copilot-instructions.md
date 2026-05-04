# Qingmo — Copilot 工作指南

本文件由 GitHub Copilot 自动读取，用于指导 AI 助手在本仓库中正确工作。

---

## 一、项目概览

| 项目 | 说明 |
|------|------|
| 语言 | Rust (egui / eframe GUI) |
| 构建命令 | `cargo build --release` |
| 测试命令 | `cargo test` |
| 配置文件 | `~/.config/qingmo/config.json` |

### 主要模块

```
src/app/
  mod.rs          — 应用状态 (TextToolApp)、查找栏 (FindBar)
  models.rs       — 所有数据结构（WorldObject、StructNode、AppConfig 等）
  file_manager.rs — 文件树 (FileNode)、OpenFile、rfd 对话框封装
  search.rs       — 全文搜索、纯文本导出、字数统计 (count_words)
  sync.rs         — 数据持久化（JSON 读写、原子写入）
  ui_helpers.rs   — 查找栏、命令面板、设置窗口等辅助 UI 组件
  panel/          — 各面板 UI（novel, objects, structure, llm, outline, characters, markdown）
  llm_backend.rs  — LLM 后端抽象（Mock / API / LocalServer / Agent）
  agent.rs        — Agent 技能集 (SkillSet / AgentBackend)
issues/           — 待办 issue 列表（Markdown 文件）
```

---

## 二、Issue 工作流循环（必须严格遵守）

每次开发迭代按以下循环执行，**不得跳过任何步骤**：

```
1. 选取 issues/ 目录中优先级最高的 issue 文件
2. 实现功能 + 补充单元测试（纯逻辑函数必须有测试）
3. 删除对应 issue 文件（issueN.md）
4. 新增 1–2 个后续 issue 文件（issueN+1.md、issueN+2.md）
5. 运行 cargo test，确认全部测试通过
6. 提交 PR
```

### Issue 文件模板

```markdown
# [类型] 标题

## 描述
（问题或功能的背景说明）

## 期望行为
（完成后用户能看到/做到什么）

## 实现建议
（具体到函数/模块/结构体）

## 优先级
🔴 高 / 🟡 中 / 🟢 低

## 验收标准
- [ ] 验收项 1
- [ ] 验收项 2
```

优先级排序：🔴 高 > 🟡 中 > 🟢 低。

---

## 三、架构约定（AI 助手必须遵守）

### 单一职责
- `models.rs` 只存数据结构，`search.rs` 只做文本处理，`sync.rs` 只做 IO。
- `search.rs`、`sync.rs` 等工具模块**不得** `use` 任何 egui/GUI 类型。

### 存储路径约定（重要）
当前存储布局（已由 `Content/` 迁移完成）：

| 数据 | 路径 |
|------|------|
| 章节 Markdown | `<project>/chapters/*.md` |
| 世界观对象 | `<project>/data/world.json` |
| 故事结构 | `<project>/data/structure.json` |
| LLM 对话历史 | `<project>/llm_history.json` |
| 废稿暂存 | `<project>/废稿/` |
| 全局配置 | `~/.config/qingmo/config.json` |

> ⚠️ **禁止**在任何新代码中使用旧路径 `Content/` 或 `Design/`。

### 原子写入
所有文件写入必须通过 `sync.rs::write_atomically(path, content)` 实现，
禁止直接调用 `std::fs::write` 写入项目文件。

### 配置兼容性
`AppConfig` / `MarkdownSettings` 新增字段**必须**加 `#[serde(default)]`，
防止旧配置文件反序列化失败。

### 可测试性
- 纯逻辑函数（`count_words`、`markdown_to_plain_text`、序列化等）必须有单元测试。
- 测试使用 `std::env::temp_dir()` + 唯一子目录，测试后清理。
- `TextToolApp` 依赖 egui，**不在单元测试中构造**；逻辑提取为纯函数后测试。

### 错误处理
- 禁止 `.unwrap()` 崩溃；使用 `.unwrap_or_default()` 或 `match`。
- 文件 IO 错误通过 `self.status` 显示中文消息。

---

## 四、代码风格

- 模块顶部注释使用 `// ── 节标题 ──────` 横线风格分隔区块。
- 函数命名：动词+名词，语义清晰（如 `sync_world_objects_to_json`）。
- 新增字段附上中文 `///` doc comment，说明用途、默认值和影响范围。
- API Key **不存入** `config.json`，通过环境变量传递。
