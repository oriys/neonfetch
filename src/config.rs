use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Config {
    pub style: Option<String>,
    pub speed: Option<f32>,
    pub duration: Option<f64>,
    pub color_fps: Option<u32>,
    pub no_logo: Option<bool>,
    pub no_packages: Option<bool>,
    pub no_header: Option<bool>,
    pub mono: Option<bool>,
    pub no_color: Option<bool>,
    pub seed: Option<u64>,
}

impl Config {
    pub fn load(explicit_path: Option<&str>, no_config: bool) -> Self {
        if no_config {
            return Self::default();
        }

        if let Some(path) = explicit_path.filter(|path| !path.trim().is_empty()) {
            return load_path(Path::new(path));
        }

        if let Some(path) = env::var_os("NEONFETCH_CONFIG")
            && !path.as_os_str().is_empty()
        {
            return load_path(Path::new(&path));
        }

        for path in default_config_paths() {
            if path.exists() {
                return load_path(&path);
            }
        }

        Self::default()
    }
}

fn default_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(path) = env::var_os("XDG_CONFIG_HOME")
        && !path.as_os_str().is_empty()
    {
        paths.push(PathBuf::from(path).join("neonfetch").join("config.toml"));
    }
    if let Some(path) = env::var_os("HOME")
        && !path.as_os_str().is_empty()
    {
        paths.push(
            PathBuf::from(path)
                .join(".config")
                .join("neonfetch")
                .join("config.toml"),
        );
    }
    paths
}

fn load_path(path: &Path) -> Config {
    match fs::read_to_string(path) {
        Ok(contents) => match parse_config(&contents) {
            Ok(config) => config,
            Err(err) => {
                warn_ignored(path, &err);
                Config::default()
            }
        },
        Err(err) if err.kind() == ErrorKind::NotFound => Config::default(),
        Err(err) => {
            warn_ignored(path, &err.to_string());
            Config::default()
        }
    }
}

fn warn_ignored(path: &Path, reason: &str) {
    eprintln!(
        "neonfetch: warning: ignoring config {}: {}",
        path.display(),
        reason
    );
}

fn parse_config(contents: &str) -> Result<Config, String> {
    let mut config = Config::default();

    for (line_index, raw_line) in contents.lines().enumerate() {
        let line_number = line_index + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        let (raw_key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {line_number}: expected key = value"))?;
        let key = normalize_key(raw_key.trim());
        if key.is_empty() {
            return Err(format!("line {line_number}: expected a key"));
        }

        let value =
            parse_value(raw_value.trim()).map_err(|err| format!("line {line_number}: {err}"))?;

        match key.as_str() {
            "style" => config.style = Some(value.into_string("style")?),
            "speed" => config.speed = Some(value.into_f64("speed")? as f32),
            "duration" => config.duration = Some(value.into_f64("duration")?),
            "color_fps" => config.color_fps = Some(value.into_u32("color_fps")?),
            "no_logo" => config.no_logo = Some(value.into_bool("no_logo")?),
            "no_packages" => config.no_packages = Some(value.into_bool("no_packages")?),
            "no_header" => config.no_header = Some(value.into_bool("no_header")?),
            "mono" => config.mono = Some(value.into_bool("mono")?),
            "no_color" => config.no_color = Some(value.into_bool("no_color")?),
            "seed" => config.seed = Some(value.into_u64("seed")?),
            _ => return Err(format!("line {line_number}: unknown key `{key}`")),
        }
    }

    Ok(config)
}

fn normalize_key(key: &str) -> String {
    key.replace('-', "_")
}

fn strip_comment(line: &str) -> &str {
    let mut in_quote = false;
    let mut escaped = false;

    for (index, ch) in line.char_indices() {
        if in_quote {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_quote = false,
                _ => {}
            }
        } else if ch == '"' {
            in_quote = true;
        } else if ch == '#' {
            return &line[..index];
        }
    }

    line
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    String(String),
    Bool(bool),
    Integer(i128),
    Float(f64),
}

impl Value {
    fn into_string(self, key: &str) -> Result<String, String> {
        match self {
            Value::String(value) => Ok(value),
            _ => Err(format!("key `{key}` expects a string")),
        }
    }

    fn into_bool(self, key: &str) -> Result<bool, String> {
        match self {
            Value::Bool(value) => Ok(value),
            _ => Err(format!("key `{key}` expects true or false")),
        }
    }

    fn into_f64(self, key: &str) -> Result<f64, String> {
        let value = match self {
            Value::Integer(value) => value as f64,
            Value::Float(value) => value,
            _ => return Err(format!("key `{key}` expects a number")),
        };
        if value.is_finite() {
            Ok(value)
        } else {
            Err(format!("key `{key}` expects a finite number"))
        }
    }

    fn into_u32(self, key: &str) -> Result<u32, String> {
        let value = self.into_integer(key)?;
        u32::try_from(value).map_err(|_| format!("key `{key}` expects a non-negative integer"))
    }

    fn into_u64(self, key: &str) -> Result<u64, String> {
        let value = self.into_integer(key)?;
        u64::try_from(value).map_err(|_| format!("key `{key}` expects a non-negative integer"))
    }

    fn into_integer(self, key: &str) -> Result<i128, String> {
        match self {
            Value::Integer(value) => Ok(value),
            Value::Float(value) if value.is_finite() && value.fract() == 0.0 => Ok(value as i128),
            _ => Err(format!("key `{key}` expects an integer")),
        }
    }
}

fn parse_value(raw: &str) -> Result<Value, String> {
    if raw.is_empty() {
        return Err("expected a value".to_string());
    }

    if raw.starts_with('"') {
        return parse_quoted_string(raw).map(Value::String);
    }

    if raw.eq_ignore_ascii_case("true") {
        return Ok(Value::Bool(true));
    }
    if raw.eq_ignore_ascii_case("false") {
        return Ok(Value::Bool(false));
    }

    if looks_like_number(raw) {
        if raw.contains('.') || raw.contains('e') || raw.contains('E') {
            return raw
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| format!("invalid number `{raw}`"));
        }
        return raw
            .parse::<i128>()
            .map(Value::Integer)
            .map_err(|_| format!("invalid integer `{raw}`"));
    }

    if raw.chars().any(char::is_whitespace) {
        return Err("unquoted strings cannot contain whitespace".to_string());
    }

    Ok(Value::String(raw.to_string()))
}

fn parse_quoted_string(raw: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut escaped = false;

    for (index, ch) in raw.char_indices().skip(1) {
        if escaped {
            match ch {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                _ => return Err(format!("unsupported escape sequence `\\{ch}`")),
            }
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => {
                let rest = &raw[index + ch.len_utf8()..];
                if rest.trim().is_empty() {
                    return Ok(output);
                }
                return Err("unexpected text after quoted string".to_string());
            }
            _ => output.push(ch),
        }
    }

    if escaped {
        Err("unfinished escape sequence".to_string())
    } else {
        Err("unterminated quoted string".to_string())
    }
}

fn looks_like_number(raw: &str) -> bool {
    let body = raw
        .strip_prefix('+')
        .or_else(|| raw.strip_prefix('-'))
        .unwrap_or(raw);

    body.chars().next().is_some_and(|ch| ch.is_ascii_digit())
        && raw
            .chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, '+' | '-' | '.' | 'e' | 'E'))
}
