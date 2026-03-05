# termy_theme_store

Theme store API for Termy, backed by PostgreSQL via SQLx.

## Run

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/termy_theme_store
export THEME_STORE_BIND=127.0.0.1:8080
export GITHUB_CLIENT_ID=...
export GITHUB_CLIENT_SECRET=...
export GITHUB_REDIRECT_URI=http://127.0.0.1:8080/auth/github/callback
cargo run -p termy_theme_store
```

Migrations run automatically on startup.

## Routes

- `GET /health`
- `GET /auth/github/login`
- `GET /auth/github/callback`
- `GET /auth/me`
- `POST /auth/logout`
- `GET /themes`
- `POST /themes`
- `GET /themes/:slug`
- `PATCH /themes/:slug`
- `GET /themes/:slug/versions`
- `POST /themes/:slug/versions`

## Example requests

Log in with GitHub (browser redirect):

```bash
open "http://127.0.0.1:8080/auth/github/login"
```

Create theme:

```bash
curl -X POST http://127.0.0.1:8080/themes \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "Tokyo Night",
    "slug": "tokyo-night",
    "description": "Dark blue terminal palette"
  }'
```

Publish version:

```bash
curl -X POST http://127.0.0.1:8080/themes/tokyo-night/versions \
  -H 'Content-Type: application/json' \
  -d '{
    "version": "1.0.0",
    "fileKey": "themes/tokyo-night/1.0.0/theme.json",
    "changelog": "Initial release",
    "checksumSha256": "..."
  }'
```
