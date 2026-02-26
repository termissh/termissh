# TermiSSH

TermiSSH is a simple terminal-based SSH connection manager.

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run
```

## Optional API Sync

Create a `.env` file and set:

```env
API_URL=https://termissh.org
```

Open the app and press `s` to set your API key.
If API settings are missing or unavailable, TermiSSH works in local-only mode.

## Shortcuts

- `Up/Down`: Select server
- `Enter`: Connect to selected server
- `n`: Add server
- `e`: Edit server
- `d`: Delete server
- `s`: Set API key
- `q`: Quit

## Session Macros

- `:p` -> `pwd`
- `:ls` -> `ls -la`
- `:top` -> `htop`
- `:u` -> `uptime`
- `:d` -> `docker ps`
- `:dc` -> `docker compose up -d`
- `:exit` -> `exit`

Note: `:` macros do not work inside interactive apps like `htop`, `vim`, or `nano`.

## Contact

- Website: https://termissh.org
- Email: hacimertgokhan@gmail.com
