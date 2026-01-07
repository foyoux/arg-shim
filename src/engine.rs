use regex::Regex;
use std::collections::HashMap;

use crate::config::Rule;

pub struct Context<'a> {
    pub named: HashMap<String, String>,
    pub positional: &'a [String],
    pub exe_name: &'a str,
    pub raw_args: String,
}

pub fn process(rules: &[Rule], context: &mut Context) -> Option<(Vec<String>, Option<u64>)> {
    for rule in rules {
        // 1. Match app_name (Case-sensitive)
        if let Some(ref target_app) = rule.app_name {
            if !context.exe_name.contains(target_app) {
                continue;
            }
        }

        // 2. Try match and extract
        let matched = if let Some(ref pattern) = rule.pattern {
            let re_str = pattern_to_regex(pattern);
            extract_variables(&re_str, &context.raw_args)
        } else if let Some(ref re_str) = rule.regex {
            extract_variables(re_str, &context.raw_args)
        } else {
            // No pattern/regex means it matches anything (if app_name matched)
            Some(HashMap::new())
        };

        if let Some(caps) = matched {
            context.named = caps;
            
            // Collect templates
            let mut results = Vec::new();
            
            // Priority: 'templates' array > 'template' string
            if let Some(ref list) = rule.templates {
                for t in list {
                    results.push(render_template(t, context));
                }
            } else if let Some(ref t) = rule.template {
                results.push(render_template(t, context));
            }
            
            if !results.is_empty() {
                return Some((results, rule.delay));
            }
        }
    }
    None
}

/// Converts simple pattern like "-ssh {user}@{host}" to regex
fn pattern_to_regex(pattern: &str) -> String {
    let mut regex_parts = Vec::new();
    let mut last_end = 0;
    
    // Match {var_name}
    // We use unwrap here because the regex is constant and valid
    let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}").expect("Invalid internal regex");
    
    for cap in re.captures_iter(pattern) {
        let match_start = cap.get(0).unwrap().start();
        let match_end = cap.get(0).unwrap().end();
        let var_name = cap.get(1).unwrap().as_str();

        // Escape text before the variable
        let text_before = &pattern[last_end..match_start];
        if !text_before.is_empty() {
            let escaped = regex::escape(text_before);
            // Allow flexible whitespace where spaces were
            let replaced = escaped.replace(" ", r"\s+");
            regex_parts.push(replaced);
        }

        // Add named capture group
        regex_parts.push(format!(r"(?P<{}>\S+)", var_name));
        
        last_end = match_end;
    }

    // Handle remaining text
    if last_end < pattern.len() {
        let text_after = &pattern[last_end..];
        let escaped = regex::escape(text_after);
        regex_parts.push(escaped.replace(" ", r"\s+"));
    }

    regex_parts.join("")
}

fn extract_variables(re_str: &str, input: &str) -> Option<HashMap<String, String>> {
    let re = Regex::new(re_str).ok()?;
    if let Some(caps) = re.captures(input) {
        let mut map = HashMap::new();
        for name in re.capture_names().flatten() {
            if let Some(m) = caps.name(name) {
                map.insert(name.to_string(), m.as_str().to_string());
            }
        }
        Some(map)
    } else {
        None
    }
}

fn render_template(template: &str, context: &Context) -> String {
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_|\-.\s]+)\s*\}\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        let parts: Vec<&str> = caps[1].split('|').map(|s| s.trim()).collect();
        let key = parts[0];
        let default = parts.get(1);

        // 1. Try named variables
        if let Some(val) = context.named.get(key) {
            if !val.is_empty() { return val.clone(); }
        }

        // 2. Try positional arguments
        if let Ok(idx) = key.parse::<usize>() {
            if idx == 0 {
                return context.exe_name.to_string();
            }
            if let Some(val) = context.positional.get(idx - 1) {
                return val.clone();
            }
        }

        // 3. Try built-ins & Environment variables
        match key {
            "RAW_ARGS" => return context.raw_args.clone(),
            "EXE_NAME" => return context.exe_name.to_string(),
            "CWD" => return std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
            k if k.starts_with("ENV:") => {
                let env_key = &k[4..];
                return std::env::var(env_key).unwrap_or_default();
            },
            _ => {}
        }

        // 4. Return default if provided, else empty
        default.map(|s| s.to_string()).unwrap_or_default()
    }).to_string()
}
