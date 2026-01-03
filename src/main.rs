use arboard::Clipboard;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
struct Config {
    #[serde(default = "default_fallback_raw")]
    fallback_raw: bool,
    #[serde(default)]
    rules: Vec<Rule>,
}

fn default_fallback_raw() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct Rule {
    #[allow(dead_code)]
    name: Option<String>,
    app_name: Option<String>,
    pattern: Option<String>,
    regex: Option<String>,
    template: String,
}

struct Context<'a> {
    named: HashMap<String, String>,
    positional: &'a [String],
    exe_name: &'a str,
    raw_args: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("arg-shim"));
    let exe_name = exe_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("arg-shim");
    
    // Remove extension for matching if desired, or keep it. Let's keep it for precision.
    let app_identity = exe_name.to_lowercase();

    // 1. Load Config
    let config = load_config(exe_name).unwrap_or_default();

    let raw_args = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        String::new()
    };

    let mut context = Context {
        named: HashMap::new(),
        positional: &args,
        exe_name,
        raw_args: raw_args.clone(),
    };

    let mut result: Option<String> = None;

    for rule in &config.rules {
        // 1. Match app_name
        if let Some(ref target_app) = rule.app_name {
            if !app_identity.contains(&target_app.to_lowercase()) {
                continue;
            }
        }

        // 2. Try match and extract
        if let Some(ref pattern) = rule.pattern {
            let re_str = pattern_to_regex(pattern);
            if let Some(caps) = extract_variables(&re_str, &raw_args) {
                context.named = caps;
                result = Some(render_template(&rule.template, &context));
                break;
            }
        } else if let Some(ref re_str) = rule.regex {
            if let Some(caps) = extract_variables(re_str, &raw_args) {
                context.named = caps;
                result = Some(render_template(&rule.template, &context));
                break;
            }
        } else {
            // No pattern/regex means it matches anything (if app_name matched)
            result = Some(render_template(&rule.template, &context));
            break;
        }
    }

    // Fallback logic
    let final_text = result.unwrap_or_else(|| {
        if config.fallback_raw {
            raw_args
        } else {
            String::new()
        }
    });

    if !final_text.is_empty() {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(final_text).unwrap();
    }
}

fn load_config(exe_name: &str) -> Option<Config> {
    let mut config_paths = Vec::new();

    if let Ok(env_path) = env::var("ARG_SHIM_CONFIG") {
        config_paths.push(PathBuf::from(env_path));
    }

    if let Ok(current_dir) = env::current_dir() {
        config_paths.push(current_dir.join(format!("{}.arg-shim.toml", exe_name)));
        config_paths.push(current_dir.join(format!("{}.toml", exe_name)));
        config_paths.push(current_dir.join("arg-shim.toml"));
    }

    if let Ok(app_data) = env::var("APPDATA") {
        config_paths.push(PathBuf::from(app_data).join("arg-shim").join("config.toml"));
    }

    for path in config_paths {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = toml::from_str(&content) {
                    return Some(config);
                }
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
    let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}").unwrap();
    
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
            if let Some(val) = context.positional.get(idx) {
                return val.clone();
            }
        }

        // 3. Try built-ins
        match key {
            "RAW_ARGS" => return context.raw_args.clone(),
            "EXE_NAME" => return context.exe_name.to_string(),
            _ => {}
        }

        // 4. Return default if provided, else empty
        default.map(|s| s.to_string()).unwrap_or_default()
    }).to_string()
}
