//! Optional sandbox policy: extra read-only paths and an HTTPS host allowlist.
//!
//! The file lives in nonaut's config directory, never in the workspace: the
//! sandbox grants the agent write access to the workspace, so a policy stored
//! there could be rewritten by the agent to widen its own sandbox.
//!
//! ```json
//! {
//!   "allow_hosts": ["crates.io", "*.crates.io"],
//!   "read_paths": ["~/.cargo/registry"],
//!   "workspaces": {
//!     "/home/me/project": { "allow_hosts": ["registry.npmjs.org"] }
//!   }
//! }
//! ```
//!
//! Top-level entries apply everywhere; a `workspaces` entry whose path equals
//! the workspace adds to them. A missing file means today's default: no
//! network and no extra paths.

use std::{
    collections::HashMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use directories::BaseDirs;
use nono::net_filter::validate_host_pattern;
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    #[serde(default)]
    allow_hosts: Vec<String>,
    #[serde(default)]
    read_paths: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    allow_hosts: Vec<String>,
    #[serde(default)]
    read_paths: Vec<String>,
    #[serde(default)]
    workspaces: HashMap<PathBuf, Rules>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Policy {
    pub allow_hosts: Vec<String>,
    pub read_paths: Vec<PathBuf>,
}

pub(crate) fn path() -> Result<PathBuf> {
    Ok(crate::config::config_dir()?.join("policy.json"))
}

pub(crate) fn load(workspace: &Path) -> Result<Policy> {
    let path = path()?;
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Policy::default()),
        Err(error) => {
            return Err(error).with_context(|| format!("could not read {}", path.display()));
        }
    };
    let home = BaseDirs::new()
        .context("could not determine the home directory")?
        .home_dir()
        .to_path_buf();
    parse(&text, workspace, &home).with_context(|| format!("invalid {}", path.display()))
}

fn parse(text: &str, workspace: &Path, home: &Path) -> Result<Policy> {
    let file: File = serde_json::from_str(text)?;
    let workspace = fs::canonicalize(workspace).unwrap_or_else(|_| workspace.to_path_buf());
    let mut rules = Rules {
        allow_hosts: file.allow_hosts,
        read_paths: file.read_paths,
    };
    if let Some(extra) = file.workspaces.iter().find_map(|(path, rules)| {
        let path = expand_home(&path.to_string_lossy(), home);
        (fs::canonicalize(&path).unwrap_or(path) == workspace).then_some(rules)
    }) {
        rules.allow_hosts.extend(extra.allow_hosts.iter().cloned());
        rules.read_paths.extend(extra.read_paths.iter().cloned());
    }

    for host in &rules.allow_hosts {
        if host == "*" {
            bail!("allow_hosts may not contain \"*\"; list the hosts the sandbox needs");
        }
        validate_host_pattern(host).map_err(anyhow::Error::msg)?;
    }
    let read_paths = rules
        .read_paths
        .iter()
        .map(|raw| {
            let path = expand_home(raw, home);
            if path.is_absolute() {
                Ok(path)
            } else {
                bail!("read_paths entry `{raw}` must be absolute or start with ~/")
            }
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Policy {
        allow_hosts: rules.allow_hosts,
        read_paths,
    })
}

fn expand_home(raw: &str, home: &Path) -> PathBuf {
    match raw.strip_prefix("~/") {
        Some(rest) => home.join(rest),
        None if raw == "~" => home.to_path_buf(),
        None => PathBuf::from(raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_matching_workspace_rules_and_expands_home() {
        let policy = parse(
            r#"{
                "allow_hosts": ["crates.io"],
                "read_paths": ["~/.cargo/registry"],
                "workspaces": {
                    "/work/app": { "allow_hosts": ["*.npmjs.org"] },
                    "/work/other": { "allow_hosts": ["example.com"] }
                }
            }"#,
            Path::new("/work/app"),
            Path::new("/home/me"),
        )
        .unwrap();
        assert_eq!(policy.allow_hosts, ["crates.io", "*.npmjs.org"]);
        assert_eq!(
            policy.read_paths,
            [PathBuf::from("/home/me/.cargo/registry")]
        );
    }

    #[test]
    fn rejects_allow_all_bad_patterns_relative_paths_and_typos() {
        let home = Path::new("/home/me");
        let workspace = Path::new("/work");
        assert!(parse(r#"{"allow_hosts": ["*"]}"#, workspace, home).is_err());
        assert!(parse(r#"{"allow_hosts": ["foo*bar.com"]}"#, workspace, home).is_err());
        assert!(parse(r#"{"read_paths": ["relative/dir"]}"#, workspace, home).is_err());
        assert!(parse(r#"{"allow_host": ["crates.io"]}"#, workspace, home).is_err());
        assert_eq!(parse("{}", workspace, home).unwrap(), Policy::default());
    }
}
