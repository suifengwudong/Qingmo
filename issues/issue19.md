# [优化] 自动保存 Crash Recovery（草稿恢复机制）

## 描述
当前自动保存直接覆盖原文件。若程序在写入过程中崩溃，可能导致文件损坏或内容丢失。参考数据存储安全设计，增加崩溃恢复机制。

## 期望行为
- 自动保存时先写入 `<filename>.swp` 临时文件，写入成功后再原子替换
- 程序启动时检测是否存在 `.swp` 文件，若存在提示用户恢复
- 配置写入（`config.json`）同样先写 `config.json.bak`，再替换

## 实现建议
- 在 `sync.rs` 的 `write_project_file` 封装函数中实现原子写入：
  ```rust
  fn write_atomically(path: &Path, content: &str) -> std::io::Result<()> {
      let tmp = path.with_extension("swp");
      std::fs::write(&tmp, content)?;
      std::fs::rename(&tmp, path)?;  // 原子替换（同分区内）
      Ok(())
  }
  ```
- 启动时扫描项目目录中的 `.swp` 文件，若存在弹出恢复提示
- `AppConfig::save()` 改为先写 `.bak` 再重命名

## 优先级
🟢 低

## 验收标准
- [ ] 模拟写入中断（手动 kill）后重启，`.swp` 文件被检测并提示恢复
- [ ] 正常自动保存流程结束后，无残留 `.swp` 文件
- [ ] `write_atomically` 有单元测试（验证 `.swp` 文件在成功后被清理）
