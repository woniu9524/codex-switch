# Troubleshooting

[中文](./troubleshooting.md) | [English](./troubleshooting.en.md)

## Codex does not pick up the change after enabling

Open a new Codex session after enabling the proxy. Many Codex config changes only take effect after a new session or process reads `~/.codex/config.toml`.

Also check that the active provider has an API key, the default model is not empty, and the local proxy port is available.

## Codex still points to codex-switch but the proxy is not running

Open `codex-switch` and disable proxying so the app can attempt config restore. If the app cannot open, use the latest backup in settings or manually inspect `~/.codex/config.toml` for `model_provider = "codex-switch"` and `[model_providers.codex-switch]`.

## Provider requests return an auth error

Confirm that the provider API key is saved and readable from the system credential store. Also confirm that the endpoint is OpenAI-compatible, commonly in the form `https://example.com/v1`.

## Model list fetching fails

Model fetching depends on provider compatibility with `/v1/models`. If the provider does not support that endpoint, enter the model name manually. A provider cannot be enabled or selected without a model name.

## image_generation is not supported by the provider

Some OpenAI-compatible providers do not support the `image_generation` tool. Edit the provider and enable the option to remove image generation tools before requests are forwarded.

## responses/compact requests fail

`codex-switch` converts `/v1/responses/compact` to `/v1/responses` and rewrites Codex compact model names to the active provider's default model when needed. If it still fails, check whether the upstream provider supports the Responses API.

## The proxy port is already in use

The default proxy port is `8787`. If it is occupied, the app attempts to find a later available port. You can also set a custom port in settings.

## Plugins or voice input still do not work

`codex-switch` is designed to preserve the Codex signed-in workflow as far as possible, but it cannot make every account-bound capability work consistently across every Codex version and provider. Confirm Codex is signed in and check provider compatibility.
