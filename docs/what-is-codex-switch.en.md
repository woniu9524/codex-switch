# What is codex-switch?

[中文](./what-is-codex-switch.md) | [English](./what-is-codex-switch.en.md)

`codex-switch` is a provider switcher built specifically for the Codex signed-in workflow. It routes Codex model requests through a local proxy to the selected OpenAI-compatible provider while helping Codex stay in its normal account-based login flow instead of falling back to a pure API-key-only workflow.

## Pain Point

Many Codex users do not simply want to swap an API key. They want to:

- Stay signed into Codex while using third-party models, relay services, or custom endpoints.
- Avoid losing plugins, voice input, and other account-bound capabilities when switching providers.
- Stop manually editing `~/.codex/config.toml`.
- Keep provider switching reversible so the original Codex config can be restored.

Most provider switchers focus on where to send the next request. `codex-switch` focuses on Codex's signed-in workflow: keep Codex operating as a signed-in app, then perform provider switching at the local proxy layer.

## How It Works

When proxying is enabled, `codex-switch` starts a local service on `127.0.0.1` and rewrites Codex's active model provider to `codex-switch`. That provider points to the local proxy instead of directly to a third-party endpoint.

When Codex sends `/v1/models`, `/v1/responses`, or `/v1/responses/compact`, the request reaches the local proxy first. The proxy forwards it to the selected OpenAI-compatible endpoint and attaches that provider's API key.

When the proxy is disabled or the app exits, `codex-switch` attempts to restore the previous `model_provider`, `model`, `openai_base_url`, and managed provider configuration.

## Capabilities

- Manage multiple OpenAI-compatible providers.
- Store provider name, endpoint, default model, homepage, notes, and API key.
- Store API keys in the system credential store.
- Switch the active provider and model with one click.
- Point Codex to a local `127.0.0.1` proxy provider.
- Proxy `/v1/models`, `/v1/responses`, and `/v1/responses/compact`.
- Attach the current provider's API key when forwarding upstream.
- Strip Codex's original `Authorization` / `Cookie` headers before forwarding.
- Rewrite `responses/compact` requests when needed.
- Remove `image_generation` tool calls for providers that do not support them.
- Back up config before enabling and attempt config restore when disabled or exiting.
- Import provider links from `codexswitch://` and compatible `ccswitch://` URLs.

## Who It Is For

`codex-switch` is for users who are already signed into Codex, primarily use Codex, and want to connect multiple OpenAI-compatible providers. It is especially useful when you want provider switching without intentionally giving up plugins, voice input, or other account-bound capabilities.

If your main concern is "Codex login state + provider switching + reversible config restore", `codex-switch` is intentionally focused on that job.

## Who It Is Not For

If you want one app to manage Claude Code, Gemini CLI, OpenCode, OpenClaw, and other AI coding CLIs, an all-in-one manager such as `cc-switch` is a better fit.

If you need cloud sync, full usage dashboards, or unified MCP / Skills / Prompts management across multiple tools, `codex-switch` is not designed around those workflows today.

If you require every account-bound Codex capability to work consistently with every Codex version and every upstream provider, be careful. `codex-switch` is designed to preserve the Codex signed-in workflow as far as possible; it does not take over official Codex login behavior or make every upstream provider behave identically.
