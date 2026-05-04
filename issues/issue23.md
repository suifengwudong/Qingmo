# [功能] CLI 上下文文件注入（--world-file / --project）

## 描述
当前 CLI 模式（`qingmo llm`）只支持通过 `--context` 直接传入文本。
对于规模化调试，用户常常需要把整个项目的世界观 JSON / 章节结构
自动注入为 LLM 上下文，而不必手动提取字符串。

## 期望行为
- `--world-file PATH`：读取指定 `world.json`，自动格式化为结构化上下文并追加到提示词
- `--project PATH`：读取 `<project>/data/world.json` 和 `<project>/data/structure.json`，
  同时注入人物设定与章节结构
- 注入内容与 `build_character_context()` / `build_structure_context()` 格式一致
- 现有 `--context` 文本仍然保留，与文件内容合并后一起拼接

## 实现建议
- 在 `cli.rs::CliArgs` 增加 `world_file: Option<PathBuf>` 和 `project_dir: Option<PathBuf>` 字段
- 新增 `build_file_context(args: &CliArgs) -> String` 函数，读取 JSON 并复用
  `models.rs` 中的 `WorldObject` / `StructNode` 反序列化逻辑
- `resolve_prompt` 中将文件上下文与 `--context` 合并后再拼接提示词

## 优先级
🟡 中

## 验收标准
- [ ] `--world-file path/to/world.json` 能正确读取人物信息并注入提示词
- [ ] `--project path/to/project` 同时注入人物+结构上下文
- [ ] `build_file_context` 有单元测试（使用 temp_dir 写入 world.json）
- [ ] 文件不存在时给出友好错误而非 panic
