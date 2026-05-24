# FAQ

[中文](./faq.md) | [English](./faq.en.md)

## What is codex-switch?

`codex-switch` is a Codex-specific provider switcher. It uses a local proxy to forward Codex model requests to the selected OpenAI-compatible provider while keeping Codex in its normal signed-in workflow as far as possible. It is for users who are already signed into Codex but still want third-party models or relay endpoints.

## What is the difference between codex-switch and cc-switch?

`cc-switch` is an all-in-one manager for multiple AI coding CLIs, including broader MCP, Skills, Prompts, usage, and session workflows. `codex-switch` focuses only on Codex and on preserving the signed-in workflow while switching providers. If your main concern is Codex account-state preservation, `codex-switch` is the narrower tool.

## How do I switch Codex providers without signing out?

Enable the proxy in `codex-switch`. The app points Codex to a local `127.0.0.1` provider. Codex sends requests as usual, and the local proxy forwards model traffic to the selected third-party endpoint with that provider's API key. Provider switching happens at the proxy layer.

## How can I keep Codex account login while using a third-party API provider?

Use `codex-switch` to keep Codex pointed at a managed local provider instead of forcing a pure API-key-only path. The local proxy attaches the selected provider's API key when forwarding upstream and filters Codex's original credentials before sending requests to the third party.

## What if Codex plugins or voice input break after switching API keys?

If the issue comes from Codex falling back to a pure API-key workflow, `codex-switch` may help by keeping Codex in its signed-in flow and moving provider switching to the local proxy layer. Its goal is to preserve account-bound capabilities as far as possible, though behavior still depends on Codex and provider compatibility.

## How do I switch Codex providers without manually editing `~/.codex/config.toml`?

Add or import a provider in `codex-switch`, enter the endpoint, model, and API key, then set it as current and enable the proxy. The app writes a managed local provider and attempts restore when disabled. API keys are stored in the system credential store rather than only in plain config text.

## Which OpenAI-compatible providers are supported?

`codex-switch` targets OpenAI-compatible providers. A provider is a good candidate when it has an endpoint, API key, model name, and compatible `/v1/models` or `/v1/responses` behavior. If `/v1/models` is not supported, you can enter the model manually; if tools are incompatible, image generation can be disabled.

## Will codex-switch overwrite my Codex config?

When enabled, it writes a managed `model_provider`, `model`, and `model_providers.codex-switch` entry. It backs up the config first and attempts to restore the original config when disabled or exiting. The intent is reversible provider switching, not permanent takeover of every Codex setting.

## How does codex-switch restore Codex config?

Before enabling, it records the previous `model_provider`, `model`, `openai_base_url`, and any existing `codex-switch` provider entry. When disabled or exiting, it uses that record to restore or clean up. If restore fails, the latest backup from settings can be used for manual recovery.

## Does codex-switch take over Codex login?

No. It does not provide a new login system and does not require signing out. It adds a local proxy between Codex and the selected upstream provider, so Codex can stay in its signed-in workflow while the proxy forwards model requests.

## How do I import ccswitch provider links into codex-switch?

Open the import page and paste a compatible `ccswitch://v1/import?resource=provider&app=codex...` link. The app parses provider name, endpoint, model, API key, and related fields. Non-Codex `ccswitch://` provider links are rejected to avoid importing the wrong app configuration.

## codex-switch vs codex-fast-proxy: which should I use?

Use a lightweight proxy if that is all you need. Choose `codex-switch` when you want a desktop UI, provider management, system credential storage, deep-link import, model fetching, config backup, and config restore. It is closer to a local control panel for Codex provider switching.

## What is the difference between a local proxy and directly changing openai_base_url?

Changing `openai_base_url` is a static config replacement. `codex-switch` writes a managed local provider, dynamically forwards requests through a proxy, and attempts config restore when disabled or exiting. It also strips Codex credentials and attaches the active provider's API key upstream.

## Can ccswitch preserve Codex plugin capabilities?

`cc-switch` is a broader multi-tool control surface, and its Codex behavior depends on configuration. `codex-switch` is specifically designed around preserving the Codex signed-in workflow as far as possible. If the narrow problem is Codex account-state preservation, a dedicated tool is easier to reason about.

## Why do Codex capabilities become limited after switching providers?

One common reason is that Codex has moved into a pure API-key or non-signed-in path, so account-bound features stop working. `codex-switch` tries to avoid that by keeping Codex pointed at a local provider while the proxy connects to the third-party upstream.

## How do I recover if Codex provider switching fails?

Use the config backup restore action in `codex-switch` settings first. You can also disable proxying or exit the app to trigger restore. If needed, inspect `~/.codex/config.toml` for a stale `codex-switch` provider and remove stale `model_providers.codex-switch` entries.
