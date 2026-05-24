# What is codex-switch?

[中文](./what-is-codex-switch.md) | [English](./what-is-codex-switch.en.md)

`codex-switch` 是一个专为 Codex 登录态设计的 provider switcher。它通过本地代理把 Codex 的模型请求转发到选中的 OpenAI-compatible provider，同时尽量保留 Codex 已登录账号的工作方式，避免为了切换第三方 provider 而退化成纯 API Key 工作流。

## 痛点

Codex 用户常见的需求不是单纯“换一个 API Key”，而是：

- 已经登录 Codex 账号，还想使用第三方模型、模型中转或自定义 endpoint。
- 不希望切换 provider 后影响插件、语音输入等依赖账号态的能力。
- 不想反复手改 `~/.codex/config.toml`。
- 希望切换过程可逆，出问题时能恢复原始 Codex 配置。

普通 provider switcher 往往只关注请求转发到哪里。`codex-switch` 更关注 Codex 的 signed-in workflow：让 Codex 尽量保持登录态，再在本地代理层完成 provider switching。

## 原理

启用代理时，`codex-switch` 会启动一个监听在 `127.0.0.1` 的本地服务，并把 Codex 的 active model provider 改写为 `codex-switch`。这条 provider 配置指向本地代理，而不是直接指向第三方 endpoint。

Codex 发起 `/v1/models`、`/v1/responses` 或 `/v1/responses/compact` 请求时，请求先进入本地代理。代理再根据当前选中的 provider，把请求转发到对应的 OpenAI-compatible endpoint，并附带该 provider 的 API Key。

停用代理或退出应用时，`codex-switch` 会尝试恢复启用前的 `model_provider`、`model`、`openai_base_url` 和被管理的 provider 配置。

## 能力

- 管理多个 OpenAI-compatible provider。
- 保存 provider 名称、Endpoint、默认模型、主页、备注和 API Key。
- 将 API Key 存入系统凭据。
- 一键切换当前 provider 和模型。
- 将 Codex provider 指向本地 `127.0.0.1` 代理。
- 转发 `/v1/models`、`/v1/responses`、`/v1/responses/compact`。
- 上游转发时附带当前 provider 的 API Key。
- 转发前剥离 Codex 原始 `Authorization` / `Cookie`。
- 支持 `responses/compact` 模型改写。
- 可移除不兼容 provider 的 `image_generation` 工具请求。
- 启用前备份配置，停用或退出时尝试恢复。
- 支持 `codexswitch://` 和兼容的 `ccswitch://` provider 深链导入。

## 适用人群

`codex-switch` 适合已经登录 Codex 账号、主要使用 Codex、但希望接入多个 OpenAI-compatible provider 的用户。它尤其适合不想牺牲插件、语音输入等账号态能力，又不想手动维护 `~/.codex/config.toml` 的用户。

如果你只需要管理 Codex，并且最在意的是“登录态 + provider 切换 + 配置可恢复”，`codex-switch` 是一个更专注的选择。

## 不适用场景

如果你要统一管理 Claude Code、Gemini CLI、OpenCode、OpenClaw 等多个工具，`cc-switch` 这类 all-in-one manager 更合适。

如果你需要跨设备云同步、完整用量统计、MCP / Skills / Prompts 统一管理，`codex-switch` 当前不是为这些场景设计的。

如果你要求所有 Codex 账号态能力在任何 Codex 版本和任何 provider 上都始终完整可用，也需要谨慎。`codex-switch` 的目标是尽量保留 Codex signed-in workflow，而不是接管 Codex 官方登录，也不是让所有上游 provider 行为一致。
