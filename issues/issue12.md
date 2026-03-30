# [Feature] LLM 对话历史持久化与会话管理

## 描述
当前 LLM 面板每次启动后对话历史清空，用户无法回顾之前的 AI 生成内容，也无法在不同写作会话间延续对同一章节的讨论。需要将对话历史持久化到磁盘，并提供会话管理界面。

## 期望行为
1. **自动持久化**：每条对话（用户 prompt + AI 回复）自动追加写入 `~/.config/qingmo/llm_history.json` 或项目根目录 `Design/llm_history.json`（优先项目内，便于随项目迁移）。
2. **会话分组**：以「项目路径 + 日期」为 key 分组，同一项目同一天的对话属于同一会话；侧边栏展示历史会话列表，点击可加载查看。
3. **搜索与过滤**：在历史面板顶部提供搜索框，对 prompt 和 response 内容进行关键词过滤。
4. **重用内容**：点击历史条目的「插入」按钮，将 AI 回复内容插入当前 Markdown 编辑器光标位置。
5. **清理**：右键历史条目可删除单条；面板底部提供「清空本项目历史」按钮（带确认对话框）。
6. **大小限制**：单个历史文件超过 2 MB 时自动归档（重命名为 `llm_history_YYYYMMDD.json`），避免无限增长。

## 实现建议

### 数据结构
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct LlmHistoryEntry {
    pub id: u64,                     // 单调递增 ID
    pub timestamp: u64,              // Unix timestamp (seconds)
    pub session_key: String,         // "project_path::YYYY-MM-DD"
    pub prompt: String,
    pub response: String,
    pub model: String,               // 使用的模型名称/路径
    pub template_name: Option<String>, // 使用的提示词模板（如有）
}

#[derive(Serialize, Deserialize, Default)]
pub struct LlmHistory {
    pub entries: Vec<LlmHistoryEntry>,
}

impl LlmHistory {
    pub fn append(&mut self, entry: LlmHistoryEntry, path: &Path) -> std::io::Result<()> {
        self.entries.push(entry);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
}
```

### LLM 面板扩展（src/app/panel/llm.rs）
```rust
// 历史面板 tab（与现有「配置」「任务」tab 并列）
"历史" tab:
    ├── 搜索框（关键词过滤）
    ├── 会话分组列表（日期 → 展开/折叠）
    │   └── 条目卡片：timestamp + prompt 前40字 + 按钮组
    │       ├── 「展开」显示完整内容
    │       ├── 「插入」将 response 插入编辑器
    │       └── 「删除」移除此条（带确认）
    └── 「清空本项目历史」按钮
```

### 完成任务时保存
```rust
// 在 LLM 任务完成回调处（poll_llm_task 或 update_llm_panel）添加：
if let Some(response) = task_done {
    let entry = LlmHistoryEntry {
        id: self.llm_history.entries.len() as u64 + 1,
        timestamp: unix_now(),
        session_key: self.session_key(),
        prompt: self.current_prompt.clone(),
        response: response.clone(),
        model: self.llm_config.model_path.clone(),
        template_name: self.selected_template_name.clone(),
    };
    let _ = self.llm_history.append(entry, &self.llm_history_path);
}
```

### 大小限制检查
```rust
pub fn maybe_archive(&self, path: &Path) {
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > 2 * 1024 * 1024 {
            let archive = path.with_file_name(
                format!("llm_history_{}.json", today_date_str())
            );
            let _ = std::fs::rename(path, &archive);
        }
    }
}
```

### 相关文件
- `src/app/models.rs`：`LlmHistoryEntry`、`LlmHistory` 结构体
- `src/app/panel/llm.rs`：历史 tab 渲染，任务完成时写入历史
- `src/app/mod.rs`：`llm_history: LlmHistory` 字段，项目打开时加载
- `~/.config/qingmo/` 或 `Design/`：`llm_history.json` 存储路径

## 优先级
🟡 中——AI 辅助写作的核心体验之一；对长期使用者价值随时间累积增大

## 验收标准
- [ ] 每次 LLM 任务完成后自动将 prompt+response 持久化到历史文件
- [ ] LLM 面板新增「历史」tab，按日期分组展示历史条目
- [ ] 历史可按关键词过滤搜索
- [ ] 「插入」按钮将 AI 回复插入当前编辑器光标位置
- [ ] 历史文件超过 2 MB 时自动归档
- [ ] 不引入新的 crate 依赖（`serde_json` 已存在）
