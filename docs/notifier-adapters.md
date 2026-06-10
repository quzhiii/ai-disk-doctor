# Notifier Adapter Foundation

Phase 13 adds a script-level Notifier Adapter Foundation for governance delivery. The stable boundary remains `governance-event.json`: scheduler scripts and governance runs create this event first, then delivery adapters consume the event without changing the Rust anomaly core.

## Supported Adapters

| Adapter | Script | Secret handling | Failure artifact |
|---|---|---|---|
| `local-file` | `scripts/governance/send-governance-event.sh` | No secrets | None |
| `webhook` | `scripts/governance/send-governance-event.sh` | `--webhook-url` for the existing generic webhook path | `webhook-failure.json` |
| `feishu` | `scripts/governance/notifiers/feishu.sh` | `FEISHU_WEBHOOK_URL` environment variable | `feishu-failure.json` |

## Feishu

The Feishu adapter posts a text message built from `headline` and `summary_markdown` in `governance-event.json`. It requires `bash`, `jq`, and `curl`.

```bash
export FEISHU_WEBHOOK_URL="https://example.test/feishu-webhook"

./scripts/governance/send-governance-event.sh \
  --adapter feishu \
  --event-path .aidisk/governance/governance-event.json \
  --output-dir .aidisk/governance
```

You can also run governance directly with Feishu delivery:

```bash
export FEISHU_WEBHOOK_URL="https://example.test/feishu-webhook"

./scripts/governance/run-governance.sh --notifier-adapter feishu
```

## Secrets

Do not pass Feishu secrets on the command line. `FEISHU_WEBHOOK_URL` must come from the environment so it can be injected by the shell, scheduler, or secret manager. The adapter never writes the resolved Feishu webhook URL into `feishu-failure.json`.

The existing generic webhook path still accepts `--webhook-url` for backwards compatibility. Prefer environment injection for concrete platform adapters.

## Failure Behavior

- Successful delivery updates `governance-event.json` with `delivery_status: delivered` and the adapter name.
- Feishu delivery failure writes `feishu-failure.json` with timing, adapter, timeout, and event-path context, but no secret URL.
- Generic webhook delivery failure keeps `webhook-failure.json` behavior unchanged.

## Future Adapters

Slack, WeChat, DingTalk, email, Telegram, and Discord can reuse the same `governance-event.json` contract and dispatcher shape. They should follow the Feishu pattern: secrets from environment variables, no daemon, no cleanup automation, and no secrets in failure artifacts.
