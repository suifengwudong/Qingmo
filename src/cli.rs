// ── CLI LLM 调试模块 ──────────────────────────────────────────────────────────
//
// 用法示例:
//   qingmo llm --prompt "写一段开场白"
//   qingmo llm --template continuation --input "主角走进了森林。"
//   qingmo llm --backend api --url http://localhost:11434/api/generate --prompt "续写"
//   echo "你好" | qingmo llm --backend mock
//
// 当检测到第一个参数为 "llm" 时，程序进入 CLI 模式，不启动 GUI。
// 其他情况下正常启动 GUI。

use std::io::Read;

use crate::app::{ApiBackend, LocalServerBackend, LlmBackend, LlmConfig, MockBackend, PromptTemplate};

// ── CliArgs ───────────────────────────────────────────────────────────────────

/// Parsed CLI arguments for the `llm` subcommand.
#[derive(Debug)]
pub struct CliArgs {
    /// Prompt text.  If None, will be read from stdin.
    pub prompt: Option<String>,
    /// Optional context block prepended to the prompt template.
    pub context: Option<String>,
    /// Optional user input used with a PromptTemplate.
    pub input: Option<String>,
    /// Prompt template name (e.g. "continuation", "expansion").
    pub template: Option<String>,
    /// Backend selector: "mock" | "api" | "local".
    pub backend: String,
    /// API / server URL.
    pub url: Option<String>,
    /// Model name / path.
    pub model: Option<String>,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Max tokens to generate.
    pub max_tokens: Option<u32>,
    /// System prompt.
    pub system_prompt: Option<String>,
}

impl Default for CliArgs {
    fn default() -> Self {
        CliArgs {
            prompt: None,
            context: None,
            input: None,
            template: None,
            backend: "mock".to_owned(),
            url: None,
            model: None,
            temperature: None,
            max_tokens: None,
            system_prompt: None,
        }
    }
}

// ── parse_args ────────────────────────────────────────────────────────────────

/// Parse the argument list that follows `llm` (i.e. `args[1..]`).
///
/// Supported flags:
///   --prompt     TEXT
///   --context    TEXT
///   --input      TEXT
///   --template   NAME   (continuation|expansion|dialogue|character|scene|twist|ending|outline)
///   --backend    NAME   (mock|api|local)
///   --url        URL
///   --model      NAME
///   --temperature FLOAT
///   --tokens     INT
///   --system     TEXT
///
/// Returns an error string if an unknown flag or missing value is encountered.
pub fn parse_args(args: &[String]) -> Result<CliArgs, String> {
    let mut out = CliArgs::default();
        let mut i = 0_usize;

    while i < args.len() {
        let flag = &args[i];
        match flag.as_str() {
            "--prompt" | "-p" => {
                out.prompt = Some(next_value(args, &mut i, "--prompt")?);
            }
            "--context" | "-c" => {
                out.context = Some(next_value(args, &mut i, "--context")?);
            }
            "--input" | "-i" => {
                out.input = Some(next_value(args, &mut i, "--input")?);
            }
            "--template" | "-t" => {
                out.template = Some(next_value(args, &mut i, "--template")?);
            }
            "--backend" | "-b" => {
                out.backend = next_value(args, &mut i, "--backend")?;
            }
            "--url" | "-u" => {
                out.url = Some(next_value(args, &mut i, "--url")?);
            }
            "--model" | "-m" => {
                out.model = Some(next_value(args, &mut i, "--model")?);
            }
            "--temperature" => {
                let s = next_value(args, &mut i, "--temperature")?;
                out.temperature = Some(s.parse::<f32>()
                    .map_err(|_| format!("--temperature: 无效的浮点数 \"{}\"", s))?);
            }
            "--tokens" | "--max-tokens" => {
                let s = next_value(args, &mut i, "--tokens")?;
                out.max_tokens = Some(s.parse::<u32>()
                    .map_err(|_| format!("--tokens: 无效的整数 \"{}\"", s))?);
            }
            "--system" | "-s" => {
                out.system_prompt = Some(next_value(args, &mut i, "--system")?);
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(format!("未知参数: {}，使用 --help 查看帮助", other));
            }
        }
        i += 1;
    }

    Ok(out)
}

fn next_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    *i += 1;
    args.get(*i)
        .cloned()
        .ok_or_else(|| format!("参数 {} 需要一个值", flag))
}

// ── resolve_prompt ────────────────────────────────────────────────────────────

/// Build the final prompt string from `CliArgs`.
///
/// Priority:
/// 1. If `--template` is given, build from template with `--context` and `--input`.
/// 2. If `--prompt` is given, use it directly (optionally prepended with `--context`).
/// 3. Otherwise, read from stdin.
pub fn resolve_prompt(args: &CliArgs) -> Result<String, String> {
    if let Some(tmpl_name) = &args.template {
        let template = parse_template(tmpl_name)?;
        let context = args.context.as_deref().unwrap_or("");
        let input   = args.input.as_deref()
            .or(args.prompt.as_deref())
            .unwrap_or("");
        if input.is_empty() {
            return Err("使用 --template 时请通过 --input 或 --prompt 提供输入文本".to_owned());
        }
        return Ok(template.fill(context, input));
    }

    if let Some(p) = &args.prompt {
        let ctx = args.context.as_deref().unwrap_or("");
        if ctx.is_empty() {
            return Ok(p.clone());
        }
        return Ok(format!("{}\n\n{}", ctx.trim(), p.trim()));
    }

    // Read from stdin
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("读取 stdin 失败: {e}"))?;
    let trimmed = buf.trim().to_owned();
    if trimmed.is_empty() {
        Err("提示词为空，请通过 --prompt 或 stdin 提供输入".to_owned())
    } else {
        let ctx = args.context.as_deref().unwrap_or("");
        if ctx.is_empty() {
            Ok(trimmed)
        } else {
            Ok(format!("{}\n\n{}", ctx.trim(), trimmed))
        }
    }
}

fn parse_template(name: &str) -> Result<PromptTemplate, String> {
    match name.to_lowercase().as_str() {
        "continuation" | "续写" | "cont"  => Ok(PromptTemplate::Continuation),
        "expansion"    | "扩写" | "exp"   => Ok(PromptTemplate::Expansion),
        "dialogue"     | "对话" | "dial"  => Ok(PromptTemplate::DialogueOptimize),
        "character"    | "人物" | "char"  => Ok(PromptTemplate::CharacterSummary),
        "scene"        | "场景"           => Ok(PromptTemplate::SceneOpening),
        "twist"        | "转折"           => Ok(PromptTemplate::PlotTwist),
        "ending"       | "结尾"           => Ok(PromptTemplate::ChapterEnding),
        "outline"      | "大纲"           => Ok(PromptTemplate::ChapterOutline),
        other => Err(format!(
            "未知模板 \"{}\"。可用: continuation, expansion, dialogue, character, scene, twist, ending, outline",
            other
        )),
    }
}

// ── build_config ──────────────────────────────────────────────────────────────

/// Build an `LlmConfig` by starting from the saved `~/.config/qingmo/config.json`
/// (if it exists) and overriding with any CLI-supplied values.
pub fn build_config(args: &CliArgs) -> LlmConfig {
    // Try to load saved config for sensible defaults.
    let saved = load_saved_config();

    // Fallback defaults mirror the values used in TextToolApp::new() (app/mod.rs).
    let mut cfg = saved.unwrap_or_else(|| LlmConfig {
        model_path:    String::new(),
        api_url:       "http://localhost:11434/api/generate".to_owned(),
        temperature:   0.7,
        max_tokens:    512,
        use_local:     false,
        system_prompt: String::new(),
    });

    // CLI flags override saved config.
    if let Some(url) = &args.url {
        cfg.api_url = url.clone();
    }
    if let Some(model) = &args.model {
        cfg.model_path = model.clone();
    }
    if let Some(t) = args.temperature {
        cfg.temperature = t;
    }
    if let Some(n) = args.max_tokens {
        cfg.max_tokens = n;
    }
    if let Some(sys) = &args.system_prompt {
        cfg.system_prompt = sys.clone();
    }

    cfg
}

/// Attempt to load `~/.config/qingmo/config.json` and return the LlmConfig.
fn load_saved_config() -> Option<LlmConfig> {
    #[cfg(target_os = "windows")]
    let home = std::env::var_os("USERPROFILE").map(std::path::PathBuf::from)?;
    #[cfg(not(target_os = "windows"))]
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from)?;

    let path = home.join(".config").join("qingmo").join("config.json");
    let text = std::fs::read_to_string(&path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&text).ok()?;
    let llm = val.get("llm_config")?;
    serde_json::from_value(llm.clone()).ok()
}

// ── build_backend ─────────────────────────────────────────────────────────────

/// Create the LLM backend from the `--backend` flag.
pub fn build_backend(args: &CliArgs) -> Result<std::sync::Arc<dyn LlmBackend>, String> {
    match args.backend.to_lowercase().as_str() {
        "mock"  | "模拟"         => Ok(std::sync::Arc::new(MockBackend)),
        "api"   | "http"         => Ok(std::sync::Arc::new(ApiBackend)),
        "local" | "llama" | "本地" => Ok(std::sync::Arc::new(LocalServerBackend)),
        other => Err(format!(
            "未知后端 \"{}\"。可用: mock, api, local",
            other
        )),
    }
}

// ── run_cli ───────────────────────────────────────────────────────────────────

/// Entry point for the CLI `llm` subcommand.
///
/// Parses `args` (everything after the `llm` subcommand word), builds the
/// config / backend / prompt, calls the backend, and writes the result to
/// stdout.  Errors are printed to stderr and the process exits with code 1.
pub fn run_cli(args: &[String]) {
    let parsed = match parse_args(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let prompt = match resolve_prompt(&parsed) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let config  = build_config(&parsed);
    let backend = match build_backend(&parsed) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    match backend.complete(&config, &prompt) {
        Ok(response) => println!("{response}"),
        Err(e)       => {
            eprintln!("LLM 调用失败: {e}");
            std::process::exit(1);
        }
    }
}

// ── help ──────────────────────────────────────────────────────────────────────

fn print_help() {
    println!(
        r#"清墨 LLM 命令行调试工具

用法:
  qingmo llm [选项]

选项:
  -p, --prompt TEXT         提示词（不指定则从 stdin 读取）
  -c, --context TEXT        上下文块（拼接在提示词前）
  -i, --input TEXT          模板输入文本（与 --template 配合使用）
  -t, --template NAME       使用内置模板生成提示词
                            可选值: continuation, expansion, dialogue,
                                    character, scene, twist, ending, outline
  -b, --backend NAME        LLM 后端（默认: mock）
                            可选值: mock, api, local
  -u, --url URL             API 端点 URL
  -m, --model NAME          模型名称 / 路径
      --temperature FLOAT   采样温度（0.0–2.0）
      --tokens INT          最大生成 Token 数
  -s, --system TEXT         系统提示词
  -h, --help                显示帮助信息

示例:
  # 使用模拟后端测试模板
  qingmo llm -b mock -t continuation -i "主角走进了森林。"

  # 调用本地 Ollama 服务
  qingmo llm -b api -u http://localhost:11434/api/generate \
             -m llama2 -p "写一段开场白"

  # 从 stdin 读取并用 llama.cpp 本地服务器续写
  echo "黑暗中，一双眼睛缓缓睁开。" | \
    qingmo llm -b local -u http://127.0.0.1:8080 -t continuation

  # 大批量调试：逐行读取提示词文件并写入结果
  while IFS= read -r line; do
    qingmo llm -b api -m gpt-4o -p "$line"
    echo "---"
  done < prompts.txt > results.txt
"#
    );
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn strs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // ── parse_args ────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_args_defaults() {
        let args = parse_args(&[]).unwrap();
        assert_eq!(args.backend, "mock");
        assert!(args.prompt.is_none());
        assert!(args.template.is_none());
    }

    #[test]
    fn test_parse_args_prompt_and_backend() {
        let args = parse_args(&strs(&["--prompt", "你好", "--backend", "api"])).unwrap();
        assert_eq!(args.prompt.as_deref(), Some("你好"));
        assert_eq!(args.backend, "api");
    }

    #[test]
    fn test_parse_args_short_flags() {
        let args = parse_args(&strs(&["-p", "测试", "-b", "local", "-m", "llama2"])).unwrap();
        assert_eq!(args.prompt.as_deref(), Some("测试"));
        assert_eq!(args.backend, "local");
        assert_eq!(args.model.as_deref(), Some("llama2"));
    }

    #[test]
    fn test_parse_args_temperature_and_tokens() {
        let args = parse_args(&strs(&["--temperature", "0.5", "--tokens", "256"])).unwrap();
        assert!((args.temperature.unwrap() - 0.5).abs() < 1e-6);
        assert_eq!(args.max_tokens, Some(256));
    }

    #[test]
    fn test_parse_args_template() {
        let args = parse_args(&strs(&["--template", "continuation", "--input", "片段"])).unwrap();
        assert_eq!(args.template.as_deref(), Some("continuation"));
        assert_eq!(args.input.as_deref(), Some("片段"));
    }

    #[test]
    fn test_parse_args_context_and_system() {
        let args = parse_args(&strs(&["-c", "人物背景", "-s", "你是写手"])).unwrap();
        assert_eq!(args.context.as_deref(), Some("人物背景"));
        assert_eq!(args.system_prompt.as_deref(), Some("你是写手"));
    }

    #[test]
    fn test_parse_args_unknown_flag() {
        let result = parse_args(&strs(&["--unknown"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--unknown"));
    }

    #[test]
    fn test_parse_args_missing_value() {
        let result = parse_args(&strs(&["--prompt"]));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_invalid_temperature() {
        let result = parse_args(&strs(&["--temperature", "abc"]));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_invalid_tokens() {
        let result = parse_args(&strs(&["--tokens", "not_a_number"]));
        assert!(result.is_err());
    }

    // ── resolve_prompt ────────────────────────────────────────────────────────

    #[test]
    fn test_resolve_prompt_direct() {
        let mut args = CliArgs::default();
        args.prompt = Some("直接提示词".to_owned());
        let p = resolve_prompt(&args).unwrap();
        assert_eq!(p, "直接提示词");
    }

    #[test]
    fn test_resolve_prompt_with_context() {
        let mut args = CliArgs::default();
        args.prompt  = Some("请续写".to_owned());
        args.context = Some("人物：李明".to_owned());
        let p = resolve_prompt(&args).unwrap();
        assert!(p.contains("人物：李明"));
        assert!(p.contains("请续写"));
    }

    #[test]
    fn test_resolve_prompt_template_continuation() {
        let mut args = CliArgs::default();
        args.template = Some("continuation".to_owned());
        args.input    = Some("主角走进了森林。".to_owned());
        let p = resolve_prompt(&args).unwrap();
        assert!(p.contains("续写"));
        assert!(p.contains("主角走进了森林。"));
    }

    #[test]
    fn test_resolve_prompt_template_with_context() {
        let mut args = CliArgs::default();
        args.template = Some("expansion".to_owned());
        args.input    = Some("夜晚，月光如水。".to_owned());
        args.context  = Some("场景：古代江南".to_owned());
        let p = resolve_prompt(&args).unwrap();
        assert!(p.contains("扩写"));
        assert!(p.contains("江南"));
        assert!(p.contains("夜晚，月光如水。"));
    }

    #[test]
    fn test_resolve_prompt_template_missing_input() {
        let mut args = CliArgs::default();
        args.template = Some("twist".to_owned());
        let result = resolve_prompt(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--input"));
    }

    #[test]
    fn test_resolve_prompt_template_uses_prompt_as_input() {
        // If --input is missing but --prompt is given, it is used as the input.
        let mut args = CliArgs::default();
        args.template = Some("outline".to_owned());
        args.prompt   = Some("一个少年踏上旅途。".to_owned());
        let p = resolve_prompt(&args).unwrap();
        assert!(p.contains("大纲"));
        assert!(p.contains("一个少年踏上旅途。"));
    }

    // ── parse_template ────────────────────────────────────────────────────────

    #[test]
    fn test_parse_template_all_english_names() {
        let names = ["continuation", "expansion", "dialogue", "character",
                     "scene", "twist", "ending", "outline"];
        for name in names {
            assert!(parse_template(name).is_ok(), "template '{}' should be valid", name);
        }
    }

    #[test]
    fn test_parse_template_all_chinese_names() {
        let names = ["续写", "扩写", "对话", "人物", "场景", "转折", "结尾", "大纲"];
        for name in names {
            assert!(parse_template(name).is_ok(), "template '{}' should be valid", name);
        }
    }

    #[test]
    fn test_parse_template_case_insensitive() {
        assert!(parse_template("Continuation").is_ok());
        assert!(parse_template("EXPANSION").is_ok());
    }

    #[test]
    fn test_parse_template_unknown() {
        let result = parse_template("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonexistent"));
    }

    // ── build_backend ─────────────────────────────────────────────────────────

    #[test]
    fn test_build_backend_mock() {
        let mut args = CliArgs::default();
        args.backend = "mock".to_owned();
        assert!(build_backend(&args).is_ok());
    }

    #[test]
    fn test_build_backend_api() {
        let mut args = CliArgs::default();
        args.backend = "api".to_owned();
        assert!(build_backend(&args).is_ok());
    }

    #[test]
    fn test_build_backend_local() {
        let mut args = CliArgs::default();
        args.backend = "local".to_owned();
        assert!(build_backend(&args).is_ok());
    }

    #[test]
    fn test_build_backend_unknown() {
        let mut args = CliArgs::default();
        args.backend = "unknownbackend".to_owned();
        let result = build_backend(&args);
        assert!(result.is_err());
    }

    // ── build_config ──────────────────────────────────────────────────────────

    #[test]
    fn test_build_config_overrides() {
        let mut args = CliArgs::default();
        args.url         = Some("http://myserver/v1/chat/completions".to_owned());
        args.model       = Some("gpt-4o".to_owned());
        args.temperature = Some(1.0);
        args.max_tokens  = Some(1024);
        args.system_prompt = Some("你是助手".to_owned());

        let cfg = build_config(&args);
        assert_eq!(cfg.api_url, "http://myserver/v1/chat/completions");
        assert_eq!(cfg.model_path, "gpt-4o");
        assert!((cfg.temperature - 1.0).abs() < 1e-6);
        assert_eq!(cfg.max_tokens, 1024);
        assert_eq!(cfg.system_prompt, "你是助手");
    }

    #[test]
    fn test_build_config_no_overrides_has_defaults() {
        let args = CliArgs::default();
        let cfg = build_config(&args);
        // Should have some default URL
        assert!(!cfg.api_url.is_empty());
        assert!(cfg.temperature > 0.0);
        assert!(cfg.max_tokens > 0);
    }

    // ── integration: parse → resolve → backend.complete ───────────────────────

    #[test]
    fn test_cli_mock_end_to_end() {
        use crate::app::MockBackend;
        let args = parse_args(&strs(&[
            "--prompt", "写一段开场白",
            "--backend", "mock",
        ])).unwrap();
        let prompt = resolve_prompt(&args).unwrap();
        let cfg    = build_config(&args);
        let result = MockBackend.complete(&cfg, &prompt);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("模拟输出"));
    }
}
