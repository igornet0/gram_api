# Local Web UI

```bash
gram start ui
# http://127.0.0.1:8788
```

Default bind: **127.0.0.1** (not 0.0.0.0).

## API

| Method | Path |
|--------|------|
| GET | `/api/status` |
| GET/POST | `/api/bots` |
| POST | `/api/bots/:name/start\|stop` |
| DELETE | `/api/bots/:name` |
| GET | `/api/accounts` *(accounts feature)* |
| POST | `/api/accounts/login` *(accounts feature)* |
| … | start/stop/logout/delete account |

UI is a minimal embedded dashboard. Domain logic stays in `control` services — not in the browser.
