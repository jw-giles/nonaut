fn main() -> anyhow::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("__nonaut_sandbox") {
        return nonaut::run_sandbox();
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(nonaut::run())
}
