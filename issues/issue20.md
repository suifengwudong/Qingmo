# [优化] LLM 历史归档与冷存储

## 描述
随着使用时间增长，`llm_history.json` 会持续膨胀。根据数据存储分层设计，超过一定量的旧记录应归档到冷存储，保持热数据文件轻量。

## 期望行为
- 当 `llm_history.json` 条目超过 **500 条**（可配置），自动将 90 天前的条目归档到 `llm_history_archive_<YYYY>.json`
- 主 `llm_history.json` 只保留最近 N 条（默认 200）
- 归档文件只读，不参与 UI 展示（除非用户手动导入）
- 在设置界面显示"历史条数 / 归档文件数"

## 实现建议
- 在 `models.rs` 的 `LlmHistory` 实现中新增 `archive_old_entries(project_dir: &Path, max_hot: usize, cold_days: u64)` 方法
- 归档文件名按年份分组：`llm_history_archive_2025.json`
- 触发时机：`llm_history.save()` 时自动检查条目数

## 数据存储分层对应
- **热数据**：最近 200 条 → `llm_history.json`
- **温/冷数据**：90 天前 → `llm_history_archive_<year>.json`

## 优先级
🟢 低

## 验收标准
- [ ] 插入 501 条记录后，调用归档方法，主文件降至 ≤200 条
- [ ] 归档文件包含被移出的旧条目，且格式与主文件相同
- [ ] `archive_old_entries` 有单元测试（使用 temp_dir）
