# [Feature] 全书字数统计面板（Stats Panel）

## 描述
issue10 已实现当前文件字数统计和每日写作目标进度条。但全书汇总统计、今日增量、历史记录等功能仍未实现。需要一个独立的统计面板或侧边区块展示这些数据。

## 期望行为

### 统计维度
1. **全书合计**：扫描 `Content/` 目录下所有 `.md` 文件，汇总总字数。
2. **今日增量**：记录本次会话开始时各文件的字数快照，每次保存后与快照对比，累计「今日新增字数」。
3. **进度条**：在「章节结构」面板底部或独立 Tab 中显示全书字数和今日增量。

## 实现建议

### 数据结构（在 `mod.rs` TextToolApp 中）
```rust
/// Per-session word-count baseline (populated on project open).
pub(super) word_count_baseline: HashMap<PathBuf, usize>,
pub(super) today_added_words: usize,
```

### 统计面板 UI
```rust
ui.heading("📊 字数统计");
ui.label(format!("全书合计: {} 字", total_count));
ui.label(format!("今日新增: +{} 字", today_added));
if goal > 0 {
    let progress = (today_added as f32 / goal as f32).min(1.0);
    ui.add(egui::ProgressBar::new(progress)
        .text(format!("{}/{} 字  ({:.0}%)", today_added, goal, progress * 100.0)));
}
```

### 相关文件
- `src/app/mod.rs`：`word_count_baseline`、`today_added_words` 字段，项目打开时采集快照
- `src/app/panel/novel.rs` 或新增 `panel/stats.rs`：字数统计面板渲染
- `src/app/search.rs`：`count_words_in_dir()` 工具函数（扫描目录汇总）

## 优先级
🟡 中——对有日更目标的连载作者尤为实用

## 验收标准
- [ ] 打开项目后记录各文件初始字数快照
- [ ] 每次 Ctrl+S 保存后更新「今日新增」累计值
- [ ] 在 UI 中展示全书合计字数
- [ ] 今日增量与每日目标进度条正确显示
- [ ] 不引入新的 crate 依赖
