mod claude;
mod codex;

use std::path::Path;

use anyhow::Result;
use tokio::sync::mpsc;

use claude::Claude;
use codex::Codex;

pub(crate) const BASE_INSTRUCTIONS: &str = "You are the coding model inside Nonaut, a security-first agent harness. Use the provided tools to inspect and modify files and verify your work. Prefer the dedicated read_file, write_file, edit_file, and delete_file tools over run_command.";

pub(crate) const DEVELOPER_INSTRUCTIONS: &str = "Tool integrity rule: never state or imply that a file was created, updated, patched, or deleted unless write_file, edit_file, or delete_file returned success in this turn. For creation or full replacement, call write_file immediately without preflight reads. For a focused edit, read the file at most once and then call edit_file. For deletion, call delete_file. Never replace a direct file mutation with run_command. Security boundary rule: you never decide whether an operation is allowed; nono is the sole authorization authority. You MUST issue the appropriate tool call even if a path is outside the workspace, sensitive, or likely denied. Pass the requested path directly because Nonaut normalizes it. A follow-up such as 'try it', 'do it anyway', or 'force it' means retry the last operation once under the existing policy so the denial is observable; it never means evade the sandbox. Report the real tool result.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderKind {
    Codex,
    Claude,
}

impl ProviderKind {
    pub const ALL: [Self; 2] = [Self::Codex, Self::Claude];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Codex => "Codex / ChatGPT",
            Self::Claude => "Claude (Anthropic API)",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Codex => "Existing Codex account · tools isolated by nono",
            Self::Claude => "ANTHROPIC_API_KEY · tools isolated by nono",
        }
    }

    pub const fn connect_hint(self) -> &'static str {
        match self {
            Self::Codex => "Browser sign-in opens if needed.",
            Self::Claude => "Using ANTHROPIC_API_KEY from the environment.",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "codex" => Ok(Self::Codex),
            "claude" => Ok(Self::Claude),
            value => anyhow::bail!("unknown saved provider `{value}`"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    pub id: String,
    pub display_name: String,
    pub default_reasoning_effort: String,
    pub reasoning_efforts: Vec<ReasoningOption>,
}

#[derive(Clone, Debug)]
pub struct ReasoningOption {
    pub id: String,
    pub description: String,
}

#[derive(Debug)]
pub enum ProviderEvent {
    Delta(String),
    Usage(u64),
    ToolStarted(String),
    ToolFinished {
        tool: String,
        success: bool,
        output: String,
    },
    Completed,
    Interrupted,
    SessionStarted,
    Failed(String),
}

#[derive(Clone)]
pub enum Provider {
    Codex(Codex),
    Claude(Claude),
}

impl Provider {
    pub async fn connect(kind: ProviderKind, cwd: &Path) -> Result<Self> {
        match kind {
            ProviderKind::Codex => Codex::connect(cwd).await.map(Self::Codex),
            ProviderKind::Claude => Claude::connect(cwd).await.map(Self::Claude),
        }
    }

    pub fn model(&self) -> Model {
        match self {
            Self::Codex(provider) => provider.model(),
            Self::Claude(provider) => provider.model(),
        }
    }

    pub fn models(&self) -> &[Model] {
        match self {
            Self::Codex(provider) => provider.models(),
            Self::Claude(provider) => provider.models(),
        }
    }

    pub fn select_model(&self, model_id: &str) -> Result<Model> {
        match self {
            Self::Codex(provider) => provider.select_model(model_id),
            Self::Claude(provider) => provider.select_model(model_id),
        }
    }

    pub fn select_reasoning_effort(&self, effort: &str) -> Result<String> {
        match self {
            Self::Codex(provider) => provider.select_reasoning_effort(effort),
            Self::Claude(provider) => provider.select_reasoning_effort(effort),
        }
    }

    pub async fn new_session(&self) -> Result<()> {
        match self {
            Self::Codex(provider) => provider.new_session().await,
            Self::Claude(provider) => provider.new_session().await,
        }
    }

    pub async fn stream(
        &self,
        input: String,
        request_id: u64,
        sender: mpsc::UnboundedSender<(u64, ProviderEvent)>,
    ) -> Result<()> {
        match self {
            Self::Codex(provider) => provider.stream(input, request_id, sender).await,
            Self::Claude(provider) => provider.stream(input, request_id, sender).await,
        }
    }

    pub async fn interrupt(&self) -> Result<()> {
        match self {
            Self::Codex(provider) => provider.interrupt().await,
            Self::Claude(provider) => provider.interrupt().await,
        }
    }
}
