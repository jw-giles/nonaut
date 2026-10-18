fn main() -> anyhow::Result<()> {
    let mode = std::env::args().nth(1);
    if mode.as_deref() == Some("__nonaut_sandbox") {
        return nonaut::run_sandbox();
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    if mode.as_deref() == Some("__nonaut_mcp") {
        let workspace = std::env::args_os()
            .nth(2)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("MCP workspace was not provided"))?;
        return runtime.block_on(nonaut::run_mcp(workspace));
    }
    runtime.block_on(nonaut::run())
}
