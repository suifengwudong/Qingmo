# [Feature] 字数统计面板与每日写作目标

## 描述
作者通常需要跟踪自己的写作进度，例如「今天写了多少字」「全书总字数」「距目标还差多少」。目前只有编辑器左下角单个文件的实时字符数（非中文词数）。需要一个更完善的字数统计系统。

## 期望行为

### 统计维度
1. **当前文件**：显示光标所在章节的字数（汉字 + 英文单词数，标点/空格不计入）。
2. **全书汇总**：扫描 `Content/` 目录下所有 `.md` 文件，汇总总字数；分卷/分章显示字数分布（与 `StructNode` 树联动）。
3. **今日增量**：记录本次会话开始时各文件的字数快照，每次保存后与快照对比，累计「今日新增字数」。
4. **写作目标**：用户可在设置中设定「每日目标字数」（默认 1000 字），状态栏/统计面板显示进度条。
5. **历史记录（可选）**：将每日完成字数写入 `~/.config/qingmo/word_count_log.json`，支持简单折线图展示（纯 egui Painter 绘制）。

## 实现建议

### 字数计算函数
```rust
/// Count CJK characters + English word-tokens, ignoring Markdown syntax.
pub fn count_words(md: &str) -> usize {
    let plain = strip_markdown_syntax(md); // 复用 issue6 中的 markdown_to_plain_text
    let mut count = 0;
    let mut in_word = false;
    for ch in plain.chars() {
        if ch as u32 >= 0x4E00 && ch as u32 <= 0x9FFF {
            // CJK Unified Ideograph: each character = 1 word
            count += 1;
            in_word = false;
        } else if ch.is_alphabetic() {
            if !in_word { count += 1; }
            in_word = true;
        } else {
            in_word = false;
        }
    }
    count
}
```

### 数据结构
```rust
/// Per-session word-count baseline (populated on project open).
pub struct WordCountBaseline {
    pub file_counts: HashMap<PathBuf, usize>,
    pub session_start: std::time::Instant,
}

/// Persisted daily record (written to config dir).
#[derive(Serialize, Deserialize)]
pub struct DailyRecord {
    pub date: String,      // "2026-03-30"
    pub added_words: usize,
}
```

### 统计面板 UI（新增「统」图标 tab，或整合到现有「纲」面板底部）
```rust
ui.heading("📊 字数统计");
ui.separator();
ui.label(format!("当前章节: {} 字", current_count));
ui.label(format!("全书合计: {} 字", total_count));
ui.label(format!("今日新增: +{} 字", today_added));
ui.separator();
ui.label("每日目标");
let goal = self.config.daily_word_goal;
if goal > 0 {
    let progress = (today_added as f32 / goal as f32).min(1.0);
    ui.add(egui::ProgressBar::new(progress)
        .text(format!("{}/{} 字  ({:.0}%)", today_added, goal, progress * 100.0)));
}
```

### 相关文件
- `src/app/models.rs`：`DailyRecord`，`AppConfig` 添加 `daily_word_goal: u32` 字段
- `src/app/mod.rs`：`WordCountBaseline` 字段，项目打开时采集快照
- `src/app/panel/outline.rs` 或新增 `panel/stats.rs`：字数统计面板渲染
- `src/app/search.rs` 或 `file_manager.rs`：`count_words()` 工具函数
- `~/.config/qingmo/word_count_log.json`：历史记录持久化

## 优先级
🟡 中——对有日更目标的连载作者尤为实用，实现难度适中

## 验收标准
- [ ] 状态栏显示当前文件字数（汉字+英文单词，不含标点空格）
- [ ] 统计面板显示当前章节、全书合计、今日新增三项数据
- [ ] 设置中可配置每日写作目标，面板显示进度条
- [ ] 数据在会话期间实时更新（每次保存后刷新）
- [ ] 不引入新的 crate 依赖
