# [功能] CLI 批量模式（--batch-file）

## 描述
在规模化大模型调试场景下（如 Copilot、Ollama），用户需要一次性运行数十到数百条
提示词并收集结果，逐行调用 CLI 的方式效率低且难以记录元数据。

## 期望行为
- `qingmo llm --batch-file prompts.jsonl` 逐行读取 JSONL 文件，每行为：
  ```json
  { "id": "1", "prompt": "写一段开场白", "template": "scene", "context": "..." }
  ```
- 输出 JSONL 文件（`--output results.jsonl`），每行包含输入 id + 模型响应 + 耗时
- 支持 `--concurrency N`（默认 1）控制并发数，便于对吞吐量基准测试
- 单条失败不中断整体，错误写入 `error` 字段

## 实现建议
- 在 `cli.rs` 中新增 `run_batch(args, batch_path, output_path, concurrency)` 函数
- 使用标准库 `std::thread::spawn` 实现简单线程池（无需异步运行时）
- 输出格式：`{ "id": "...", "response": "...", "elapsed_ms": 123, "error": null }`

## 优先级
🟡 中

## 验收标准
- [ ] 10 条 JSONL 输入，`--concurrency 1` 输出 10 条结果，顺序不变
- [ ] 某条含空 prompt 的行输出 `"error": "提示词为空"`，其他行不受影响
- [ ] `--concurrency 4` 时多线程结果与单线程结果内容相同（Mock backend）
- [ ] `run_batch` 有单元测试覆盖成功与失败两条路径
