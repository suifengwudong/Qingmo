# [功能] 命令面板（Command Palette）

## 描述
实现类似 VS Code 的命令面板（`Ctrl+P` 或 `Ctrl+Shift+P`），允许用户快速跳转到章节、搜索命令、切换面板，无需鼠标操作。

## 期望行为
- `Ctrl+P`：弹出浮动搜索框，列出所有章节文件（按名称模糊匹配）
- `Ctrl+Shift+P`：列出所有可执行命令（新建章节、保存、导出、切换主题等）
- 使用上下方向键导航，`Enter` 确认，`Esc` 关闭
- 匹配结果实时过滤，高亮匹配字符

## 实现建议
- 在 `ui_helpers.rs` 中新增 `CommandPalette` 结构体（类似 `FindBar`）
- `TextToolApp` 中增加 `command_palette: Option<CommandPalette>` 字段
- 命令列表定义为静态 `&[(&str, fn(&mut TextToolApp))]` 数组
- 文件列表从当前项目 `file_tree` 动态构建
- 模糊匹配：按字符包含顺序过滤（不需要连续）

## 优先级
🟡 中

## 验收标准
- [ ] `Ctrl+P` 打开面板，输入章节名片段能过滤出正确结果并跳转
- [ ] `Ctrl+Shift+P` 打开命令列表，选择"新建章节"能正确执行
- [ ] `Esc` 关闭面板，不影响编辑器焦点
- [ ] 至少 3 个命令面板单元测试（模糊匹配逻辑）
