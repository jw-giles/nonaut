use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde_json::{Map, Value};

use crate::providers::ProviderKind;

pub fn load_default_provider() -> Result<Option<ProviderKind>> {
    let path = provider_path()?;
    let legacy = legacy_path("provider")?;
    match read_config(&path, &legacy)? {
        Some((_, value)) => ProviderKind::parse(&value).map(Some),
        None => Ok(None),
    }
}

pub fn save_default_provider(provider: ProviderKind) -> Result<()> {
    write_config(provider_path()?, provider.id())
}

pub fn load_model(provider: ProviderKind) -> Result<Option<String>> {
    let path = model_path(provider)?;
    let legacy = legacy_path(&format!("model-{}", provider.id()))?;
    match read_config(&path, &legacy)? {
        Some((_, value)) => {
            let model = value.trim();
            if model.is_empty() {
                Ok(None)
            } else {
                Ok(Some(model.to_owned()))
            }
        }
        None => Ok(None),
    }
}

pub fn save_model(provider: ProviderKind, model: &str) -> Result<()> {
    write_config(model_path(provider)?, model)
}

pub fn load_reasoning(provider: ProviderKind, model: &str) -> Result<Option<String>> {
    let path = reasoning_path(provider)?;
    let legacy = legacy_path(&format!("reasoning-{}.json", provider.id()))?;
    let values = match read_config(&path, &legacy)? {
        Some((source, value)) => serde_json::from_str::<Map<String, Value>>(&value)
            .with_context(|| format!("could not parse {}", source.display()))?,
        None => return Ok(None),
    };
    Ok(values.get(model).and_then(Value::as_str).map(str::to_owned))
}

pub fn save_reasoning(provider: ProviderKind, model: &str, effort: &str) -> Result<()> {
    let path = reasoning_path(provider)?;
    let legacy = legacy_path(&format!("reasoning-{}.json", provider.id()))?;
    let mut values = match read_config(&path, &legacy)? {
        Some((source, value)) => serde_json::from_str::<Map<String, Value>>(&value)
            .with_context(|| format!("could not parse {}", source.display()))?,
        None => Map::new(),
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

fn read_config(path: &Path, legacy: &Path) -> Result<Option<(PathBuf, String)>> {
    match fs::read_to_string(path) {
        Ok(value) => Ok(Some((path.to_path_buf(), value))),
        Err(error) if error.kind() == ErrorKind::NotFound => match fs::read_to_string(legacy) {
            Ok(value) => Ok(Some((legacy.to_path_buf(), value))),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => {
                Err(error).with_context(|| format!("could not read {}", legacy.display()))
            }
        },
        Err(error) => Err(error).with_context(|| format!("could not read {}", path.display())),
    }
}

fn provider_path() -> Result<PathBuf> {
    let project = ProjectDirs::from("ai", "nolabs", "nonaut")
        .context("could not determine the Nonaut configuration directory")?;
    Ok(project.config_dir().join("provider"))
}

fn model_path(provider: ProviderKind) -> Result<PathBuf> {
    let project = ProjectDirs::from("ai", "nolabs", "nonaut")
        .context("could not determine the Nonaut configuration directory")?;
    Ok(project
        .config_dir()
        .join(format!("model-{}", provider.id())))
}

fn reasoning_path(provider: ProviderKind) -> Result<PathBuf> {
    let project = ProjectDirs::from("ai", "nolabs", "nonaut")
        .context("could not determine the Nonaut configuration directory")?;
    Ok(project
        .config_dir()
        .join(format!("reasoning-{}.json", provider.id())))
}

fn legacy_path(name: &str) -> Result<PathBuf> {
    let project = ProjectDirs::from("ai", "nolabs", "whoa")
        .context("could not determine the legacy configuration directory")?;
    Ok(project.config_dir().join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_provider() {
        assert_eq!(ProviderKind::parse("codex\n").unwrap(), ProviderKind::Codex);
        assert_eq!(ProviderKind::parse("claude").unwrap(), ProviderKind::Claude);
    }

    #[test]
    fn rejects_unknown_provider() {
        assert!(ProviderKind::parse("surprise").is_err());
    }
}
