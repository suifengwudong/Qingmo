# [Feature] 世界对象关系图谱可视化

## 描述
「世界对象」面板目前提供卡片视图和列表视图，但对象之间的关联关系（`ObjectLink`）只能逐条查看，无法直观呈现整体关系网络。增加「图谱」视图可帮助作者快速理解人物关系。

## 期望行为
1. **图谱视图切换**：在「世界对象」面板顶部增加「图谱」tab，与现有「卡片」「列表」并列。
2. **节点与连线**：每个 `WorldObject` 显示为圆形节点（颜色按 `ObjectKind` 区分），`ObjectLink` 显示为带标签的连线。
3. **交互**：支持拖拽移动节点；点击节点跳转至对应对象的详情编辑（复用现有卡片编辑区）。
4. **布局**：初始使用简单力导向布局（弹力模型，纯 Rust 实现，无需外部图形库）；节点位置持久化到 `AppConfig` 或专属 JSON。
5. **无外部依赖**：完全基于 `egui` 自定义绘图（`Painter`），不引入第三方图形库。

## 实现建议

### 数据结构
在 `AppConfig` 或独立文件 `graph_layout.json` 中存储节点位置：

```rust
pub struct GraphLayout {
    /// ObjectIndex → screen position (normalized 0.0-1.0)
    pub positions: HashMap<usize, (f32, f32)>,
}
```

### 渲染逻辑（src/app/panel/characters.rs）
```rust
fn show_graph_view(&mut self, ui: &mut Ui) {
    let painter = ui.painter();
    let rect = ui.available_rect_before_wrap();

    // 绘制连线
    for (i, obj) in self.world_objects.iter().enumerate() {
        let p1 = self.graph_layout.pos(i, rect);
        for link in &obj.links {
            if let Some(j) = self.find_object_index(&link.target_name) {
                let p2 = self.graph_layout.pos(j, rect);
                painter.line_segment([p1, p2], egui::Stroke::new(1.5, Color32::GRAY));
                // 绘制关系标签
                let mid = egui::pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
                painter.text(mid, egui::Align2::CENTER_CENTER, &link.relation,
                    egui::FontId::proportional(11.0), Color32::LIGHT_GRAY);
            }
        }
    }

    // 绘制节点（可拖拽）
    for (i, obj) in self.world_objects.iter().enumerate() {
        let pos = self.graph_layout.pos(i, rect);
        let node_rect = egui::Rect::from_center_size(pos, egui::vec2(80.0, 30.0));
        let response = ui.interact(node_rect, ui.id().with(i), egui::Sense::drag());
        if response.dragged() {
            self.graph_layout.update_pos(i, response.interact_pointer_pos().unwrap(), rect);
        }
        let color = kind_color(obj.kind);
        painter.rect_filled(node_rect, 8.0, color);
        painter.text(node_rect.center(), egui::Align2::CENTER_CENTER, &obj.name,
            egui::FontId::proportional(12.0), Color32::WHITE);
    }
}
```

### 相关文件
- `src/app/panel/characters.rs`：增加图谱 tab 与渲染逻辑
- `src/app/models.rs`：`GraphLayout` 数据结构（可选持久化）
- `src/app/mod.rs`：图谱布局读写（可选）

## 优先级
🟡 中——提升人物关系理解效率，对复杂群像故事尤为重要

## 验收标准
- [ ] 「世界对象」面板新增「图谱」tab，可与现有视图切换
- [ ] 所有对象以带颜色区分的节点显示，关联关系以连线+标签展示
- [ ] 节点支持拖拽重新布局
- [ ] 点击节点可查看/编辑对象详情
- [ ] 不引入任何新的 crate 依赖
