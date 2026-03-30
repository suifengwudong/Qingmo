# [Feature] 新项目模板：空白项目 / 短篇小说 / 长篇小说

## 描述
当前「应用模板」功能（`apply_template_short` / `apply_template_long`）已在代码中实现，但缺少入口 UI。用户新建项目后需手动创建目录结构，学习成本较高。需要在「新建项目」流程中提供模板选择对话框。

## 期望行为
1. **触发时机**：用户通过「文件 → 新建项目」选择项目文件夹后，弹出模板选择弹窗（egui Modal）。
2. **三种模板**：
   - **空白项目**：仅创建 `Content/`、`Design/` 两个空目录
   - **短篇小说**：`Content/` 下生成「序章.md、第一章.md、第二章.md、第三章.md、尾声.md」
   - **长篇小说**：`Content/` 下生成「第一卷/序章.md…、第二卷/第一章.md…」两层结构
3. **跳过选项**：弹窗提供「跳过，创建空白项目」按钮，保持与旧行为兼容。

## 实现建议

### 状态字段（src/app/mod.rs 中的 TextToolApp）
```rust
// 模板选择弹窗是否显示
show_template_dialog: bool,
pending_project_root: Option<PathBuf>,
```

### 模板选择弹窗
```rust
fn show_template_dialog(&mut self, ctx: &egui::Context) {
    if !self.show_template_dialog { return; }
    egui::Modal::new(egui::Id::new("template_dialog")).show(ctx, |ui| {
        ui.heading("选择项目模板");
        ui.separator();
        if ui.button("📄  空白项目").clicked() {
            self.apply_blank_template();
            self.show_template_dialog = false;
        }
        ui.add_space(4.0);
        if ui.button("📖  短篇小说（序章 + 3章 + 尾声）").clicked() {
            self.apply_template_short();
            self.show_template_dialog = false;
        }
        ui.add_space(4.0);
        if ui.button("📚  长篇小说（两卷 × 3章结构）").clicked() {
            self.apply_template_long();
            self.show_template_dialog = false;
        }
        ui.add_space(8.0);
        if ui.button("跳过").clicked() {
            self.show_template_dialog = false;
        }
    });
}
```

### 触发逻辑（「文件 → 新建项目」处理）
```rust
fn open_new_project(&mut self) {
    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
        self.project_root = Some(folder.clone());
        self.pending_project_root = Some(folder);
        self.show_template_dialog = true; // 触发模板选择弹窗
        self.refresh_tree();
    }
}
```

### 相关文件
- `src/app/mod.rs`：「文件」菜单新建项目逻辑，模板弹窗渲染，`apply_blank_template()` 方法
- `src/app/sync.rs`：`apply_template_short()` 和 `apply_template_long()` 已实现，无需修改

## 优先级
🟢 低——降低新用户上手门槛，`apply_template_*` 逻辑已存在，只需补充 UI 入口

## 验收标准
- [ ] 「文件 → 新建项目」打开文件夹后弹出模板选择弹窗
- [ ] 三种模板均可正常创建对应目录结构
- [ ] 「跳过」按钮创建空白项目，行为与旧版一致
- [ ] 模板应用失败时状态栏显示错误信息
