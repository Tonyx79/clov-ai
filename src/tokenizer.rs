use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum TokenizerProfile {
    #[default]
    Approx,
    Claude,
    Openai,
    Gemini,
    GenericCode,
}

impl TokenizerProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approx => "approx",
            Self::Claude => "claude",
            Self::Openai => "openai",
            Self::Gemini => "gemini",
            Self::GenericCode => "generic-code",
        }
    }
}

impl fmt::Display for TokenizerProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TokenizerProfile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "approx" => Ok(Self::Approx),
            "claude" | "claude-like" => Ok(Self::Claude),
            "openai" | "openai-like" => Ok(Self::Openai),
            "gemini" | "gemini-like" => Ok(Self::Gemini),
            "generic-code" | "code" => Ok(Self::GenericCode),
            other => Err(format!("unknown tokenizer profile: {other}")),
        }
    }
}

pub trait TokenCounter {
    fn count(&self, text: &str) -> usize;
}

pub fn count_tokens(text: &str, profile: TokenizerProfile) -> usize {
    counter_for(profile).count(text)
}

pub fn profile_from_env(key: &str) -> Option<TokenizerProfile> {
    std::env::var(key).ok()?.parse().ok()
}

pub fn counter_for(profile: TokenizerProfile) -> Box<dyn TokenCounter> {
    match profile {
        TokenizerProfile::Approx => Box::new(ApproxCounter),
        TokenizerProfile::Claude => Box::new(WeightedCounter::new(0.24, 0.34, 0.06, 0.55, 0.22)),
        TokenizerProfile::Openai => Box::new(WeightedCounter::new(0.25, 0.32, 0.05, 0.52, 0.18)),
        TokenizerProfile::Gemini => Box::new(WeightedCounter::new(0.25, 0.30, 0.05, 0.50, 0.16)),
        TokenizerProfile::GenericCode => {
            Box::new(WeightedCounter::new(0.28, 0.50, 0.05, 0.60, 0.35))
        }
    }
}

struct ApproxCounter;

impl TokenCounter for ApproxCounter {
    fn count(&self, text: &str) -> usize {
        if text.is_empty() {
            0
        } else {
            (text.len() as f64 / 4.0).ceil() as usize
        }
    }
}

struct WeightedCounter {
    ascii_weight: f64,
    punctuation_weight: f64,
    whitespace_weight: f64,
    non_ascii_weight: f64,
    newline_weight: f64,
}

impl WeightedCounter {
    const fn new(
        ascii_weight: f64,
        punctuation_weight: f64,
        whitespace_weight: f64,
        non_ascii_weight: f64,
        newline_weight: f64,
    ) -> Self {
        Self {
            ascii_weight,
            punctuation_weight,
            whitespace_weight,
            non_ascii_weight,
            newline_weight,
        }
    }
}

impl TokenCounter for WeightedCounter {
    fn count(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let mut total = 0.0;
        for ch in text.chars() {
            total += if ch == '\n' || ch == '\r' {
                self.newline_weight
            } else if ch.is_ascii_whitespace() {
                self.whitespace_weight
            } else if ch.is_ascii_punctuation() {
                self.punctuation_weight
            } else if ch.is_ascii() {
                self.ascii_weight
            } else {
                self.non_ascii_weight
            };
        }

        total.ceil() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approx_profile_preserves_legacy_chars_per_token_behavior() {
        assert_eq!(count_tokens("", TokenizerProfile::Approx), 0);
        assert_eq!(count_tokens("abcd", TokenizerProfile::Approx), 1);
        assert_eq!(count_tokens("abcde", TokenizerProfile::Approx), 2);
    }

    #[test]
    fn code_profile_penalizes_symbol_heavy_source_more_than_approx() {
        let source = "fn main() {\n    let value = foo(bar::baz());\n}\n";
        assert!(
            count_tokens(source, TokenizerProfile::GenericCode)
                > count_tokens(source, TokenizerProfile::Approx)
        );
    }

    #[test]
    fn profiles_parse_from_strings() {
        assert_eq!(
            "approx".parse::<TokenizerProfile>().unwrap(),
            TokenizerProfile::Approx
        );
        assert_eq!(
            "claude".parse::<TokenizerProfile>().unwrap(),
            TokenizerProfile::Claude
        );
        assert_eq!(
            "generic-code".parse::<TokenizerProfile>().unwrap(),
            TokenizerProfile::GenericCode
        );
    }
}
