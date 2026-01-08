use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_fallback_raw")]
    pub fallback_raw: bool,
    #[serde(default = "default_delay_ms")]
    pub default_delay_ms: u64,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fallback_raw: default_fallback_raw(),
            default_delay_ms: default_delay_ms(),
            rules: Vec::new(),
        }
    }
}

fn default_fallback_raw() -> bool {
    true
}

fn default_delay_ms() -> u64 {
    1000
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub app_name: Option<String>,
    pub pattern: Option<String>,
    pub regex: Option<String>,
    pub template: Option<String>,
    pub templates: Option<Vec<String>>,
    pub delay: Option<u64>,
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

    let template = r#"# arg-shim 配置文件模板

# 如果没有匹配到规则，是否将原始参数复制到剪贴板？(默认: true)
# fallback_raw = true

# 使用多模板复制时的默认间隔延迟（毫秒）(默认: 1000)
# default_delay_ms = 1000

[[rules]]
# 可选：规则名称
name = "示例：Putty 转 SSH"

# 可选：要匹配的可执行文件名称（区分大小写）
# app_name = "putty"

# 策略 A：简单模式匹配（推荐）
# 从命令行参数中捕获 {user}, {host}, {port}
# 模式中的空格匹配任意长度的空白字符
pattern = "-ssh {user}@{host} -P {port}"

# 策略 B：正则表达式（高级）
# regex = '''^--target\s+(?P<host>[a-zA-Z0-9.-]+)(\s+--port\s+(?P<port>\d+))?'''

# 输出模板
# 可以使用 'template' (单个字符串) 或 'templates' (字符串数组)
# {{user}} 引用捕获的变量
# {{1}}, {{2}}... 引用原始参数的位置索引（0=程序名，1=第一个参数）
# {{port | 22}} 如果 {port} 未捕获或为空，则使用默认值 22
template = "ssh -p {{port | 22}} {{user}}@{{host}}"

# 多步剪贴板示例：
# templates = [
#    "{{password}}",       # 先复制（将存入剪贴板历史，Win+V 查看）
#    "ssh {{user}}@{host}" # 后复制（将成为当前活动内容，Ctrl+V 粘贴）
# ]
# delay = 500 # 覆盖此规则的全局延迟设置

# [[rules]]
# name = "示例：简单的位置参数"
# template = "echo 用户是 {{1}}, 主机是 {{2}}"
"#;

    fs::write(&target, template)?;
    println!("Successfully created 'arg-shim.toml' in the current directory.");
    Ok(())
}
