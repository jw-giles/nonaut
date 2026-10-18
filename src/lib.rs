mod config;
mod providers;
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

pub async fn run() -> Result<()> {
    if std::env::args().nth(1).as_deref() == Some("__nonaut_tool") {
        return tools::run_worker().context("tool worker failed");
    }

    let cwd = std::env::current_dir().context("could not determine the working directory")?;
    let (default_provider, startup_warning) = match config::load_default_provider() {
        Ok(provider) => (provider, None),
        Err(error) => (
            None,
            Some(format!("Could not load the saved provider: {error:#}")),
        ),
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
