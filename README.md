# oxy-sf

**oxy-sf** is a fast, native (Rust) reimplementation of the commonly used parts
of the Salesforce CLI (`sf`). The built binary is named **`oxysf`**.

> oxy-sf is an independent, community project. It is **not** affiliated with,
> endorsed by, or sponsored by Salesforce.

## Why

The official `sf` CLI is feature-rich but starts a Node.js runtime on every
invocation, which adds noticeable latency to common, scriptable operations.
oxy-sf reimplements the high-traffic commands as a single small native binary
with fast startup and true (non-async) parallelism for batch work.

## Build & run

Requires a stable Rust toolchain (edition 2021).

```sh
# Build everything
cargo build

# Run the CLI
cargo run -p oxysf -- --help

# Or build a release binary
cargo build --release
./target/release/oxysf --help
```

## Command surface (implemented so far)

Global flags are available on every command:

- `--json` — machine-readable JSON envelopes on stdout
- `-o, --target-org <USERNAME|ALIAS>` — the org to act against
- `--api-version <VERSION>` — override the REST API version (default `62.0`)
- `-v, --verbose` — verbose output (also unmasks secrets in `org display`)

Commands:

| Command | Description |
| --- | --- |
| `oxysf org login web` | OAuth2 web-server (browser) flow with PKCE |
| `oxysf org login jwt` | JWT bearer flow (`--username`, `--client-id`, `--jwt-key-file`) |
| `oxysf org logout` | Remove stored credentials (`-o` or `--all`) |
| `oxysf org list` | List locally authenticated orgs |
| `oxysf org display` | Show details for the target org (token masked unless `-v`) |
| `oxysf data query` | Run a SOQL query (`-q`), output `human`/`json`/`csv` |

## Output conventions

- JSON results (success or error) go to **stdout**.
- Human-readable success output goes to **stdout**; human errors go to **stderr**.
- Logs and progress indicators go to **stderr**, keeping stdout pipe-clean.

## Credential storage

Authenticated orgs are stored as one JSON file per org under
`~/.sf/oxysf/<username>.json`, created with `0600` permissions on Unix.

oxy-sf writes into its **own** `oxysf/` subdirectory rather than touching the
real `sf` CLI's encrypted auth files. Matching `sf`'s on-disk encryption for a
true drop-in experience is a documented follow-up; for now these files are
plaintext-but-`0600`.

## Workspace layout

```
crates/
  oxysf-core/   # the engine: auth, connection, REST, config, output, threading
  oxysf/        # the `oxysf` binary: clap root + dispatch
  oxysf-org/    # the `org` topic
  oxysf-data/   # the `data` topic
```

## License

MIT. See [LICENSE](./LICENSE).
