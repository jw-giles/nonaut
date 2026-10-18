fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = args.first().map(String::as_str);
    if mode == Some("__nonaut_sandbox") {
        return nonaut::run_sandbox();
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    if mode == Some("__nonaut_mcp") {
        let workspace = std::env::args_os()
            .nth(2)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("MCP workspace was not provided"))?;
        return runtime.block_on(nonaut::run_mcp(workspace));
    }
    if mode == Some("__nonaut_tui") {
        return runtime.block_on(nonaut::run_tui());
    }
    if mode == Some("__nonaut_session") {
        let id = args
            .get(1)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session id was not provided"))?;
        let workspace = args
            .get(2)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("session workspace was not provided"))?;
        return runtime.block_on(nonaut::run_session_worker(id, workspace));
    }
    if mode == Some("exec") {
        let command = args
            .iter()
            .skip(1)
            .skip_while(|argument| argument.as_str() == "--")
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        if command.trim().is_empty() {
            anyhow::bail!("usage: nonaut exec [--] <command>");
        }
        let output = runtime.block_on(nonaut::run_exec(command))?;
        print!("{output}");
        if !output.ends_with('\n') {
            println!();
        }
        return Ok(());
    }
    runtime.block_on(nonaut::run(args))
}
