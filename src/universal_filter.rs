//! Universal content filtering for MCP responses.
//!
//! Detects content type and structure automatically without hardcoded
//! tool-specific logic. Applies heuristic-based chrome stripping and
//! adaptive truncation.

use crate::tracking::estimate_tokens;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

pub struct FilterContext {
    pub max_tokens: usize,
    pub preserve_code: bool,
    pub aggressive_chrome_strip: bool,
}

impl Default for FilterContext {
    fn default() -> Self {
        Self {
            max_tokens: 2000,
            preserve_code: true,
            aggressive_chrome_strip: true,
        }
    }
}

pub fn filter_response(text: &str, context: &FilterContext) -> String {
    // Attempt to parse as JSON first
    if let Ok(mut json) = serde_json::from_str::<Value>(text) {
        let content_type = detect_content_type(&json);
        match content_type {
            ContentType::WebSearch => {
                apply_search_filters(&mut json, context);
            }
            ContentType::Code => {
                apply_code_filters(&mut json, context);
            }
            ContentType::StructuredData => {
                apply_data_filters(&mut json, context);
            }
            ContentType::PlainText => {
                apply_text_filters(&mut json, context);
            }
            ContentType::Unknown => {}
        }
        return serde_json::to_string(&json).unwrap_or_else(|_| text.to_string());
    }

    // Fallback: it's just a raw string
    let cleaned = strip_universal_chrome(text);
    let limit = adaptive_truncation_limit(&cleaned);
    filter_by_token_budget(&cleaned, context.max_tokens.min(limit / 4))
}

enum ContentType {
    WebSearch,
    Code,
    StructuredData,
    PlainText,
    Unknown,
}

fn detect_content_type(response: &Value) -> ContentType {
    // Heuristic detection without tool name matching
    if response.get("results").is_some() {
        return ContentType::WebSearch;
    }
    
    if let Some(text) = response.get("text").and_then(|t| t.as_str()) {
        if text.contains("```") || text.contains("fn ") || text.contains("class ") {
            return ContentType::Code;
        }
        return ContentType::PlainText;
    }
    
    if response.get("data").is_some() {
        return ContentType::StructuredData;
    }
    
    ContentType::Unknown
}

fn apply_search_filters(response: &mut Value, context: &FilterContext) {
    if let Some(results) = response.get_mut("results").and_then(|r| r.as_array_mut()) {
        for result in results {
            if let Some(text) = result.get_mut("text").and_then(|t| t.as_str()) {
                let cleaned = if context.aggressive_chrome_strip {
                    strip_universal_chrome(text)
                } else {
                    text.to_string()
                };
                let limit = adaptive_truncation_limit(&cleaned);
                let truncated = filter_by_token_budget(&cleaned, context.max_tokens.min(limit / 4));
                *result.get_mut("text").unwrap() = Value::String(truncated);
            }
        }
    }
}

fn apply_code_filters(response: &mut Value, context: &FilterContext) {
    if let Some(text) = response.get_mut("text").and_then(|t| t.as_str()) {
        let cleaned = if context.preserve_code {
            strip_universal_chrome(text) // Lighter strip could be applied here if needed
        } else {
            strip_universal_chrome(text)
        };
        let truncated = filter_by_token_budget(&cleaned, context.max_tokens);
        *response.get_mut("text").unwrap() = Value::String(truncated);
    }
}

fn apply_data_filters(_response: &mut Value, _context: &FilterContext) {
    // Future parsing / filtering of structured data
}

fn apply_text_filters(response: &mut Value, context: &FilterContext) {
    if let Some(text) = response.get_mut("text").and_then(|t| t.as_str()) {
        let cleaned = strip_universal_chrome(text);
        let truncated = filter_by_token_budget(&cleaned, context.max_tokens);
        *response.get_mut("text").unwrap() = Value::String(truncated);
    }
}

pub fn strip_universal_chrome(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut skip_block = false;
    let mut blank_count = 0;
    
    for line in text.lines() {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            blank_count += 1;
            skip_block = false;
            if blank_count <= 2 {
                result.push('\n');
            }
            continue;
        }
        blank_count = 0;
        
        if is_navigation_chrome(trimmed) || is_footer_garbage(trimmed) || is_advertisement(trimmed) {
            skip_block = true;
            continue;
        }
        
        if skip_block && trimmed.len() > 40 {
            skip_block = false;
        }
        
        if skip_block {
            continue;
        }
        
        result.push_str(line);
        result.push('\n');
    }
    
    collapse_whitespace(&result)
}

fn is_navigation_chrome(line: &str) -> bool {
    lazy_static! {
        static ref NAV: Vec<Regex> = vec![
            Regex::new(r"(?i)^\s*(skip to (main |)content|menu|navigation|breadcrumb|sidebar)\s*$").unwrap(),
            Regex::new(r"(?i)^\s*(sign in|log in|register|subscribe|newsletter)\s*$").unwrap(),
            Regex::new(r"(?i)^\s*\[?(home|about|contact|blog|faq|help|support)\]?\s*$").unwrap(),
            Regex::new(r"(?i)^\s*(previous|next|related (posts|articles)|you (may|might) also like)\s*$").unwrap(),
        ];
    }
    NAV.iter().any(|p| p.is_match(line.trim()))
}

fn is_footer_garbage(line: &str) -> bool {
    lazy_static! {
        static ref FOOTER: Vec<Regex> = vec![
            Regex::new(r"(?i)^\s*(cookie|privacy|terms of (use|service)|accept all|reject all|manage preferences)\s*$").unwrap(),
            Regex::new(r"(?i)^\s*(©|copyright|all rights reserved|powered by)\b").unwrap(),
            Regex::new(r"(?i)^\s*[|·•–—]\s*(privacy|terms|contact|about|sitemap|careers|press)").unwrap(),
        ];
    }
    FOOTER.iter().any(|p| p.is_match(line.trim()))
}

fn is_advertisement(line: &str) -> bool {
    lazy_static! {
        static ref ADS: Vec<Regex> = vec![
            Regex::new(r"(?i)^\s*(advertisement|sponsored|promoted|ad)\s*$").unwrap(),
        ];
    }
    ADS.iter().any(|p| p.is_match(line.trim()))
}

fn collapse_whitespace(text: &str) -> String {
    lazy_static! {
        static ref MULTIPLE_BLANKS: Regex = Regex::new(r"\n{4,}").unwrap();
        static ref MULTIPLE_SPACES: Regex = Regex::new(r"[ \t]{3,}").unwrap();
    }
    let result = MULTIPLE_BLANKS.replace_all(text, "\n\n");
    let result = MULTIPLE_SPACES.replace_all(&result, " ");
    result.trim().to_string()
}

fn adaptive_truncation_limit(text: &str) -> usize {
    let line_count = text.lines().count();
    let avg_line_length = text.len() / line_count.max(1);
    
    if avg_line_length > 80 {
        return 5000;
    }
    
    if avg_line_length > 40 {
        return 2500;
    }
    
    1500
}

fn filter_by_token_budget(text: &str, max_tokens: usize) -> String {
    let estimated = estimate_tokens(text);
    
    if estimated <= max_tokens {
        return text.to_string();
    }
    
    let mut low = 0;
    let mut high = text.len();
    
    while low < high {
        let mid = (low + high) / 2;
        // make sure mid is at a char boundary
        let mut adj_mid = mid;
        while adj_mid > 0 && !text.is_char_boundary(adj_mid) {
            adj_mid -= 1;
        }
        
        let chunk = &text[..adj_mid];
        
        if estimate_tokens(chunk) <= max_tokens {
            if low == adj_mid {
                break;
            }
            low = adj_mid;
        } else {
            high = adj_mid;
        }
    }
    
    format!("{}\n[... truncated to {} tokens by clov]", 
            &text[..low], max_tokens)
}
