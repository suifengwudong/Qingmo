use crate::app::LlmConfig;

// ── LlmBackend trait ──────────────────────────────────────────────────────────

/// Abstraction over different LLM backends.
///
/// Implementations are expected to be called from a **background thread**
/// so that the egui UI is never blocked.  The trait is `Send + Sync` to
/// support being wrapped in `Arc<dyn LlmBackend>` and shared across threads.
pub trait LlmBackend: Send + Sync {
    /// Send a completion request with the given prompt and return the
    /// model's text response, or a human-readable error string on failure.
    fn complete(&self, config: &LlmConfig, prompt: &str) -> Result<String, String>;

    /// Human-readable name shown in the UI.
    fn name(&self) -> &'static str;
}

// ── MockBackend ───────────────────────────────────────────────────────────────

/// Simulated backend – returns a canned response without any network call.
/// Useful for offline development and tests.
pub struct MockBackend;

impl LlmBackend for MockBackend {
    fn name(&self) -> &'static str { "模拟模型" }

    fn complete(&self, config: &LlmConfig, prompt: &str) -> Result<String, String> {
        if prompt.trim().is_empty() {
            return Err("提示词为空，请输入内容后再试".to_owned());
        }
        Ok(format!(
            "【模拟输出 – 请配置真实模型】\n\n根据您的提示「{}…」，这里将显示模型生成的文本。\n\n当前配置:\n- {}: {}\n- 温度: {:.2}\n- 最大Token: {}",
            prompt.chars().take(30).collect::<String>(),
            if config.use_local { "本地模型" } else { "API" },
            if config.use_local { &config.model_path } else { &config.api_url },
            config.temperature,
            config.max_tokens,
        ))
    }
}

// ── ApiBackend ────────────────────────────────────────────────────────────────

/// HTTP API backend – supports both Ollama-style (`/api/generate`) and
/// OpenAI-compatible (`/v1/chat/completions`) endpoints.
///
/// Selection heuristic:
///   - URL ending in `/api/generate`  → Ollama request body
///   - URL ending in `/chat/completions` → OpenAI request body
///   - Otherwise                          → OpenAI request body
pub struct ApiBackend;

impl LlmBackend for ApiBackend {
    fn name(&self) -> &'static str { "HTTP API" }

    fn complete(&self, config: &LlmConfig, prompt: &str) -> Result<String, String> {
        let url = config.api_url.trim_end_matches('/');

        if url.ends_with("/api/generate") {
            Self::call_ollama(config, prompt)
        } else {
            Self::call_openai(config, prompt)
        }
    }
}

impl ApiBackend {
    /// Call an Ollama `/api/generate` endpoint.
    fn call_ollama(config: &LlmConfig, prompt: &str) -> Result<String, String> {
        let model = Self::model_name(config);
        let body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": config.temperature,
                "num_predict": config.max_tokens,
            }
        });

        let mut response = ureq::post(&config.api_url)
            .send_json(&body)
            .map_err(|e| format!("请求失败 ({}): {e}", config.api_url))?;

        let json: serde_json::Value = response
            .body_mut()
            .read_json()
            .map_err(|e| format!("响应解析失败: {e}"))?;

        json.get("response")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned())
            .ok_or_else(|| format!("无法从响应中读取 'response' 字段: {json}"))
    }

    /// Call an OpenAI-compatible `/v1/chat/completions` endpoint.
    fn call_openai(config: &LlmConfig, prompt: &str) -> Result<String, String> {
        let model = Self::model_name(config);
        let body = serde_json::json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
        });

        let mut response = ureq::post(&config.api_url)
            .send_json(&body)
            .map_err(|e| format!("请求失败 ({}): {e}", config.api_url))?;

        let json: serde_json::Value = response
            .body_mut()
            .read_json()
            .map_err(|e| format!("响应解析失败: {e}"))?;

        json.get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("message"))
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned())
            .ok_or_else(|| format!("无法从响应中读取 choices[0].message.content: {json}"))
    }

    /// Extract a model name from the config: use model_path if set (local), else a default.
    fn model_name(config: &LlmConfig) -> &str {
        let path = config.model_path.trim();
        if !path.is_empty() { path } else { "llama2" }
    }
}

// ── LlmTask ───────────────────────────────────────────────────────────────────

/// State for a non-blocking LLM request running on a background thread.
/// The UI polls `try_recv()` each frame to check for completion.
pub struct LlmTask {
    pub receiver: std::sync::mpsc::Receiver<Result<String, String>>,
}

impl LlmTask {
    /// Spawn a background thread that calls `backend.complete(config, prompt)` and
    /// sends the result back through the returned `LlmTask`.
    pub fn spawn(
        backend: std::sync::Arc<dyn LlmBackend>,
        config: LlmConfig,
        prompt: String,
    ) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = backend.complete(&config, &prompt);
            let _ = tx.send(result);
        });
        LlmTask { receiver: rx }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::LlmConfig;

    fn default_config() -> LlmConfig {
        LlmConfig {
            model_path: String::new(),
            api_url: "http://localhost:11434/api/generate".to_owned(),
            temperature: 0.7,
            max_tokens: 512,
            use_local: true,
        }
    }

    #[test]
    fn test_mock_backend_empty_prompt() {
        let backend = MockBackend;
        let result = backend.complete(&default_config(), "");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("提示词为空"));
    }

    #[test]
    fn test_mock_backend_with_prompt() {
        let backend = MockBackend;
        let result = backend.complete(&default_config(), "写一段开场白");
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("模拟输出"));
    }

    #[test]
    fn test_mock_backend_name() {
        let backend = MockBackend;
        assert_eq!(backend.name(), "模拟模型");
    }

    #[test]
    fn test_api_backend_name() {
        let backend = ApiBackend;
        assert_eq!(backend.name(), "HTTP API");
    }

    #[test]
    fn test_llm_task_mock() {
        let backend: std::sync::Arc<dyn LlmBackend> = std::sync::Arc::new(MockBackend);
        let task = LlmTask::spawn(backend, default_config(), "测试提示词".to_owned());
        // The mock completes instantly; wait a moment and check
        let result = task.receiver.recv_timeout(std::time::Duration::from_secs(2));
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}
