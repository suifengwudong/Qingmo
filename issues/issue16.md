# [Tech Debt] 单元测试模块化重组

## 描述
当前所有单元测试集中在 `src/app/mod.rs` 的底部 `#[cfg(test)]` 块中，超过 600 行，导致：
1. `mod.rs` 文件过长，可读性差（违反 SRP）。
2. 模型测试与文件管理测试、查找栏测试混在一起，难以定位。
3. 新增测试时不知道放哪里。

## 期望行为

将测试分散到各自的模块中：

| 测试内容 | 目标文件 |
|---------|---------|
| `WorldObject`, `StructNode`, `AppConfig` 等模型 | `src/app/models.rs` 底部 `#[cfg(test)]` |
| `FileNode`, `OpenFile` | `src/app/file_manager.rs` 底部 `#[cfg(test)]` |
| `markdown_to_plain_text`, `count_words`, `search_dir`, `copy_dir_all` | `src/app/search.rs` 底部（已有） |
| `FindBar` 相关 | `src/app/ui_helpers.rs` 底部 `#[cfg(test)]` |
| 集成级测试（需要多模块协作） | 保留在 `src/app/mod.rs` |

## 实现建议

每个模块底部添加：
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // 只测试本模块的类型与函数
}
```

将 `mod.rs` 中对应测试剪切到目标文件，调整 `use` 路径后运行 `cargo test` 验证。

## 优先级
🟡 中——工程化债务，影响长期可维护性

## 验收标准
- [ ] `mod.rs` 测试块不超过 100 行（仅保留集成测试）
- [ ] `models.rs` / `file_manager.rs` / `ui_helpers.rs` 各自有 `#[cfg(test)]` 块
- [ ] `cargo test` 全部通过，测试数量不减少
- [ ] 无重复测试（同一逻辑不在两个文件中各测一遍）
