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

**已完成 Issues**：issue1–issue20

完成摘要：
- issue17：命令面板（`Ctrl+Shift+P`，模糊匹配，键盘导航，5 个单元测试）
- issue18：全书字数统计（总字数 + 今日新增 + 日目标进度条 + 各章字数折叠面板，4 个单元测试）
- issue19：崩溃恢复（`write_atomically` 原子写入 + 项目打开时扫描 `.swp` 文件并弹出恢复对话框）
- issue20：LLM 历史归档（`archive_old_entries` 按年归档 + 设置窗口显示历史条数/归档文件数/手动归档按钮）

**当前待处理 Issues**：issue21（场景时间线）, issue22（正则全文搜索）, issue23（写作会话统计）

---

## 五、数据存储设计

> 设计原则：**业务驱动存储、单一职责、可扩展优先、数据安全第一、读写分离思想**。

### 5.1 Qingmo 的数据全貌

| 数据类型 | 描述 | 数量级 | 访问模式 | 一致性要求 | 生命周期 |
|---------|------|--------|---------|-----------|---------|
| 小说内容 | Markdown 文本文件 | 每章 1–100 KB | 高频读写（编辑器实时） | 强一致 | 长期热数据 |
| 世界观对象 | JSON 文件（人物/地点/道具等） | 单项目几十 KB | 读多写少 | 强一致 | 长期热数据 |
| 故事结构 | JSON 嵌套树（StructNode） | < 1 MB | 读多写少 | 强一致 | 长期热数据 |
| 用户配置 | `config.json`（AppConfig） | < 10 KB | 启动读/设置写 | 强一致 | 永久保留 |
| LLM 对话历史 | JSON 数组（LlmHistoryEntry） | 数百条/项目 | 追加写、列表读 | 最终一致 | 热数据→冷归档 |
| 废稿文件 | 软删除的 Markdown 文件 | 少量 | 极低频 | 无要求 | 温/冷数据 |
| 全文搜索索引 | 当前：内存扫描；未来：倒排索引 | 与小说体量一致 | 只读查询 | 最终一致 | 重建即可 |

### 5.2 存储分层架构

```
┌─────────────────────────────────────────────────────────┐
│  内存层（热路径）                                          │
│  TextToolApp 字段：当前打开文件内容、FindBar 缓存、         │
│  LLM 历史条目列表、世界观对象 Vec                          │
├─────────────────────────────────────────────────────────┤
│  本地文件层（持久化，按职责分文件）                          │
│  ├── <project>/chapters/*.md      小说章节（主体内容）     │
│  ├── <project>/world/*.json       世界观对象              │
│  ├── <project>/structure.json     故事结构树               │
│  ├── <project>/llm_history.json   LLM 对话记录            │
│  ├── <project>/废稿/              软删除暂存区             │
│  └── ~/.config/qingmo/config.json 全局用户配置            │
├─────────────────────────────────────────────────────────┤
│  归档层（冷数据，未来扩展）                                  │
│  ├── 旧版本 llm_history 可按年份压缩存档                   │
│  └── 废稿目录超限时可 zip 归档后清理                        │
└─────────────────────────────────────────────────────────┘
```

### 5.3 存储选型决策（对应本项目）

| 数据特征 | 本项目选择 | 理由 |
|---------|-----------|------|
| 小说正文（纯文本、强一致） | **本地 Markdown 文件** | 人类可读、版本控制友好、无依赖 |
| 世界观/结构（半结构化、灵活 schema） | **本地 JSON 文件**（类 MongoDB 思路） | 嵌套结构自然映射、`serde` 序列化零依赖 |
| LLM 历史（追加写、读时过滤） | **单 JSON 文件 + 内存索引** | 数据量小，无需数据库；`next_id` 计数器保证唯一性 |
| 用户配置（KV、启动加载） | **单 JSON 文件**（类 Redis KV 思路） | `#[serde(default)]` 保证向前兼容 |
| 全文搜索（目前） | **内存扫描**（`search_dir` 按行读取） | 项目规模下已够用；未来可升级为内置倒排索引 |
| 图片/封面等大文件 | **原始文件路径引用**（不嵌入 JSON） | 避免 base64 膨胀，保持 JSON 轻量 |

> **反模式已规避**：
> - ✅ 不将大文本嵌入 JSON（图片存路径引用）
> - ✅ 单文件按职责拆分，不用一个 `data.json` 扛所有
> - ✅ `废稿/` 软删除防止数据丢失（代替硬删除）
> - ✅ `AppConfig` 字段全部带 `#[serde(default)]`，防止旧配置文件读取崩溃

### 5.4 数据安全与容灾

| 风险 | 当前措施 | 未来增强建议 |
|-----|---------|------------|
| 编辑中崩溃丢失内容 | `auto_save_interval_secs` 定期保存 | 增加 crash recovery 草稿文件（`.swp` 机制） |
| 误删章节 | 软删除到 `废稿/` 目录 | 废稿目录超 30 天自动 zip 归档 |
| 配置损坏 | `.unwrap_or_default()` 降级加载 | 保存前先写 `.bak` 备份 |
| API Key 泄露 | 不存入 `config.json`，走环境变量 | 支持系统 Keychain 集成 |
| LLM 历史 ID 冲突 | `next_id` 单调递增计数器 | 已修复（issue12） |

### 5.5 性能优化策略

| 场景 | 优化手段 | 实现位置 |
|-----|---------|---------|
| 查找栏高频刷新 | `cached_lower` 小写缓存 + `MatchRange` 预计算 char offset | `mod.rs` FindBar + `ui_helpers.rs` |
| 字数统计 | 仅在 `resp.changed()` 时重算，非每帧 | `panel/novel.rs` |
| 世界观加载 | 项目打开时一次性加载进内存 Vec | `sync.rs` |
| 全文搜索 | 按文件逐行读取，命中即返回（非全量加载） | `search.rs::search_dir` |
| JSON 序列化 | 写入前先序列化到 String，失败时不覆盖文件 | `sync.rs` 写文件封装 |

### 5.6 可扩展性预留

当项目规模增长到需要升级存储时，迁移路径如下：

```
当前：本地 JSON 文件
  ↓（章节数 > 500 或搜索变慢）
阶段二：内置 SQLite（via rusqlite），统一管理元数据 + 倒排索引
  ↓（多端同步需求）
阶段三：可选云同步（WebDAV / S3 兼容对象存储），本地文件为主、云端为备份
```

**设计原则**：文件格式保持人类可读（Markdown + JSON），任何阶段都可以直接用文本编辑器访问数据，**不绑定专有数据库格式**。
