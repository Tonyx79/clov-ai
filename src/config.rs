use crate::tokenizer::TokenizerProfile;
use crate::universal_filter::FilterContext;
use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub tracking: TrackingConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub filters: FilterConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub tee: crate::tee::TeeConfig,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum McpPreset {
    ClaudeCodeStrict,
    ClaudeCodeBalanced,
    OpenaiBalanced,
    GeminiSearchHeavy,
    LocalDevVerbose,
}

impl McpPreset {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeCodeStrict => "claude-code-strict",
            Self::ClaudeCodeBalanced => "claude-code-balanced",
            Self::OpenaiBalanced => "openai-balanced",
            Self::GeminiSearchHeavy => "gemini-search-heavy",
            Self::LocalDevVerbose => "local-dev-verbose",
        }
    }

    pub fn filter_context(self) -> FilterContext {
        match self {
            Self::ClaudeCodeStrict => FilterContext {
                max_tokens: 2200,
                tokenizer_profile: TokenizerProfile::Claude,
                preserve_code: true,
                aggressive_chrome_strip: true,
                max_array_items: 4,
                max_object_keys: 10,
            },
            Self::ClaudeCodeBalanced => FilterContext {
                max_tokens: 4096,
                tokenizer_profile: TokenizerProfile::Claude,
                preserve_code: true,
                aggressive_chrome_strip: true,
                max_array_items: 6,
                max_object_keys: 16,
            },
            Self::OpenaiBalanced => FilterContext {
                max_tokens: 4500,
                tokenizer_profile: TokenizerProfile::Openai,
                preserve_code: true,
                aggressive_chrome_strip: true,
                max_array_items: 6,
                max_object_keys: 16,
            },
            Self::GeminiSearchHeavy => FilterContext {
                max_tokens: 5500,
                tokenizer_profile: TokenizerProfile::Gemini,
                preserve_code: false,
                aggressive_chrome_strip: true,
                max_array_items: 8,
                max_object_keys: 18,
            },
            Self::LocalDevVerbose => FilterContext {
                max_tokens: 8000,
                tokenizer_profile: TokenizerProfile::Approx,
                preserve_code: true,
                aggressive_chrome_strip: false,
                max_array_items: 12,
                max_object_keys: 24,
            },
        }
    }
}

impl std::fmt::Display for McpPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for McpPreset {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "claude-code-strict" => Ok(Self::ClaudeCodeStrict),
            "claude-code-balanced" => Ok(Self::ClaudeCodeBalanced),
            "openai-balanced" => Ok(Self::OpenaiBalanced),
            "gemini-search-heavy" => Ok(Self::GeminiSearchHeavy),
            "local-dev-verbose" => Ok(Self::LocalDevVerbose),
            other => Err(format!("unknown MCP preset: {other}")),
        }
    }
}

pub fn preset_from_env(key: &str) -> Option<McpPreset> {
    std::env::var(key).ok()?.parse().ok()
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct McpConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<McpPreset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer_profile: Option<TokenizerProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_array_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_object_keys: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_code: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggressive_chrome_strip: Option<bool>,
}

impl McpConfig {
    pub fn apply_overrides(&self, context: &mut FilterContext) {
        if let Some(max_tokens) = self.max_tokens {
            context.max_tokens = max_tokens;
        }
        if let Some(tokenizer_profile) = self.tokenizer_profile {
            context.tokenizer_profile = tokenizer_profile;
        }
        if let Some(max_array_items) = self.max_array_items {
            context.max_array_items = max_array_items;
        }
        if let Some(max_object_keys) = self.max_object_keys {
            context.max_object_keys = max_object_keys;
        }
        if let Some(preserve_code) = self.preserve_code {
            context.preserve_code = preserve_code;
        }
        if let Some(aggressive_chrome_strip) = self.aggressive_chrome_strip {
            context.aggressive_chrome_strip = aggressive_chrome_strip;
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingConfig {
    pub enabled: bool,
    pub history_days: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_path: Option<PathBuf>,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            history_days: 90,
            database_path: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub colors: bool,
    pub emoji: bool,
    pub max_width: usize,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            colors: true,
            emoji: true,
            max_width: 120,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterConfig {
    pub ignore_dirs: Vec<String>,
    pub ignore_files: Vec<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            ignore_dirs: vec![
                ".git".into(),
                "node_modules".into(),
                "target".into(),
                "__pycache__".into(),
                ".venv".into(),
                "vendor".into(),
            ],
            ignore_files: vec!["*.lock".into(), "*.min.js".into(), "*.min.css".into()],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = get_config_path()?;

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = get_config_path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn create_default() -> Result<PathBuf> {
        let config = Config::default();
        config.save()?;
        get_config_path()
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    Ok(config_dir.join("clov").join("config.toml"))
}

pub fn show_config() -> Result<()> {
    let path = get_config_path()?;
    println!("Config: {}", path.display());
    println!();

    if path.exists() {
        let config = Config::load()?;
        println!("{}", toml::to_string_pretty(&config)?);
    } else {
        println!("(default config, file not created)");
        println!();
        let config = Config::default();
        println!("{}", toml::to_string_pretty(&config)?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_maps_to_expected_filter_context() {
        let context = McpPreset::GeminiSearchHeavy.filter_context();
        assert_eq!(context.tokenizer_profile, TokenizerProfile::Gemini);
        assert_eq!(context.max_tokens, 5500);
        assert!(!context.preserve_code);
    }

    #[test]
    fn mcp_config_overrides_selected_preset() {
        let mut context = McpPreset::ClaudeCodeStrict.filter_context();
        let config = McpConfig {
            preset: None,
            max_tokens: Some(9000),
            tokenizer_profile: Some(TokenizerProfile::GenericCode),
            max_array_items: Some(11),
            max_object_keys: Some(22),
            preserve_code: Some(false),
            aggressive_chrome_strip: Some(false),
        };
        config.apply_overrides(&mut context);

        assert_eq!(context.max_tokens, 9000);
        assert_eq!(context.tokenizer_profile, TokenizerProfile::GenericCode);
        assert_eq!(context.max_array_items, 11);
        assert_eq!(context.max_object_keys, 22);
        assert!(!context.preserve_code);
        assert!(!context.aggressive_chrome_strip);
    }
}
