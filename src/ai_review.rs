use std::time::Duration;

use serde_json::{json, Value};

use crate::i18n::Language;
use crate::providers::{clean_token, PullRequestDetail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProviderKind {
    OpenAi,
    Anthropic,
    OpenRouter,
    Gemini,
    Custom,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelPreset {
    pub id: &'static str,
    pub badge_en: &'static str,
    pub badge_es: &'static str,
    pub tip_en: &'static str,
    pub tip_es: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct BaseUrlPreset {
    pub label: &'static str,
    pub base_url: &'static str,
    pub suggested_model: Option<&'static str>,
    pub tip_en: &'static str,
    pub tip_es: &'static str,
}

impl AiProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::OpenRouter => "openrouter",
            Self::Gemini => "gemini",
            Self::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "anthropic" | "claude" => Self::Anthropic,
            "openrouter" => Self::OpenRouter,
            "gemini" | "google" => Self::Gemini,
            "custom" => Self::Custom,
            _ => Self::OpenAi,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::OpenRouter => "OpenRouter",
            Self::Gemini => "Gemini",
            Self::Custom => "Custom",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::OpenAi => "gpt-6.2-sol",
            Self::Anthropic => "claude-sonnet-5-5",
            Self::OpenRouter => "anthropic/claude-sonnet-5.5",
            Self::Gemini => "gemini-2.5-flash",
            Self::Custom => "qwen3-coder",
        }
    }

    /// Community-recommended models for Pull Request code reviews per provider.
    pub fn preset_models(self) -> &'static [ModelPreset] {
        match self {
            Self::OpenAi => &[
                ModelPreset {
                    id: "gpt-6.2-sol",
                    badge_en: "Best review",
                    badge_es: "Mejor review",
                    tip_en: "Community top pick on OpenAI: high precision, low false positives, fast and cost-effective for PR reviews.",
                    tip_es: "Top recomendado en OpenAI: alta precisión, pocos falsos positivos, rápido y rentable para PRs.",
                },
                ModelPreset {
                    id: "gpt-6.2-astra",
                    badge_en: "Deep audit",
                    badge_es: "Auditoría profunda",
                    tip_en: "Flagship OpenAI reasoning model — best for complex refactors, concurrency bugs, and security audits.",
                    tip_es: "Modelo insignia de razonamiento de OpenAI — ideal para refactors complejos, concurrencia y seguridad.",
                },
                ModelPreset {
                    id: "gpt-6.1-terra",
                    badge_en: "Fast",
                    badge_es: "Rápido",
                    tip_en: "Fast everyday reviewer for smaller PRs with clear, concise feedback.",
                    tip_es: "Revisor rápido para PRs pequeños con comentarios claros y concisos.",
                },
                ModelPreset {
                    id: "o3",
                    badge_en: "Reasoning",
                    badge_es: "Razonamiento",
                    tip_en: "Classic OpenAI reasoning model trusted for catching subtle logic bugs.",
                    tip_es: "Modelo clásico de razonamiento de OpenAI muy sólido detectando bugs de lógica.",
                },
            ],
            Self::Anthropic => &[
                ModelPreset {
                    id: "claude-sonnet-5-5",
                    badge_en: "Best review",
                    badge_es: "Mejor review",
                    tip_en: "Community #1 for daily PR reviews: top signal-to-noise ratio, crisp diff understanding, and fast turnaround.",
                    tip_es: "Favorito #1 de la comunidad para PRs diarios: excelente señal/ruido, gran comprensión de diffs y rápido.",
                },
                ModelPreset {
                    id: "claude-opus-5-5",
                    badge_en: "Deep audit",
                    badge_es: "Auditoría profunda",
                    tip_en: "Gold standard for deep architectural, race-condition, and OWASP security reviews on critical PRs.",
                    tip_es: "Estándar de oro para auditorías profundas de arquitectura, concurrencia y seguridad OWASP.",
                },
                ModelPreset {
                    id: "claude-fable-5-2",
                    badge_en: "Max reasoning",
                    badge_es: "Máx. razonamiento",
                    tip_en: "Frontier long-horizon reasoning model for massive refactors and complex invariants.",
                    tip_es: "Modelo de razonamiento de frontera para refactors masivos e invariantes complejos.",
                },
                ModelPreset {
                    id: "claude-haiku-5-5",
                    badge_en: "Fast & cheap",
                    badge_es: "Rápido / barato",
                    tip_en: "Ultra-fast, low-cost linter pass for small or routine PRs.",
                    tip_es: "Revisión ultra rápida y económica para PRs pequeños o rutinarios.",
                },
            ],
            Self::OpenRouter => &[
                ModelPreset {
                    id: "anthropic/claude-sonnet-5.5",
                    badge_en: "Best review",
                    badge_es: "Mejor review",
                    tip_en: "Community #1 overall PR reviewer via OpenRouter — balanced speed, accuracy, and low noise.",
                    tip_es: "Favorito #1 en OpenRouter para code review — balance ideal entre velocidad, precisión y poco ruido.",
                },
                ModelPreset {
                    id: "anthropic/claude-opus-5.5",
                    badge_en: "Deep audit",
                    badge_es: "Auditoría profunda",
                    tip_en: "Maximum-depth security and logic review for high-risk PRs.",
                    tip_es: "Revisión de máxima profundidad en seguridad y lógica para PRs críticos.",
                },
                ModelPreset {
                    id: "openai/gpt-6.2-astra",
                    badge_en: "Top OpenAI",
                    badge_es: "Top OpenAI",
                    tip_en: "OpenAI flagship reasoning via OpenRouter — strict structure and thorough edge-case detection.",
                    tip_es: "Insignia de razonamiento de OpenAI vía OpenRouter — estructura estricta y detección de casos borde.",
                },
                ModelPreset {
                    id: "google/gemini-4-neon",
                    badge_en: "Large PRs",
                    badge_es: "PRs grandes",
                    tip_en: "Fast code-focused Gemini 4 with massive context window for large multi-file diffs.",
                    tip_es: "Gemini 4 enfocado en código con ventana de contexto masiva para diffs grandes.",
                },
                ModelPreset {
                    id: "deepseek/deepseek-v4-pro",
                    badge_en: "Best value",
                    badge_es: "Calidad / precio",
                    tip_en: "Frontier-grade code reasoning at a fraction of proprietary model pricing.",
                    tip_es: "Razonamiento de código nivel frontera a una fracción del costo.",
                },
                ModelPreset {
                    id: "qwen/qwen3-coder-next",
                    badge_en: "Code specialist",
                    badge_es: "Especialista código",
                    tip_en: "Open-weight model tuned specifically for software engineering and code auditing.",
                    tip_es: "Modelo open-weight afinado específicamente para ingeniería y auditoría de código.",
                },
            ],
            Self::Gemini => &[
                ModelPreset {
                    id: "gemini-2.5-flash",
                    badge_en: "Best value",
                    badge_es: "Recomendado",
                    tip_en: "Widely available (including Google AI Studio free tier), fast, built-in thinking, and 1M-token context.",
                    tip_es: "Muy disponible (incluye capa gratuita de AI Studio), rápido, con razonamiento y contexto de 1M tokens.",
                },
                ModelPreset {
                    id: "gemini-2.5-pro",
                    badge_en: "Deep audit",
                    badge_es: "Auditoría profunda",
                    tip_en: "Strong reasoning and 1M context window — excels on large multi-file monorepo PRs.",
                    tip_es: "Razonamiento profundo y 1M de contexto — sobresale en PRs grandes de monorepos.",
                },
                ModelPreset {
                    id: "gemini-4-neon",
                    badge_en: "Best code",
                    badge_es: "Mejor código",
                    tip_en: "Latest Gemini 4 coding model built for high-speed repository and diff auditing.",
                    tip_es: "Último modelo Gemini 4 optimizado para código y auditoría rápida de diffs.",
                },
                ModelPreset {
                    id: "gemini-4-carbon",
                    badge_en: "Flagship",
                    badge_es: "Insignia",
                    tip_en: "Flagship Gemini 4 model for demanding technical reasoning and architecture reviews.",
                    tip_es: "Modelo insignia Gemini 4 para razonamiento técnico exigente y arquitectura.",
                },
                ModelPreset {
                    id: "gemini-2.5-flash-lite",
                    badge_en: "Ultra-fast",
                    badge_es: "Ultra rápido",
                    tip_en: "Lowest latency and cost for quick sanity checks on small PRs.",
                    tip_es: "Menor latencia y costo para revisiones rápidas en PRs pequeños.",
                },
            ],
            Self::Custom => &[
                ModelPreset {
                    id: "qwen3-coder",
                    badge_en: "Best local",
                    badge_es: "Mejor local",
                    tip_en: "Community #1 local coding model for Ollama and LM Studio.",
                    tip_es: "Modelo local #1 de la comunidad para código en Ollama y LM Studio.",
                },
                ModelPreset {
                    id: "deepseek-reasoner",
                    badge_en: "DeepSeek",
                    badge_es: "DeepSeek",
                    tip_en: "Official DeepSeek API reasoning model — great for deep bug & security hunting.",
                    tip_es: "Modelo oficial de razonamiento de DeepSeek — ideal para cazar bugs y seguridad.",
                },
                ModelPreset {
                    id: "deepseek-chat",
                    badge_en: "DeepSeek fast",
                    badge_es: "DeepSeek rápido",
                    tip_en: "Official DeepSeek API fast model — very affordable daily PR reviews.",
                    tip_es: "Modelo rápido oficial de DeepSeek — muy económico para PRs diarios.",
                },
                ModelPreset {
                    id: "llama-3.3-70b-versatile",
                    badge_en: "Groq",
                    badge_es: "Groq",
                    tip_en: "High-speed general & coding model on Groq Cloud.",
                    tip_es: "Modelo rápido de propósito general y código en Groq Cloud.",
                },
                ModelPreset {
                    id: "codestral-latest",
                    badge_en: "Mistral",
                    badge_es: "Mistral",
                    tip_en: "Mistral's dedicated code generation and review model.",
                    tip_es: "Modelo dedicado de Mistral para generación y revisión de código.",
                },
                ModelPreset {
                    id: "grok-4.7",
                    badge_en: "xAI",
                    badge_es: "xAI",
                    tip_en: "Latest xAI Grok flagship model via api.x.ai/v1.",
                    tip_es: "Último modelo insignia de xAI Grok vía api.x.ai/v1.",
                },
            ],
        }
    }

    /// Suggested known Base URLs (users can pick one or type any custom URL manually).
    pub fn base_url_presets(self) -> &'static [BaseUrlPreset] {
        match self {
            Self::OpenAi => &[
                BaseUrlPreset {
                    label: "Official",
                    base_url: "",
                    suggested_model: None,
                    tip_en: "Default OpenAI endpoint (https://api.openai.com/v1/chat/completions).",
                    tip_es: "Endpoint oficial de OpenAI (https://api.openai.com/v1/chat/completions).",
                },
                BaseUrlPreset {
                    label: "LiteLLM Proxy",
                    base_url: "http://localhost:4000/v1",
                    suggested_model: None,
                    tip_en: "Local/team LiteLLM OpenAI-compatible gateway.",
                    tip_es: "Gateway local o corporativo LiteLLM compatible con OpenAI.",
                },
            ],
            Self::Anthropic => &[
                BaseUrlPreset {
                    label: "Official",
                    base_url: "",
                    suggested_model: None,
                    tip_en: "Default Anthropic endpoint (https://api.anthropic.com/v1/messages).",
                    tip_es: "Endpoint oficial de Anthropic (https://api.anthropic.com/v1/messages).",
                },
                BaseUrlPreset {
                    label: "LiteLLM Proxy",
                    base_url: "http://localhost:4000",
                    suggested_model: None,
                    tip_en: "Local/team LiteLLM Anthropic proxy.",
                    tip_es: "Proxy LiteLLM local o corporativo para Anthropic.",
                },
            ],
            Self::OpenRouter => &[BaseUrlPreset {
                label: "Official",
                base_url: "",
                suggested_model: None,
                tip_en: "Default OpenRouter endpoint (https://openrouter.ai/api/v1/chat/completions).",
                tip_es: "Endpoint oficial de OpenRouter (https://openrouter.ai/api/v1/chat/completions).",
            }],
            Self::Gemini => &[
                BaseUrlPreset {
                    label: "v1beta (default)",
                    base_url: "",
                    suggested_model: None,
                    tip_en: "Default Google AI Studio v1beta endpoint (supports all Gemini 2.5/4 models).",
                    tip_es: "Endpoint oficial v1beta de Google AI Studio (soporta todos los modelos Gemini 2.5/4).",
                },
                BaseUrlPreset {
                    label: "v1 (stable)",
                    base_url: "https://generativelanguage.googleapis.com/v1",
                    suggested_model: None,
                    tip_en: "Google AI Studio stable v1 endpoint.",
                    tip_es: "Endpoint estable v1 de Google AI Studio.",
                },
            ],
            Self::Custom => &[
                BaseUrlPreset {
                    label: "Ollama",
                    base_url: "http://localhost:11434/v1",
                    suggested_model: Some("qwen3-coder"),
                    tip_en: "Local Ollama server (no API key needed).",
                    tip_es: "Servidor local de Ollama (no requiere API Key).",
                },
                BaseUrlPreset {
                    label: "LM Studio",
                    base_url: "http://localhost:1234/v1",
                    suggested_model: Some("qwen3-coder"),
                    tip_en: "Local LM Studio OpenAI-compatible server (no API key needed).",
                    tip_es: "Servidor local de LM Studio compatible con OpenAI (sin API Key).",
                },
                BaseUrlPreset {
                    label: "vLLM / Local",
                    base_url: "http://localhost:8000/v1",
                    suggested_model: Some("qwen3-coder"),
                    tip_en: "Self-hosted vLLM or llama.cpp OpenAI-compatible server.",
                    tip_es: "Servidor self-hosted vLLM o llama.cpp compatible con OpenAI.",
                },
                BaseUrlPreset {
                    label: "DeepSeek",
                    base_url: "https://api.deepseek.com/v1",
                    suggested_model: Some("deepseek-reasoner"),
                    tip_en: "Official DeepSeek API (requires DeepSeek API key).",
                    tip_es: "API oficial de DeepSeek (requiere API Key de DeepSeek).",
                },
                BaseUrlPreset {
                    label: "Groq",
                    base_url: "https://api.groq.com/openai/v1",
                    suggested_model: Some("llama-3.3-70b-versatile"),
                    tip_en: "Groq Cloud ultra-fast inference (requires gsk_... key).",
                    tip_es: "Groq Cloud inferencia ultra rápida (requiere clave gsk_...).",
                },
                BaseUrlPreset {
                    label: "xAI (Grok)",
                    base_url: "https://api.x.ai/v1",
                    suggested_model: Some("grok-4.7"),
                    tip_en: "Official xAI API for Grok models (requires xAI API key).",
                    tip_es: "API oficial de xAI para modelos Grok (requiere API Key de xAI).",
                },
                BaseUrlPreset {
                    label: "Mistral",
                    base_url: "https://api.mistral.ai/v1",
                    suggested_model: Some("codestral-latest"),
                    tip_en: "Official Mistral AI API (Codestral / Mistral Large).",
                    tip_es: "API oficial de Mistral AI (Codestral / Mistral Large).",
                },
                BaseUrlPreset {
                    label: "Together",
                    base_url: "https://api.together.xyz/v1",
                    suggested_model: Some("deepseek-ai/DeepSeek-V3"),
                    tip_en: "Together AI serverless open-weight models.",
                    tip_es: "Modelos open-weight serverless en Together AI.",
                },
                BaseUrlPreset {
                    label: "Fireworks",
                    base_url: "https://api.fireworks.ai/inference/v1",
                    suggested_model: Some("accounts/fireworks/models/deepseek-v3"),
                    tip_en: "Fireworks AI fast open-weight inference.",
                    tip_es: "Inferencia rápida open-weight en Fireworks AI.",
                },
            ],
        }
    }

    pub fn default_endpoint(self) -> &'static str {
        match self {
            Self::OpenAi => "https://api.openai.com/v1/chat/completions",
            Self::Anthropic => "https://api.anthropic.com/v1/messages",
            Self::OpenRouter => "https://openrouter.ai/api/v1/chat/completions",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Self::Custom => "http://localhost:11434/v1/chat/completions",
        }
    }

    pub fn env_var_name(self) -> Option<&'static str> {
        match self {
            Self::OpenAi => Some("OPENAI_API_KEY"),
            Self::Anthropic => Some("ANTHROPIC_API_KEY"),
            Self::OpenRouter => Some("OPENROUTER_API_KEY"),
            Self::Gemini => Some("GEMINI_API_KEY"),
            Self::Custom => None,
        }
    }

    pub fn api_key_url(self) -> Option<&'static str> {
        match self {
            Self::OpenAi => Some("https://platform.openai.com/api-keys"),
            Self::Anthropic => Some("https://console.anthropic.com/settings/keys"),
            Self::OpenRouter => Some("https://openrouter.ai/keys"),
            Self::Gemini => Some("https://aistudio.google.com/apikey"),
            Self::Custom => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub enabled: bool,
    pub provider: AiProviderKind,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub custom_instructions: String,
}

impl AiConfig {
    pub fn resolved_api_key(&self) -> Option<(String, &'static str)> {
        let cfg = clean_token(&self.api_key);
        if !cfg.is_empty() {
            return Some((cfg, "Settings"));
        }
        if let Some(env_name) = self.provider.env_var_name() {
            if let Ok(v) = std::env::var(env_name) {
                let cleaned = clean_token(&v);
                if !cleaned.is_empty() {
                    return Some((cleaned, env_name));
                }
            }
        }
        None
    }

    pub fn resolved_model(&self) -> String {
        let m = self.model.trim();
        let chosen = if m.is_empty() {
            self.provider.default_model()
        } else {
            m
        };
        if self.provider == AiProviderKind::Gemini {
            chosen.trim_start_matches("models/").to_string()
        } else {
            chosen.to_string()
        }
    }

    pub fn resolved_endpoint(&self) -> String {
        let custom = self.base_url.trim();
        match self.provider {
            AiProviderKind::Gemini => {
                let model = self.resolved_model();
                normalize_gemini_endpoint(custom, &model)
            }
            AiProviderKind::Anthropic => normalize_anthropic_endpoint(custom),
            AiProviderKind::Custom => {
                if custom.is_empty() {
                    self.provider.default_endpoint().to_string()
                } else {
                    normalize_openai_endpoint(custom)
                }
            }
            AiProviderKind::OpenAi | AiProviderKind::OpenRouter => {
                if custom.is_empty() {
                    self.provider.default_endpoint().to_string()
                } else {
                    normalize_openai_endpoint(custom)
                }
            }
        }
    }

    pub fn has_credentials(&self) -> bool {
        match self.provider {
            AiProviderKind::Custom => !self.resolved_endpoint().is_empty(),
            _ => self.resolved_api_key().is_some(),
        }
    }
}

fn normalize_gemini_endpoint(custom_base: &str, model: &str) -> String {
    let trimmed = custom_base.trim().trim_end_matches('/');
    if trimmed.contains(":generateContent") {
        return trimmed.to_string();
    }
    let base = if trimmed.is_empty() {
        "https://generativelanguage.googleapis.com/v1beta"
    } else if let Some(stripped) = trimmed.strip_suffix("/openai/chat/completions") {
        stripped
    } else if let Some(stripped) = trimmed.strip_suffix("/openai") {
        stripped
    } else if let Some(stripped) = trimmed.strip_suffix("/models") {
        stripped
    } else {
        trimmed
    };
    if base.ends_with("/v1beta") || base.ends_with("/v1") {
        format!("{base}/models/{model}:generateContent")
    } else {
        format!("{base}/v1beta/models/{model}:generateContent")
    }
}

fn normalize_anthropic_endpoint(custom_base: &str) -> String {
    let trimmed = custom_base.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "https://api.anthropic.com/v1/messages".to_string()
    } else if trimmed.ends_with("/messages") {
        trimmed.to_string()
    } else if trimmed.ends_with("/v1") {
        format!("{trimmed}/messages")
    } else {
        format!("{trimmed}/v1/messages")
    }
}

fn normalize_openai_endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else if trimmed.ends_with("/v1")
        || trimmed.ends_with("/v1beta")
        || trimmed.ends_with("/v2")
        || trimmed.ends_with("/openai")
    {
        format!("{trimmed}/chat/completions")
    } else {
        format!("{trimmed}/v1/chat/completions")
    }
}

fn is_openai_reasoning_model(model: &str) -> bool {
    let m = model.to_lowercase();
    m.starts_with("o1")
        || m.starts_with("o3")
        || m.starts_with("o4")
        || m.starts_with("gpt-5")
        || m.starts_with("gpt-6")
}

fn gemini_supports_custom_temperature(model: &str) -> bool {
    let m = model.to_lowercase();
    !(m.starts_with("gemini-4") || m.starts_with("gemini-3"))
}

#[derive(Debug, Clone, Default)]
pub enum AiReviewState {
    #[default]
    Idle,
    Running {
        pr_number: u64,
        model: String,
    },
    Ready {
        pr_number: u64,
        model: String,
        report: String,
    },
    Error {
        pr_number: u64,
        message: String,
    },
}

/// Tests that the configured AI provider, API key, endpoint, and model respond cleanly.
pub fn test_ai_connection(cfg: &AiConfig) -> Result<String, String> {
    let model = cfg.resolved_model();
    let reply = send_prompt(
        cfg,
        "You are a concise health check assistant.",
        "Reply with the single word: OK",
        256,
        Duration::from_secs(20),
    )?;
    let snippet: String = reply.trim().chars().take(40).collect();
    Ok(format!("{} • {model} ({snippet})", cfg.provider.label()))
}

/// Runs a comprehensive senior-engineer & linter AI review on a Pull Request.
pub fn review_pull_request(
    cfg: &AiConfig,
    detail: &PullRequestDetail,
    lang: Language,
) -> Result<String, String> {
    if !cfg.enabled {
        return Err("AI Reviewer is disabled. Enable it in Settings → AI Reviewer.".to_string());
    }
    let system_prompt = build_system_prompt(lang, &cfg.custom_instructions);
    let user_prompt = build_pr_review_prompt(detail);
    let raw = send_prompt(cfg, &system_prompt, &user_prompt, 4096, Duration::from_secs(120))?;

    let pr = &detail.summary;
    let model = cfg.resolved_model();
    let header = format!(
        "## AI Code Review — PR #{} (`{}`)\n> Reviewed with **{}** (`{}`) • Author: @{}\n\n",
        pr.number,
        pr.title,
        cfg.provider.label(),
        model,
        pr.author
    );
    Ok(format!("{header}{}", raw.trim()))
}

fn build_system_prompt(lang: Language, custom_instructions: &str) -> String {
    let lang_directive = match lang {
        Language::En => "Write the entire review report in clear, concise English.",
        Language::Es => "Escribe todo el reporte de revisión en español claro y técnico.",
    };

    let mut prompt = format!(
        "You are a Staff Software Engineer, Application Security Architect, and automated code linter reviewing a Pull Request.\n\
        {lang_directive}\n\
        Analyze the PR commits and unified diff patches thoroughly and produce a structured, actionable Markdown report ready to post as a PR review comment.\n\
        Do NOT invent code or files that are not in the diff. Cite exact file paths and line/symbol names.\n\n\
        Use these exact Markdown sections:\n\
        ### Verdict & Summary\n\
        - **Verdict**: `Approve` | `Request Changes` | `Comment`\n\
        - **Risk Level**: `Low` | `Medium` | `High` | `Critical`\n\
        - **Quality Score**: `X/10`\n\
        - Brief 2-3 sentence summary of what the PR does.\n\n\
        ### Security & Cybersecurity\n\
        - Audit for OWASP Top 10, injection (SQL/XSS/Command/Path traversal), auth/authz bypasses, hardcoded secrets/tokens, unsafe deserialization, SSRF, and insecure defaults. Say `No security issues found.` if clean.\n\n\
        ### Bugs, Edge Cases & Correctness\n\
        - Catch null/panic risks, race conditions, error-swallowing, state bugs, breaking API changes, and missing validations.\n\n\
        ### Performance & Benchmarks\n\
        - Spot O(n^2) loops, N+1 queries, blocking I/O on UI/main threads, avoidable clones/allocations, memory leaks, or missing pagination/caching.\n\n\
        ### Code Quality, Linting & Best Practices\n\
        - Flag architecture/SOLID issues, dead code, duplication, naming, maintainability, test coverage gaps, and language-specific idioms.\n\n\
        ### Prioritized Action Items\n\
        - Checklist (`- [ ] **[Critical/High/Medium/Low]** `file`: concrete fix or code snippet`)."
    );

    if !custom_instructions.trim().is_empty() {
        prompt.push_str("\n\nAdditional team conventions & rules:\n");
        prompt.push_str(custom_instructions.trim());
    }
    prompt
}

fn build_pr_review_prompt(detail: &PullRequestDetail) -> String {
    const MAX_DIFF_CHARS: usize = 48_000;
    let pr = &detail.summary;
    let mut out = String::new();

    out.push_str(&format!(
        "PR #{}: {}\nAuthor: @{}\nBranch: {} -> {}\n",
        pr.number, pr.title, pr.author, pr.source_branch, pr.target_branch
    ));
    if let (Some(a), Some(d)) = (pr.additions, pr.deletions) {
        out.push_str(&format!(
            "Stats: +{a} -{d} across {} files\n",
            detail.files.len()
        ));
    }
    if !pr.body.trim().is_empty() {
        let body_trunc: String = pr.body.trim().chars().take(3000).collect();
        out.push_str(&format!("\nPR Description:\n{body_trunc}\n"));
    }

    if !detail.commits.is_empty() {
        out.push_str("\nCommits:\n");
        for c in detail.commits.iter().take(40) {
            out.push_str(&format!("- {} {} ({})\n", c.short_hash, c.message, c.author));
        }
    }

    out.push_str("\nChanged Files Summary:\n");
    for f in &detail.files {
        out.push_str(&format!(
            "- [{}] {} (+{} -{})\n",
            f.status, f.path, f.additions, f.deletions
        ));
    }

    // Prioritize real code patches over lockfiles/generated files within token budget
    let mut ordered: Vec<_> = detail.files.iter().collect();
    ordered.sort_by_key(|f| usize::from(is_noisy_generated_file(&f.path)));

    out.push_str("\nUnified Diff Patches:\n");
    let mut used = 0usize;
    let mut skipped = 0usize;
    for f in ordered {
        if is_noisy_generated_file(&f.path) {
            skipped += 1;
            continue;
        }
        if f.patch.trim().is_empty() {
            continue;
        }
        let header = format!(
            "\n--- FILE: {} [{}] (+{} -{}) ---\n",
            f.path, f.status, f.additions, f.deletions
        );
        if used + header.len() >= MAX_DIFF_CHARS {
            skipped += 1;
            continue;
        }
        out.push_str(&header);
        used += header.len();

        let remaining = MAX_DIFF_CHARS.saturating_sub(used);
        if f.patch.len() <= remaining {
            out.push_str(&f.patch);
            used += f.patch.len();
        } else {
            let slice: String = f.patch.chars().take(remaining).collect();
            out.push_str(&slice);
            out.push_str("\n... [diff truncated due to context budget]\n");
            used = MAX_DIFF_CHARS;
        }
    }

    if skipped > 0 {
        out.push_str(&format!(
            "\nNote: {skipped} lockfile/generated or overflow file patch(es) omitted from diff budget.\n"
        ));
    }

    out
}

fn is_noisy_generated_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    let filename = lower.rsplit('/').next().unwrap_or(&lower);
    matches!(
        filename,
        "cargo.lock"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "composer.lock"
            | "gemfile.lock"
            | "poetry.lock"
            | "pubspec.lock"
            | "go.sum"
    ) || lower.ends_with(".min.js")
        || lower.ends_with(".min.css")
        || lower.ends_with(".map")
        || lower.ends_with(".svg")
        || lower.ends_with(".snap")
}

fn send_prompt(
    cfg: &AiConfig,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
    timeout: Duration,
) -> Result<String, String> {
    let api_key = cfg.resolved_api_key().map(|(k, _)| k);
    if cfg.provider != AiProviderKind::Custom && api_key.is_none() {
        return Err(format!(
            "AI API Error ({}): No API key configured. Open Settings → AI Reviewer and paste your API key.",
            cfg.provider.label()
        ));
    }

    let url = cfg.resolved_endpoint();
    let model = cfg.resolved_model();

    match cfg.provider {
        AiProviderKind::Gemini => {
            let key = api_key.unwrap_or_default();
            let gen_cfg = if gemini_supports_custom_temperature(&model) {
                json!({
                    "maxOutputTokens": max_tokens,
                    "temperature": 0.2
                })
            } else {
                json!({
                    "maxOutputTokens": max_tokens
                })
            };
            let payload = json!({
                "systemInstruction": {
                    "parts": [{ "text": system_prompt }]
                },
                "contents": [
                    {
                        "role": "user",
                        "parts": [{ "text": user_prompt }]
                    }
                ],
                "generationConfig": gen_cfg
            });
            let resp = ureq::post(&url)
                .timeout(timeout)
                .set("x-goog-api-key", &key)
                .set("Content-Type", "application/json")
                .set("User-Agent", "gity-x")
                .send_json(payload)
                .map_err(|e| format_ai_http_error("Gemini", &url, &model, e))?;
            let val: Value = resp
                .into_json()
                .map_err(|e| format!("AI API Error (Gemini): JSON parse error: {e}"))?;
            extract_gemini_text(&val)
                .ok_or_else(|| "AI API Error (Gemini): response contained no text output.".to_string())
        }
        AiProviderKind::Anthropic => {
            let key = api_key.unwrap_or_default();
            let payload = json!({
                "model": model,
                "max_tokens": max_tokens,
                "system": system_prompt,
                "messages": [
                    { "role": "user", "content": user_prompt }
                ]
            });
            let resp = ureq::post(&url)
                .timeout(timeout)
                .set("x-api-key", &key)
                .set("anthropic-version", "2023-06-01")
                .set("content-type", "application/json")
                .set("User-Agent", "gity-x")
                .send_json(payload)
                .map_err(|e| format_ai_http_error("Anthropic", &url, &model, e))?;
            let val: Value = resp
                .into_json()
                .map_err(|e| format!("AI API Error (Anthropic): JSON parse error: {e}"))?;
            extract_anthropic_text(&val)
                .ok_or_else(|| "AI API Error (Anthropic): response contained no text content.".to_string())
        }
        AiProviderKind::OpenAi | AiProviderKind::OpenRouter | AiProviderKind::Custom => {
            let mut req = ureq::post(&url)
                .timeout(timeout)
                .set("Content-Type", "application/json")
                .set("User-Agent", "gity-x");
            if let Some(key) = &api_key {
                req = req.set("Authorization", &format!("Bearer {key}"));
            }
            if cfg.provider == AiProviderKind::OpenRouter {
                req = req
                    .set("HTTP-Referer", "https://github.com/williamjkc69/gity-x")
                    .set("X-Title", "gity x");
            }

            let payload = if cfg.provider == AiProviderKind::OpenAi {
                if is_openai_reasoning_model(&model) {
                    json!({
                        "model": model,
                        "max_completion_tokens": max_tokens,
                        "messages": [
                            { "role": "developer", "content": system_prompt },
                            { "role": "user", "content": user_prompt }
                        ]
                    })
                } else {
                    json!({
                        "model": model,
                        "max_completion_tokens": max_tokens,
                        "temperature": 0.2,
                        "messages": [
                            { "role": "system", "content": system_prompt },
                            { "role": "user", "content": user_prompt }
                        ]
                    })
                }
            } else {
                json!({
                    "model": model,
                    "max_tokens": max_tokens,
                    "temperature": 0.2,
                    "messages": [
                        { "role": "system", "content": system_prompt },
                        { "role": "user", "content": user_prompt }
                    ]
                })
            };

            let resp = req
                .send_json(payload)
                .map_err(|e| format_ai_http_error(cfg.provider.label(), &url, &model, e))?;
            let val: Value = resp
                .into_json()
                .map_err(|e| format!("AI API Error ({}): JSON parse error: {e}", cfg.provider.label()))?;
            extract_openai_text(&val).ok_or_else(|| {
                format!(
                    "AI API Error ({}): response contained no message content.",
                    cfg.provider.label()
                )
            })
        }
    }
}

fn format_ai_http_error(provider: &str, url: &str, model: &str, err: ureq::Error) -> String {
    match err {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            let detail = extract_ai_error_msg(&body);
            let hint = match code {
                400 => "Check model name and Base URL in Settings → AI Reviewer.",
                401 => "Your AI API key was rejected (invalid or revoked). Check Settings → AI Reviewer.",
                403 => "API key lacks permission, billing is disabled, or region is restricted.",
                404 => "Model ID or endpoint URL not found. Leave Base URL empty for default or pick a preset model.",
                429 => "Rate limit or quota exceeded on your AI provider account.",
                500..=599 => "AI provider server error — retry in a few seconds.",
                _ => "Check your AI provider settings and try again.",
            };
            if detail.is_empty() {
                format!("AI API Error ({provider}, HTTP {code})\nModel: {model}\nHint: {hint}\nURL: {url}")
            } else {
                format!("AI API Error ({provider}, HTTP {code}): {detail}\nModel: {model}\nHint: {hint}\nURL: {url}")
            }
        }
        ureq::Error::Transport(t) => {
            format!("AI API Error ({provider} network): {t}\nURL: {url}")
        }
    }
}

fn extract_ai_error_msg(body: &str) -> String {
    if let Ok(val) = serde_json::from_str::<Value>(body) {
        if let Some(msg) = val.pointer("/error/message").and_then(Value::as_str) {
            return msg.to_string();
        }
        if let Some(msg) = val.get("message").and_then(Value::as_str) {
            return msg.to_string();
        }
        if let Some(msg) = val.get("error").and_then(Value::as_str) {
            return msg.to_string();
        }
    }
    body.chars().take(400).collect()
}

fn extract_openai_text(val: &Value) -> Option<String> {
    let content = val.pointer("/choices/0/message/content")?;
    if let Some(s) = content.as_str() {
        return Some(s.to_string());
    }
    if let Some(arr) = content.as_array() {
        let mut out = Vec::new();
        for item in arr {
            if let Some(t) = item.get("text").and_then(Value::as_str) {
                out.push(t);
            }
        }
        if !out.is_empty() {
            return Some(out.join("\n"));
        }
    }
    None
}

fn extract_anthropic_text(val: &Value) -> Option<String> {
    let arr = val.get("content")?.as_array()?;
    let mut parts = Vec::new();
    for item in arr {
        if item.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(t) = item.get("text").and_then(Value::as_str) {
                parts.push(t);
            }
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

fn extract_gemini_text(val: &Value) -> Option<String> {
    let parts = val.pointer("/candidates/0/content/parts")?.as_array()?;
    let mut out = Vec::new();
    for p in parts {
        // Skip internal reasoning/thought blocks from Gemini 2.5 thinking models
        if p.get("thought").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        if let Some(t) = p.get("text").and_then(Value::as_str) {
            out.push(t);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_all_provider_endpoints() {
        assert_eq!(
            normalize_gemini_endpoint("", "gemini-2.5-flash"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent"
        );
        assert_eq!(
            normalize_gemini_endpoint(
                "https://generativelanguage.googleapis.com/v1beta",
                "gemini-2.5-flash"
            ),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent"
        );
        assert_eq!(
            normalize_gemini_endpoint(
                "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
                "gemini-2.5-pro"
            ),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent"
        );
        assert_eq!(
            normalize_anthropic_endpoint("https://api.anthropic.com"),
            "https://api.anthropic.com/v1/messages"
        );
        assert_eq!(
            normalize_openai_endpoint("http://localhost:11434"),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            normalize_openai_endpoint("https://api.deepseek.com/v1"),
            "https://api.deepseek.com/v1/chat/completions"
        );
        assert_eq!(
            normalize_openai_endpoint("https://api.groq.com/openai/v1"),
            "https://api.groq.com/openai/v1/chat/completions"
        );
        assert_eq!(
            normalize_openai_endpoint("https://api.fireworks.ai/inference/v1"),
            "https://api.fireworks.ai/inference/v1/chat/completions"
        );
    }

    #[test]
    fn all_presets_and_base_urls_resolve_cleanly() {
        for prov in [
            AiProviderKind::OpenAi,
            AiProviderKind::Anthropic,
            AiProviderKind::OpenRouter,
            AiProviderKind::Gemini,
            AiProviderKind::Custom,
        ] {
            let models = prov.preset_models();
            assert!(!models.is_empty());
            assert_eq!(models[0].id, prov.default_model());
            for bp in prov.base_url_presets() {
                let cfg = AiConfig {
                    enabled: true,
                    provider: prov,
                    api_key: "test".into(),
                    model: bp
                        .suggested_model
                        .unwrap_or_else(|| prov.default_model())
                        .into(),
                    base_url: bp.base_url.into(),
                    custom_instructions: String::new(),
                };
                let ep = cfg.resolved_endpoint();
                assert!(
                    ep.starts_with("http://") || ep.starts_with("https://"),
                    "invalid endpoint {ep} for {:?} / {}",
                    prov,
                    bp.label
                );
            }
        }
        assert!(is_openai_reasoning_model("gpt-6.2-astra"));
        assert!(is_openai_reasoning_model("gpt-6.2-sol"));
        assert!(!gemini_supports_custom_temperature("gemini-4-neon"));
        assert!(gemini_supports_custom_temperature("gemini-2.5-flash"));
    }

    #[test]
    fn detects_noisy_lockfiles() {
        assert!(is_noisy_generated_file("Cargo.lock"));
        assert!(is_noisy_generated_file("web/package-lock.json"));
        assert!(is_noisy_generated_file("dist/bundle.min.js"));
        assert!(!is_noisy_generated_file("src/providers/github.rs"));
    }

    #[test]
    fn parses_openai_anthropic_and_gemini_responses() {
        let oai = json!({
            "choices": [{ "message": { "content": "Looks good!" } }]
        });
        assert_eq!(extract_openai_text(&oai).as_deref(), Some("Looks good!"));

        let ant = json!({
            "content": [{ "type": "text", "text": "No issues found." }]
        });
        assert_eq!(
            extract_anthropic_text(&ant).as_deref(),
            Some("No issues found.")
        );

        let gem = json!({
            "candidates": [{
                "content": {
                    "parts": [
                        { "text": "internal thought", "thought": true },
                        { "text": "OK" }
                    ]
                }
            }]
        });
        assert_eq!(extract_gemini_text(&gem).as_deref(), Some("OK"));
    }
}
