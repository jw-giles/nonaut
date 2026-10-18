mod codex;

use std::path::Path;

use anyhow::Result;
use tokio::sync::mpsc;

use codex::Codex;

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

    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "codex" => Ok(Self::Codex),
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
}

impl Provider {
    pub async fn connect(kind: ProviderKind, cwd: &Path) -> Result<Self> {
        match kind {
            ProviderKind::Codex => Codex::connect(cwd).await.map(Self::Codex),
        }
    }

    pub fn model(&self) -> Model {
        match self {
            Self::Codex(provider) => provider.model(),
        }
    }

    pub fn models(&self) -> &[Model] {
        match self {
            Self::Codex(provider) => provider.models(),
        }
    }

    pub fn select_model(&self, model_id: &str) -> Result<Model> {
        match self {
            Self::Codex(provider) => provider.select_model(model_id),
        }
    }

    pub fn select_reasoning_effort(&self, effort: &str) -> Result<String> {
        match self {
            Self::Codex(provider) => provider.select_reasoning_effort(effort),
        }
    }

    pub async fn new_session(&self) -> Result<()> {
        match self {
            Self::Codex(provider) => provider.new_session().await,
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
        }
    }

    pub async fn interrupt(&self) -> Result<()> {
        match self {
            Self::Codex(provider) => provider.interrupt().await,
        }
    }
}
