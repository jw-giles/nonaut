mod config;
mod mcp;
mod providers;
mod sessions;
mod tools;
mod tui;

use anyhow::{Context, Result};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
};

use tui::App;

pub fn run_sandbox() -> Result<()> {
    tools::run_sandbox().context("sandboxed tool failed")
}

pub async fn run_mcp(workspace: std::path::PathBuf) -> Result<()> {
    mcp::run(workspace).await.context("MCP server failed")
}

pub async fn run(args: Vec<String>) -> Result<()> {
    sessions::run_cli(args).await
}

pub async fn run_session_worker(id: String, workspace: std::path::PathBuf) -> Result<()> {
    sessions::run_worker(id, workspace).await
}

pub async fn run_tui() -> Result<()> {
    let cwd = std::env::current_dir().context("could not determine the working directory")?;
    let (default_provider, startup_warning) =
        if std::env::var_os("NONAUT_NO_AUTO_CONNECT").is_some() {
            (None, None)
        } else {
            match config::load_default_provider() {
                Ok(provider) => (provider, None),
                Err(error) => (
                    None,
                    Some(format!("Could not load the saved provider: {error:#}")),
                ),
            }
        };
    let mut app = App::new(cwd, default_provider, startup_warning);
    let mut terminal = ratatui::init();
    let keyboard_enhancements = execute!(
        std::io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    )
    .is_ok();
    let mouse_capture = execute!(std::io::stdout(), EnableMouseCapture).is_ok();
    let result = app.run(&mut terminal).await;
    if mouse_capture {
        let _ = execute!(std::io::stdout(), DisableMouseCapture);
    }
    if keyboard_enhancements {
        let _ = execute!(std::io::stdout(), PopKeyboardEnhancementFlags);
    }
    ratatui::restore();
    result.context("terminal error")?;
    Ok(())
}
