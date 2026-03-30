# [功能] 章节时间线视图（Scene Timeline）

## 描述
小说写作中，场景的时间顺序与叙事顺序往往不同（如插叙、倒叙）。需要一个"时间线视图"帮助作者管理每个章节/场景在故事世界内部时间轴上的位置，避免时间逻辑矛盾。

## 期望行为
- 在「章节结构」面板中新增"时间线"子视图（与现有的"结构树"并列）
- 每个 `StructNode` 可选填 `scene_date: Option<String>`（故事内部日期，自由格式，如 "第3年春"）
- 时间线视图将节点按 `scene_date` 字典序排列，显示为横向或纵向时间轴
- 未填 `scene_date` 的节点归入"未标注"组
- 支持在时间线视图中点击节点跳转到对应章节文件

## 实现建议
- `StructNode` 增加 `scene_date: Option<String>` 字段（`#[serde(default)]`）
- `panel/structure.rs` 增加时间线渲染函数 `draw_timeline_view`
- 时间线视图使用 `egui::ScrollArea::horizontal` 横向滚动
- 排序逻辑提取为纯函数 `sort_nodes_by_scene_date(nodes: &[&StructNode]) -> Vec<&StructNode>`，便于单元测试

## 优先级
🟡 中

## 验收标准
- [ ] `StructNode` 的 `scene_date` 字段可以在结构面板中编辑和保存
- [ ] 时间线视图按 `scene_date` 字典序显示节点，未标注节点排最后
- [ ] `sort_nodes_by_scene_date` 有单元测试（含空值、混合有值/无值情况）
- [ ] 现有结构数据向前兼容（无 `scene_date` 字段的旧 JSON 正常加载）
