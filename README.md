# asapi

`asapi` is a public Apple App Store research CLI and local research workspace. It
does not require Apple credentials.

The existing commands query Apple directly:

```bash
asapi search "habit tracker" --country us
asapi lookup 284882215 --country us
asapi lookup 284882215 --country us --full
asapi popularity 284882215
asapi reviews 284882215 --country us
asapi chart free --country us
asapi list countries
```

## Local application

Start the local Axum API and embedded React interface:

```bash
asapi app serve
```

The application listens on `http://127.0.0.1:3000` by default. The only command
under `asapi app` is `serve`; projects, apps, storefronts, and keywords can be
managed in the web interface or through the same JSON API used by agents.

On first run, the server creates a `Default` project. Each project has an
isolated SQLite database at:

```text
~/.local/share/asapi-storage/projects/<project-id>.sqlite3
```

Choose a different base directory when necessary:

```bash
asapi app serve --storage-path /path/to/base
```

This stores databases below
`/path/to/base/asapi-storage/projects/`.

The API description is available from the running server:

```text
http://127.0.0.1:3000/api/openapi.json
```

Agents can read 30-day snapshots from
`GET /api/v1/projects/<project-id>/apps/<app-id>/history` using the optional
`country` and `resource=details|popularity` query parameters.

Example agent workflow:

```bash
# Find the Default project.
curl http://127.0.0.1:3000/api/v1/projects

# Track an app in its main US storefront.
curl -X POST \
  http://127.0.0.1:3000/api/v1/projects/<project-id>/apps \
  -H 'content-type: application/json' \
  -d '{"app_id":284882215,"country":"us"}'

# Configure an additional storefront.
curl -X POST \
  http://127.0.0.1:3000/api/v1/projects/<project-id>/apps/284882215/storefronts \
  -H 'content-type: application/json' \
  -d '{"country":"jp","auto_refresh":false}'

# Track a keyword. Search results are shared by query and country per project.
curl -X POST \
  http://127.0.0.1:3000/api/v1/projects/<project-id>/apps/284882215/keywords \
  -H 'content-type: application/json' \
  -d '{"keyword":"music","country":"us","notes":"Core category term"}'
```

The main storefront and opt-in automatic storefronts refresh after 24 hours.
Opening an on-demand storefront or review page refreshes stale data. App,
popularity, and keyword observations are retained for 30 days.

## Workspace

The Rust workspace is split by responsibility:

- `crates/appstore-api`: Apple HTTP client, normalization, and typed query inputs
- `crates/app`: SQLx persistence, projects, storefronts, caching, and refresh logic
- `crates/server`: Axum endpoints and embedded static application
- `crates/cli`: the `asapi` executable and Clap interface
- `web`: React and TypeScript application

Build and verify:

```bash
cargo test --workspace
cargo build --release
```

Source builds require Node.js and npm. Cargo installs the locked web
dependencies when needed, rebuilds the React application when its sources
change, and embeds it in the `asapi` binary automatically.

## Install or update

On macOS or Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/maximbezd99/asapi-cli/main/install.sh | bash
```

The installer supports Intel/AMD and ARM systems and installs to
`~/.local/bin` by default. To choose another directory:

```bash
curl -fsSL https://raw.githubusercontent.com/maximbezd99/asapi-cli/main/install.sh |
  INSTALL_DIR=<dir> bash
```

Install the bundled agent skill with:

```bash
asapi install-skill
```
