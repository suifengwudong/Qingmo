# [功能] 写作会话记录与统计（Session Journal）

## 描述
作者希望追踪每次写作会话的生产力数据：写了多少字、持续多长时间、集中在哪些章节。这些数据可以帮助建立写作习惯，也可用于生成"写作日报"。

## 期望行为
- 每次打开项目时开始一个新会话，记录：开始时间、结束时间、各章节新增字数
- 会话结束（项目关闭/应用退出）时将本次会话数据追加到 `<project>/data/sessions.json`
- 在设置窗口或统计面板显示：
  - 今日累计写作时长
  - 近 7 天每日字数折线（使用 egui Plot 或简易字符图）
  - 历史最高单日字数
- `sessions.json` 格式与 `llm_history.json` 类似（JSON 数组，每条含 `start_ts`、`end_ts`、`words_added: HashMap<String,i64>`）

## 实现建议
- `models.rs` 新增 `SessionEntry` 结构体和 `SessionJournal`（类比 `LlmHistory`）
- `TextToolApp` 增加 `session_start: Instant`、`session_journal_path: Option<PathBuf>` 字段
- 退出前通过 `eframe::App::on_exit` 回调或 `save_config` 调用保存会话
- 统计面板可作为设置窗口的一个新 Tab，或作为底部可折叠区域

## 优先级
🟢 低

## 验收标准
- [ ] 打开项目写入 100 字后关闭，`sessions.json` 存在且包含正确字数
- [ ] 再次打开项目后，新会话另起一条记录，不覆盖旧数据
- [ ] 设置面板显示"今日写作时长"（精确到分钟）
- [ ] `SessionJournal::append` 有单元测试（使用 temp_dir）
