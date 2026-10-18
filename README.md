# whoa

`whoa` is a security-first agent harness built with
[`nono`](https://github.com/nolabs-ai/nono) as its sandbox enforcement layer.

The first milestone is a minimal Codex-backed agent shell. Whoa launches
`codex app-server` as its unsandboxed provider control plane, speaks the
app-server protocol over stdio, and streams responses into its own TUI.

Codex owns ChatGPT login, model discovery, conversation state, and inference.
Whoa never reads or stores its OAuth tokens. It saves only UI preferences such as
the selected provider, model, and per-model reasoning effort in the platform
configuration directory. Threads are ephemeral and transcripts are not persisted.

Whoa exposes seven model tools: `read_file`, `list_directory`, `write_file`,
`edit_file`, `delete_file`, `search_files`, and `run_command`. Codex's native
execution tools remain disabled.
Every model-requested operation starts in a fresh `nono run --allow <workspace>`
sandbox with read/write access to the directory where Whoa was launched, no
filesystem access outside that workspace, blocked network access, and a cleared
environment. `HOME` and `TMPDIR` point at the workspace, while structured tool
paths beginning with `~` are expanded before execution so nono can enforce the
real requested path. Sandbox denials are reduced to a concise policy message;
normal application errors retain their stderr. No provider credentials or
ambient API keys are inherited.

## Requirements

- `codex` CLI 0.162.1 or a compatible release with app-server support
- `nono` 0.79.0 or newer

Whoa prefers the official curl installation at `~/.local/bin/nono` over a
package-manager copy found earlier in `PATH`. Set `NONO_BIN` to an explicit
executable path to override discovery.

## Run

```sh
cargo run
```

On the first run, Whoa opens directly into its TUI and asks for a default
provider. Codex / ChatGPT is the only provider for now. The choice is remembered,
so later runs connect immediately. If Codex is already signed in, Whoa reuses
that session; otherwise app-server opens the normal ChatGPT login flow.

Use `/provider` to reopen provider setup and `/help` to list commands. `/q` or
`/quit` exits Whoa. `Ctrl+C` interrupts an active turn, or exits while idle.

Use `/model` to open the provider-backed model picker. Navigate with the arrow
keys or `j`/`k`, press Enter to apply a model, or Escape to close it. The selected
model is remembered per provider and restored on later runs.

Use `/reasoning` to choose from the current model's provider-advertised reasoning
efforts. The choice is remembered per model. Lower efforts generally respond
faster and consume fewer reasoning tokens. Use `/new` to start a fresh ephemeral
thread while keeping the current provider, model, and reasoning effort.

The prompt stays editable while a response streams. Shift+Enter inserts a
newline, Escape interrupts the active response, and submitting another prompt
interrupts the response before sending the new turn. The footer reports the
current Codex thread's cumulative token usage.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
