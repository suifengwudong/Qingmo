# [功能] 全书字数统计面板（Full-Book Stats）

## 描述
在界面中增加一个"全书统计"视图，显示整个项目所有章节的字数汇总、每章字数、日目标完成进度、写作趋势。

## 期望行为
- 侧边栏或底部状态栏显示**总字数**（所有 `.md` 章节合计）
- 统计面板（可展开/折叠）显示：
  - 每章字数列表（章节名 + 字数 + 占比进度条）
  - 今日新增字数（基于 LLM 历史 session 或文件 mtime）
  - 日目标完成百分比（`daily_word_goal` 字段已存在于 `MarkdownSettings`）
- 字数统计使用现有 `count_words()` 函数（已支持中英文混合）

## 实现建议
- 在 `search.rs` 中新增 `count_words_in_dir(dir: &Path) -> Vec<(String, usize)>` 函数
- `TextToolApp` 增加 `book_stats: Option<BookStats>` 缓存字段（项目打开时计算一次，保存时更新）
- `BookStats` 结构体：`total_words: usize, chapters: Vec<(String, usize)>, last_calculated: u64`
- 在 `panel/novel.rs` 底部增加统计展示 UI

## 优先级
🟡 中

## 验收标准
- [ ] 打开含多章节的项目后，底部/侧边显示正确总字数
- [ ] 各章字数与手动 `count_words()` 结果一致
- [ ] `daily_word_goal = 1000` 时，写满 1000 字后进度条显示 100%
- [ ] `count_words_in_dir` 有单元测试（使用 temp_dir）
