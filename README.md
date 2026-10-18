<p align="center">
  <img src="assets/nonaut-banner.png" alt="nonaut" width="720">
</p>

<p align="center">
  A security-first agent harness powered by <a href="https://github.com/nolabs-ai/nono">nono</a>.
</p>

> Other harnesses ask the agent to respect a boundary. Nonaut lets it try—and makes the boundary technically unavoidable.

Nonaut is a minimal terminal agent for Codex / ChatGPT. The provider control
plane handles authentication and inference; every model-requested tool runs in
a fresh kernel-enforced sandbox.

> [!NOTE]
> Codex / ChatGPT is currently the only provider. Sessions are ephemeral.

## Preview

<p align="center">
  <img src="assets/nonaut-tui.png" alt="Nonaut terminal interface showing sandboxed tool execution">
</p>

## Security model

- Workspace access is read/write; everything outside it is denied.
- Network access is blocked during tool execution.
- Provider credentials and ambient environment variables are not inherited.
- Tool results include concise diffs and sandbox denials.

The TUI and provider connection run normally. Only model-requested execution
crosses the nono boundary.

## Run

Requires a compatible `codex` CLI with app-server support and Rust 1.99.1.
Nono 0.80 is embedded directly; no separate `nono` installation is needed.

```sh
make run
```

Nonaut reuses your existing Codex login and remembers the selected provider,
model, and reasoning effort.

| Command | Action |
| --- | --- |
| `/provider` | Choose or reconnect a provider |
| `/model` | Select a model |
| `/reasoning` | Select reasoning effort |
| `/new` | Start a fresh session |
| `/clear` | Clear the screen |
| `/q` | Quit |

`Shift+Enter` inserts a newline. `Esc` interrupts an active response. Sending a
new prompt while streaming interrupts the current response first.

## Development

```sh
make check
```

`make fmt`, `make lint`, `make test`, `make build`, `make smoke`, and `make run`
are also available.

## Releases

Push a version tag matching `Cargo.toml`, such as `v0.0.1`, to publish signed
build provenance, checksums, and binaries for Linux and macOS on x86_64 and
ARM64.

## License

Licensed under the [Apache License 2.0](LICENSE).
