# sRight

sRight is a macOS Finder context menu helper. The current implementation covers the initial
FinderSync to CLI loop and Phase 2 basic file actions:

- FinderSync dynamic menu generation from config
- Rust core config, action, and JSONL log model
- `sright-cli` action runner with dangerous-action confirmation gating
- Tauri 2 + Vue 3 + TypeScript preferences app, aligned with the dbx-style Vite/pnpm stack
- Basic actions for copying paths/names, file info, moving to trash, permanent delete,
  folder creation from filename, and dissolving folders

Templates, compression, image tools, translations, and custom scripts are intentionally not implemented yet.

## Layout

```text
apps/desktop/                 Tauri + Vue preferences app
crates/sright-core/           Shared Rust core
crates/sright-cli/            Finder action CLI
native/macos/FinderSyncExtension/
docs/                         Requirements and design inputs
```

## Config And Logs

By default, sRight uses:

```text
~/Library/Application Support/sRight/config.json
~/Library/Application Support/sRight/actions.jsonl
```

Tests and local diagnostics can override this with:

```sh
export SRIGHT_APP_SUPPORT_DIR=/tmp/sright-app-support
```

## Rust

```sh
cargo fmt --all --check
cargo test --workspace
cargo run -p sright-cli -- config init
cargo run -p sright-cli -- config print
cargo run -p sright-cli -- action run --id debug.echo --path "$PWD/docs/requirements.md"
cargo run -p sright-cli -- action run --id copy.path --path "$PWD/docs/requirements.md"
cargo run -p sright-cli -- logs tail --limit 5
```

## Desktop App

```sh
pnpm install
pnpm --filter @sright/desktop typecheck
pnpm --filter @sright/desktop tauri dev
```

The preferences app reads and writes the same JSON config as the CLI. It exposes menu enables,
dangerous-action confirmation settings, diagnostics, and a `Run Debug Action` smoke button.

## FinderSync Extension

See `native/macos/README.md` for the local Xcode setup. The extension reads `config.json`,
shows enabled menu actions, asks for native confirmation before dangerous actions, and invokes
`sright-cli action run --id <action-id>`.
