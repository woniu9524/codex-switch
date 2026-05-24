[中文](./README.md) | [English](./README.en.md)

# codex-switch

> Provider switching for Codex without giving up normal account-based login capabilities.

`codex-switch` is a Tauri desktop app built for a very specific Codex problem: when `cc-switch`-style proxying is used with Codex, the workflow can fall back into a pure API-key path and lose capabilities that depend on a normal signed-in account.

That matters because features such as plugin access, voice input, and other account-bound Codex capabilities should not have to disappear just because you want to route model traffic to another provider. `codex-switch` exists to keep Codex signed in while still adding switchable OpenAI-compatible providers.

- It does not replace Codex account login
- It does not require you to sign out
- It does not force Codex into an API-key-only workflow
- It handles provider switching, local proxying, and config restore on the desktop side

## The Problem It Solves

- `cc-switch`-style proxying for Codex does not reliably preserve normal account-based capabilities
- Pure API-key routing can break or weaken plugin access, voice input, and similar signed-in features
- Switching providers often means repeated manual edits to `~/.codex/config.toml`
- There is usually no reversible desktop control surface for proxy enablement, provider switching, and config restore

## Project Goal

The goal of `codex-switch` is simple:

1. Keep Codex in a normal signed-in account state.
2. Add switchable OpenAI-compatible providers on top of that signed-in workflow.
3. Preserve the conditions needed for account-bound features such as plugins and voice input.
4. Bring provider switching, model switching, proxy control, and config recovery into one local desktop app.

## Core Capabilities

- Manage multiple OpenAI-compatible providers
- Store provider name, endpoint, default model, homepage, notes, and API key
- Switch the active provider and model with one click
- Start a local proxy and inject a `codex-switch` provider entry into Codex config
- Restore the original Codex configuration when the proxy is disabled or the app exits
- Keep the latest config backup for manual recovery
- Detect Codex login mode from local auth state, including API key, ChatGPT login, mixed, and unknown
- Import providers from `codexswitch://` and `ccswitch://` links
- Fetch model lists from compatible `/v1/models` endpoints
- Remove `image_generation` tool calls for providers that do not support them
- Configure proxy port, Codex directory override, theme, and launch behavior

## How It Is Different

Most provider switchers focus on one question: which API should the next request hit?

`codex-switch` focuses on a different question: how do we switch providers for Codex without abandoning the signed-in account workflow? Instead of replacing Codex auth, it places a local proxy between Codex and the upstream provider so Codex can stay logged in while requests are forwarded to the selected provider.

## How It Works

1. The app inspects Codex `auth.json` and `config.toml` to understand the current login state.
2. When proxying is enabled, it starts a local service and rewrites Codex to use `codex-switch` as the active model provider.
3. That provider points to a local `127.0.0.1` endpoint while preserving the login-oriented Codex flow.
4. Codex sends `/v1/models`, `/v1/responses`, and related requests to the local proxy.
5. The proxy forwards those requests to the currently selected upstream provider and attaches that provider's API key.
6. When proxying is disabled or the app exits, the previous Codex configuration is restored as far as possible.

## Good Fit For

- You are already signed into Codex and want provider switching on top of that
- You do not want to lose plugin access, voice input, or similar account-bound capabilities
- You switch endpoints and models frequently across multiple providers
- You want to stop editing local Codex config by hand
- You want proxy control, provider management, and config recovery in one place

## Tech Stack

- Tauri 2
- React 18
- TypeScript
- Vite
- Rust

## Quick Start

### Requirements

- Node.js
- pnpm
- Rust toolchain
- Tauri desktop build environment

### Install

```bash
pnpm install
```

### Development

```bash
pnpm dev
```

This starts both the frontend dev server and the Tauri desktop app.

### Common Commands

```bash
pnpm dev
pnpm build
pnpm typecheck
pnpm build:renderer
```

## Project Structure

```text
src/                React app entry and pages
src/pages/          Control, providers, import, and settings screens
src/components/     Shared layout and base UI components
src/features/       Feature modules for providers and settings
src/lib/            Frontend API bridge and helper utilities
src-tauri/src/app/  State, config writing, auth diagnostics, deep links, secrets, and app logic
src-tauri/src/proxy Local proxy implementation
```

## Reference & Thanks

This project is inspired by and developed with reference to:

- [farion1231/cc-switch](https://github.com/farion1231/cc-switch)
- [gaoguobin/codex-fast-proxy](https://github.com/gaoguobin/codex-fast-proxy)

Thanks to the ideas and prior exploration from these projects, which helped clarify the direction of `codex-switch`: keep Codex signed in, then make provider switching work on top of that.
