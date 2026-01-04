use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_fallback_raw")]
    pub fallback_raw: bool,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fallback_raw: default_fallback_raw(),
            rules: Vec::new(),
        }
    }
}

fn default_fallback_raw() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub app_name: Option<String>,
    pub pattern: Option<String>,
    pub regex: Option<String>,
    pub template: String,
}

pub fn load(exe_name: &str) -> Option<Config> {
    let mut config_paths = Vec::new();

    // 1. Environment Variable
    if let Ok(env_path) = env::var("ARG_SHIM_CONFIG") {
        config_paths.push(PathBuf::from(env_path));
    }

    // 2. Current Working Directory (Only if running as arg-shim)
    // This is useful for testing/debugging, but we don't want arbitrary CWD configs
    // being picked up when shimmed as another tool.
    let is_arg_shim = exe_name == "arg-shim" || exe_name == "arg-shim.exe";
    
    if is_arg_shim {
        if let Ok(current_dir) = env::current_dir() {
            add_search_paths(&mut config_paths, &current_dir, exe_name);
        }
    }

    // 3. Executable Directory
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            add_search_paths(&mut config_paths, exe_dir, exe_name);
        }
    }

    // 4. AppData
    if let Ok(app_data) = env::var("APPDATA") {
        config_paths.push(PathBuf::from(app_data).join("arg-shim").join("config.toml"));
    }

    // Try to load the first existing one
    for path in config_paths {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&content) {
                    return Some(config);
                }
            }
        }
    }
    None
}

fn add_search_paths(paths: &mut Vec<PathBuf>, base_dir: &Path, exe_name: &str) {
    // 1. Try exact match (e.g. putty.exe.toml)
    paths.push(base_dir.join(format!("{}.arg-shim.toml", exe_name)));
    paths.push(base_dir.join(format!("{}.toml", exe_name)));

    // 2. Try stem match (e.g. putty.toml) if exe_name has an extension
    let path = Path::new(exe_name);
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if stem != exe_name {
            paths.push(base_dir.join(format!("{}.arg-shim.toml", stem)));
            paths.push(base_dir.join(format!("{}.toml", stem)));
        }
    }

    // 3. Generic config
    paths.push(base_dir.join("arg-shim.toml"));
}

pub fn create_default_config() -> std::io::Result<()> {
    let target = PathBuf::from("arg-shim.toml");
    if target.exists() {
        println!("Error: 'arg-shim.toml' already exists in the current directory.");
        return Ok(());
    }

    let template = r#"# arg-shim configuration template

# If no rules match, should we copy the raw arguments to the clipboard? (Default: true)
# fallback_raw = true

[[rules]]
# Optional: name of the rule
name = "Example: Putty to SSH"

# Optional: executable name to match (case-sensitive)
# app_name = "putty"

# Strategy A: Simple Pattern (Recommended)
# Captures {user}, {host}, {port} from the command line.
# Spaces in the pattern match any amount of whitespace.
pattern = "-ssh {user}@{host} -P {port}"

# Strategy B: Regex (Advanced)
# regex = '''^--target\s+(?P<host>[a-zA-Z0-9.-]+)(\s+--port\s+(?P<port>\d+))?'''

# Output Template
# {{user}} refers to the captured variable.
# {{1}}, {{2}}... refers to the original argument index.
# {{port | 22}} provides a default value of 22 if {port} is not captured.
template = "ssh -p {{port | 22}} {{user}}@{{host}}"

# [[rules]]
# name = "Example: Simple Positional"
# template = "echo User is {{1}}, Host is {{2}}"
"#;

    fs::write(&target, template)?;
    println!("Successfully created 'arg-shim.toml' in the current directory.");
    Ok(())
}
