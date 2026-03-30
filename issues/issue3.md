# [Feature] 亮色/暗色主题切换

## 描述
当前 UI 仅支持 egui 默认暗色主题，无法切换为亮色或跟随系统主题，在白天光照环境下使用体验不佳。

## 期望行为
1. **三种模式**：在设置窗口提供主题选项：`跟随系统`（默认）/ `亮色` / `暗色`。
2. **实时预览**：选择后立即生效，无需重启。
3. **持久化**：主题设置写入 `AppConfig`，下次启动自动恢复。

## 实现建议

### 数据模型
在 `src/app/models.rs` 的 `AppConfig` 中增加字段：

```rust
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self { ThemeMode::System }
}

// 在 AppConfig 中添加：
#[serde(default)]
pub theme: ThemeMode,
```

### 主题应用
在 `src/app/mod.rs` 的 `update()` 函数起始处根据配置设置 Visuals：

```rust
match self.config.theme {
    ThemeMode::Light  => ctx.set_visuals(egui::Visuals::light()),
    ThemeMode::Dark   => ctx.set_visuals(egui::Visuals::dark()),
    ThemeMode::System => { /* 保持 egui 默认，由 OS 决定 */ }
}
```

### 设置窗口 UI
在设置弹窗中添加 ComboBox：

```rust
egui::ComboBox::from_label("主题")
    .selected_text(match self.config.theme {
        ThemeMode::System => "跟随系统",
        ThemeMode::Light  => "亮色",
        ThemeMode::Dark   => "暗色",
    })
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut self.config.theme, ThemeMode::System, "跟随系统");
        ui.selectable_value(&mut self.config.theme, ThemeMode::Light,  "亮色");
        ui.selectable_value(&mut self.config.theme, ThemeMode::Dark,   "暗色");
    });
```

### 相关文件
- `src/app/models.rs`：新增 `ThemeMode` 枚举，`AppConfig` 添加 `theme` 字段
- `src/app/mod.rs`：`update()` 起始处应用主题；设置窗口添加 ComboBox

## 优先级
🟡 中——提升不同使用环境下的视觉舒适度

## 验收标准
- [ ] 可在设置中切换「亮色 / 暗色 / 跟随系统」三种主题
- [ ] 切换后立即生效，无需重启
- [ ] 主题偏好持久化，重启后自动恢复
- [ ] 旧版 `AppConfig` JSON（无 `theme` 字段）可正常反序列化（通过 `#[serde(default)]`）
