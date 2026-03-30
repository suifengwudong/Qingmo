# Qingmo 项目架构与开发指南

本文件面向 AI 助手与人类开发者，记录项目的模块设计原则、架构约定与开发规范。

---

## 一、项目概览

| 项目 | 说明 |
|------|------|
| 语言 | Rust (egui / eframe GUI) |
| 配置文件 | `~/.config/qingmo/config.json`（`AppConfig` 结构体序列化） |
| 测试命令 | `cargo test` |
| 构建命令 | `cargo build --release` |

### 主要模块

```
src/app/
  mod.rs          — 应用状态 (TextToolApp)、测试模块
  models.rs       — 所有数据结构（WorldObject、StructNode、AppConfig 等）
  file_manager.rs — 文件树 (FileNode)、OpenFile、rfd 对话框封装
  search.rs       — 全文搜索、纯文本导出、字数统计 (count_words)
  sync.rs         — 数据持久化（JSON 读写、Markdown 同步）
  ui_helpers.rs   — 查找栏、设置窗口等辅助 UI 组件
  panel/          — 各面板 UI（novel, objects, structure, llm）
  llm_backend.rs  — LLM 后端抽象（Mock / API / LocalServer / Agent）
  agent.rs        — Agent 技能集 (SkillSet / AgentBackend)
issues/           — 待办 issue 列表（Markdown 文件）
```

---

## 二、模块设计原则（适用于本项目）

### 1. 单一职责 (SRP)
- 每个 `.rs` 文件只承担一类职责：`models.rs` 只存数据结构，`search.rs` 只做文本处理，`sync.rs` 只做 IO。
- 新功能先判断归属，不要向已经较大的 `mod.rs` 继续追加无关逻辑；宜新建子模块。

### 2. 高内聚、低耦合
- 面板模块（`panel/`）通过 `TextToolApp` 上的公开方法与核心状态交互，**不直接操作其他面板的字段**。
- `search.rs`、`sync.rs` 等工具模块不应 `use` 任何 GUI/egui 类型，保持纯逻辑层。

### 3. 模块化与可复用（DRY）
- 文本处理工具函数（`markdown_to_plain_text`、`count_words`）放在 `search.rs`，声明为 `pub(in crate::app)` 以便所有面板复用。
- 公共 UI 辅助函数（进度条、标签构建等）提取到 `ui_helpers.rs`，避免面板代码重复。

### 4. 接口清晰、稳定
- 对外（面板层）只暴露高层方法（如 `sync_world_objects_to_json`），隐藏底层文件路径细节。
- `AppConfig` / `MarkdownSettings` 结构体字段新增时**必须**使用 `#[serde(default)]`，确保旧配置文件向前兼容。

### 5. 可扩展性（开闭原则）
- 新 LLM 后端通过实现 `LlmBackend` trait 注入，不修改现有后端代码。
- 新面板通过在 `Panel` 枚举中添加变体并在 `panel/` 下新建文件实现，不破坏现有面板。

### 6. 可测试性
- **纯逻辑函数**（`count_words`、`markdown_to_plain_text`、`node_at`、序列化等）必须有对应单元测试。
- 测试使用 `std::env::temp_dir()` + 唯一后缀目录，测试结束后删除，避免污染。
- `TextToolApp` 依赖 egui 上下文，**不在单元测试中构造**；相关逻辑提取为纯函数后再测试。

### 7. 健壮性与容错
- 文件 IO 操作统一通过 `write_project_file` / `read_project_file` 封装，在 `self.status` 中显示中文错误信息。
- 所有 `serde_json::from_str` 结果用 `.unwrap_or_default()` 或 `match` 处理，禁止 `.unwrap()` 崩溃。
- 删除文件前先移入 `废稿/` 目录（软删除），防止误操作数据丢失。

### 8. 性能与资源
- 大文件查找：`FindBar::refresh_matches` 在非大小写区分模式下缓存小写内容（见 issue13），避免每帧重复 `to_lowercase()`。
- 字数统计 `count_words` 调用时机：仅在编辑器内容变化时（`resp.changed()`）重新计算，而非每帧。

### 9. 可读性与可维护性
- 模块顶部注释使用 `// ── 节标题 ──────` 横线风格分隔区块，保持一致。
- 函数命名：动词+名词，中文语义命名方法（如 `sync_world_objects_to_json`、`move_to_trash`）。
- 新增字段附上中文 doc comment（`///`），说明用途、默认值和影响范围。

### 10. 安全与隐私
- 配置文件 `~/.config/qingmo/config.json` 中 **不存储 API Key**；API Key 应通过环境变量传递。
- 不在 `status` 状态栏或日志中打印文件全路径的用户隐私部分（如家目录用户名）。

### 11. 适配业务演进
- Issue 文件（`issues/issueN.md`）是功能迭代的唯一来源：每轮开发「完成→删旧→增新」。
- 新 issue 遵循模板：`# [类型] 标题 / 描述 / 期望行为 / 实现建议 / 优先级 / 验收标准`。
- 优先级：🔴 高 > 🟡 中 > 🟢 低。

---

## 三、配置文件说明

**路径**：`~/.config/qingmo/config.json`

**对应结构体**：`src/app/models.rs` → `AppConfig`

**示例**：
```json
{
  "llm_config": {
    "model_path": "",
    "api_url": "http://localhost:11434/api/generate",
    "temperature": 0.7,
    "max_tokens": 512,
    "use_local": true,
    "system_prompt": ""
  },
  "md_settings": {
    "preview_font_size": 14.0,
    "default_to_preview": false,
    "hide_json": true,
    "tab_size": 2,
    "auto_extract_structure": false,
    "editor_font_size": 13.0,
    "auto_save_interval_secs": 60,
    "show_files_tab": false,
    "daily_word_goal": 1000
  },
  "last_project": "/home/user/my_novel",
  "auto_load": true,
  "theme": "Dark"
}
```

**字段新增规则**：
1. 在 `MarkdownSettings` 或 `AppConfig` 中添加字段。
2. 为字段添加 `#[serde(default = "fn_name")]` 属性，并实现对应的默认值函数。
3. 在 `MarkdownSettings::default()` 实现中补充字段。
4. 在设置窗口（`ui_helpers.rs` → `draw_settings_window`）中添加对应控件。
5. 在 `AGENT.md` 的示例 JSON 中更新字段说明。

---

## 四、Issue 工作流

```
1. 选取当前 issues/ 目录中优先级最高的 issue
2. 实现功能 + 补充单元测试
3. 删除对应 issue 文件
4. 新增 1-2 个后续 issue 文件（issueN+1.md）
5. 运行 cargo test 确认全部通过
6. 提交 PR
```

**已完成 Issues**：issue1–issue9, issue10（字数统计）, issue14（纯文本导出修复）

**当前待处理 Issues**：issue11（命令面板）, issue12（LLM 历史持久化）, issue13（查找栏性能）
