<p align="center">
  <img src="assets/nonaut-banner.png" alt="nonaut" width="720">
</p>

<p align="center">
  A security-first agent harness powered by <a href="https://github.com/nolabs-ai/nono">nono</a>.
</p>

Nonaut is a minimal terminal agent for Codex / ChatGPT. The provider control
plane handles authentication and inference; every model-requested tool runs in
a fresh kernel-enforced sandbox.

> Early stage: Codex / ChatGPT is the only provider and sessions are ephemeral.

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

Requires a compatible `codex` CLI with app-server support, `nono` 0.79 or
newer, and Rust 1.99.1.

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

Set `NONO_BIN` to override nono discovery. Otherwise Nonaut prefers
`~/.local/bin/nono` and falls back to `PATH`.
