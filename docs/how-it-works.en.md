# How it works

[中文](./how-it-works.md) | [English](./how-it-works.en.md)

`codex-switch` keeps Codex in its normal signed-in workflow, routes model requests to a local `127.0.0.1` proxy, and forwards those requests to the selected OpenAI-compatible provider. It backs up the Codex config before enabling and attempts to restore the original config when disabled or exiting.

## 1. Read Codex State

The app locates the Codex config directory. By default this is `.codex` under the user's home directory, but a custom Codex directory can be set in settings.

It checks:

- `auth.json`: detects whether Codex appears to be using API key auth, ChatGPT login, mixed auth, or an unknown state.
- `config.toml`: reads the current `model_provider`, `model`, `openai_base_url`, and provider entries.

## 2. Prepare a Config Lease and Backup

Before enabling the proxy, `codex-switch` reads the current `config.toml`, records the original values, and saves a recent backup under `~/.codex-switch/backups/`.

The purpose is reversibility. When the proxy is disabled, the app exits, or manual recovery is needed, the previous config can be restored as far as possible.

## 3. Inject a Local Provider

When proxying is enabled, the app changes Codex's active provider to `codex-switch` and writes a local provider entry:

```toml
model_provider = "codex-switch"
model = "selected provider default model"

[model_providers.codex-switch]
name = "codex-switch"
base_url = "http://127.0.0.1:{port}/v1"
wire_api = "responses"
requires_openai_auth = true
supports_websockets = false
```

The important part is that `base_url` points to the local proxy, while `requires_openai_auth = true` preserves the login-oriented Codex flow.

## 4. Receive Codex Requests Locally

The local proxy only handles explicitly supported paths:

- `GET /health`
- `GET /v1/models`
- `POST /v1/responses`
- `POST /v1/responses/compact`

Other paths return `path_not_allowed`, so the proxy does not become an uncontrolled generic HTTP forwarder.

## 5. Forward to the Active Provider

The proxy reads the active provider, its endpoint, its model, and its API key from the system credential store. It then builds the upstream request.

During forwarding, it:

- Sends the request to the active provider's OpenAI-compatible endpoint.
- Attaches that provider's API key.
- Strips Codex's original `Authorization` and `Cookie` headers before forwarding to third parties.
- Supports zstd request body decompression.
- Converts `/v1/responses/compact` to `/v1/responses` and rewrites the model when needed.
- Removes `image_generation` tool calls when the provider has that compatibility option enabled.

## 6. Switch Providers and Models

When the active provider changes, the app updates the active provider state. If the proxy is already enabled, it also updates Codex's current model to the new provider's default model.

Codex continues to call the same local proxy, while the proxy forwards to the newly selected upstream endpoint.

## 7. Disable and Restore

When proxying is disabled or the app exits, `codex-switch` stops the local proxy and attempts to restore the recorded original config:

- Restore the previous `model_provider`.
- Restore the previous `model`.
- Restore the previous `openai_base_url`.
- Remove or restore the managed `model_providers.codex-switch` entry.

If no valid config lease exists, the app also cleans stale `codex-switch` provider entries so Codex is not left pointing at a dead local proxy.

## Boundary

`codex-switch` does not take over official Codex login, and it cannot make every account-bound capability work consistently across every Codex version and upstream provider. It is designed to preserve the Codex signed-in workflow as far as possible while forwarding model requests to the selected OpenAI-compatible provider.
