//! Workspace snapshots for `/undo`, built on nono's content-addressed store.
//!
//! A snapshot is taken before every agent turn and `/exec`, and `/undo`
//! restores the newest one. Snapshots live in nonaut's state directory,
//! outside the workspace and `/tmp`, so sandboxed tools cannot alter them.
//! `.git`, build output and `.gitignore`d files are not tracked, so they are
//! neither captured nor touched by a restore.

use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use nono::undo::{
    Change, ExclusionConfig, ExclusionFilter, SnapshotManager, SnapshotManifest, WalkBudget,
};

const UNTRACKED: [&str; 3] = [".git", "target", "node_modules"];

pub(crate) struct Undo {
    manager: SnapshotManager,
    session_dir: PathBuf,
    stack: Vec<SnapshotManifest>,
    /// Why the last checkpoint failed, reported when `/undo` finds nothing.
    last_error: Option<String>,
}

impl Undo {
    pub(crate) fn open(workspace: &Path) -> Result<Self> {
        Self::open_in(workspace, &state_dir()?)
    }

    fn open_in(workspace: &Path, store: &Path) -> Result<Self> {
        let session_dir = store.join(format!("{}-{}", std::process::id(), unix_nanos()));
        fs::create_dir_all(&session_dir)
            .with_context(|| format!("could not create {}", session_dir.display()))?;
        fs::set_permissions(&session_dir, fs::Permissions::from_mode(0o700))?;
        let filter = ExclusionFilter::new(
            ExclusionConfig {
                exclude_patterns: UNTRACKED.iter().map(|name| (*name).to_owned()).collect(),
                ..ExclusionConfig::default()
            },
            workspace,
        )
        .context("could not read the workspace ignore rules")?;
        let manager = SnapshotManager::new(
            session_dir.clone(),
            vec![workspace.to_path_buf()],
            filter,
            WalkBudget::default(),
        )
        .context("could not start the undo store")?;
        Ok(Self {
            manager,
            session_dir,
            stack: Vec::new(),
            last_error: None,
        })
    }

    /// Records the workspace as it is now. A failure only disables undo for
    /// the coming turn; it never blocks the turn itself.
    pub(crate) fn checkpoint(&mut self) {
        let result = match self.stack.last() {
            Some(previous) => self
                .manager
                .create_incremental(previous)
                .map(|(manifest, _)| manifest),
            None => self.manager.create_baseline(),
        };
        match result {
            Ok(manifest) => {
                self.stack.push(manifest);
                self.last_error = None;
            }
            Err(error) => self.last_error = Some(error.to_string()),
        }
    }

    /// Restores the newest snapshot. `Ok(None)` means there is nothing to undo.
    pub(crate) fn undo(&mut self) -> Result<Option<Vec<Change>>> {
        let Some(manifest) = self.stack.pop() else {
            return match &self.last_error {
                Some(error) => anyhow::bail!("the last snapshot failed: {error}"),
                None => Ok(None),
            };
        };
        self.manager
            .restore_to(&manifest)
            .map(Some)
            .context("could not restore the snapshot")
    }
}

impl Drop for Undo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.session_dir);
    }
}

fn state_dir() -> Result<PathBuf> {
    let state_home = match env::var_os("XDG_STATE_HOME") {
        Some(directory) => PathBuf::from(directory),
        None => env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(".local/state"))
            .context("HOME is not set; cannot place undo snapshots")?,
    };
    Ok(state_home.join("nonaut/undo"))
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("nonaut-undo-{name}-{}", unix_nanos()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn undo_reverts_edits_creations_and_deletions_but_not_untracked_dirs() {
        let workspace = temp("ws");
        fs::write(workspace.join("kept.txt"), "original").unwrap();
        fs::write(workspace.join("deleted.txt"), "will be deleted").unwrap();
        fs::create_dir_all(workspace.join("target")).unwrap();

        let mut undo = Undo::open_in(&workspace, &temp("store")).unwrap();
        undo.checkpoint();

        fs::write(workspace.join("kept.txt"), "changed by the agent").unwrap();
        fs::remove_file(workspace.join("deleted.txt")).unwrap();
        fs::write(workspace.join("created.txt"), "new").unwrap();
        fs::write(workspace.join("target/build.bin"), "build output").unwrap();

        let changes = undo.undo().unwrap().unwrap();
        assert_eq!(changes.len(), 3);
        assert_eq!(
            fs::read_to_string(workspace.join("kept.txt")).unwrap(),
            "original"
        );
        assert!(workspace.join("deleted.txt").exists());
        assert!(!workspace.join("created.txt").exists());
        assert!(workspace.join("target/build.bin").exists());
        assert!(undo.undo().unwrap().is_none());
        fs::remove_dir_all(&workspace).unwrap();
    }
}
