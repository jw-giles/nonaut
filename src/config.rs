use std::{fs, io::ErrorKind, path::PathBuf};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;
use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderKind {
    Codex,
}

impl ProviderKind {
    pub const ALL: [Self; 1] = [Self::Codex];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Codex => "codex",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Codex => "Codex / ChatGPT",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Codex => "Existing Codex account · tools isolated by nono",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "codex" => Ok(Self::Codex),
            value => bail!("unknown saved provider `{value}`"),
        }
    }
}

pub fn load_default_provider() -> Result<Option<ProviderKind>> {
    let path = provider_path()?;
    match fs::read_to_string(&path) {
        Ok(value) => ProviderKind::parse(&value).map(Some),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("could not read {}", path.display())),
    }
}

pub fn save_default_provider(provider: ProviderKind) -> Result<()> {
    write_config(provider_path()?, provider.id())
}

pub fn load_model(provider: ProviderKind) -> Result<Option<String>> {
    let path = model_path(provider)?;
    match fs::read_to_string(&path) {
        Ok(value) => {
            let model = value.trim();
            if model.is_empty() {
                Ok(None)
            } else {
                Ok(Some(model.to_owned()))
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("could not read {}", path.display())),
    }
}

pub fn save_model(provider: ProviderKind, model: &str) -> Result<()> {
    write_config(model_path(provider)?, model)
}

pub fn load_reasoning(provider: ProviderKind, model: &str) -> Result<Option<String>> {
    let path = reasoning_path(provider)?;
    let values = match fs::read_to_string(&path) {
        Ok(value) => serde_json::from_str::<Map<String, Value>>(&value)
            .with_context(|| format!("could not parse {}", path.display()))?,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("could not read {}", path.display()));
        }
    };
    Ok(values.get(model).and_then(Value::as_str).map(str::to_owned))
}

pub fn save_reasoning(provider: ProviderKind, model: &str, effort: &str) -> Result<()> {
    let path = reasoning_path(provider)?;
    let mut values = match fs::read_to_string(&path) {
        Ok(value) => serde_json::from_str::<Map<String, Value>>(&value)
            .with_context(|| format!("could not parse {}", path.display()))?,
        Err(error) if error.kind() == ErrorKind::NotFound => Map::new(),
        Err(error) => {
            return Err(error).with_context(|| format!("could not read {}", path.display()));
        }
    };
    values.insert(model.to_owned(), Value::String(effort.to_owned()));
    let encoded =
        serde_json::to_string_pretty(&values).context("could not encode reasoning settings")?;
    write_config(path, &encoded)
}

fn write_config(path: PathBuf, value: &str) -> Result<()> {
    let directory = path
        .parent()
        .context("provider configuration path has no parent")?;
    fs::create_dir_all(directory)
        .with_context(|| format!("could not create {}", directory.display()))?;
    fs::write(&path, format!("{}\n", value.trim()))
        .with_context(|| format!("could not save {}", path.display()))
}

fn provider_path() -> Result<PathBuf> {
    let project = ProjectDirs::from("ai", "nolabs", "whoa")
        .context("could not determine the Whoa configuration directory")?;
    Ok(project.config_dir().join("provider"))
}

fn model_path(provider: ProviderKind) -> Result<PathBuf> {
    let project = ProjectDirs::from("ai", "nolabs", "whoa")
        .context("could not determine the Whoa configuration directory")?;
    Ok(project
        .config_dir()
        .join(format!("model-{}", provider.id())))
}

fn reasoning_path(provider: ProviderKind) -> Result<PathBuf> {
    let project = ProjectDirs::from("ai", "nolabs", "whoa")
        .context("could not determine the Whoa configuration directory")?;
    Ok(project
        .config_dir()
        .join(format!("reasoning-{}.json", provider.id())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_provider() {
        assert_eq!(ProviderKind::parse("codex\n").unwrap(), ProviderKind::Codex);
    }

    #[test]
    fn rejects_unknown_provider() {
        assert!(ProviderKind::parse("surprise").is_err());
    }
}
