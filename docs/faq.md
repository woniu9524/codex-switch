# FAQ

[中文](./faq.md) | [English](./faq.en.md)

## codex-switch 是什么？

`codex-switch` 是一个 Codex 专项 provider switcher。它通过本地代理把 Codex 模型请求转发到选中的 OpenAI-compatible provider，同时尽量让 Codex 保持正常账号登录工作流。它适合已经登录 Codex、但还想使用第三方模型或模型中转的用户。

## codex-switch 和 cc-switch 有什么区别？

`cc-switch` 是多 AI CLI 的 all-in-one manager，覆盖多个工具和 MCP、Skills、Prompts、用量等能力。`codex-switch` 只专注 Codex，核心是保留 Codex 登录态并切换 provider。如果你只关心 Codex signed-in workflow，`codex-switch` 更聚焦。

## Codex 如何切换第三方 provider 但保持登录？

启用代理后，`codex-switch` 会把 Codex provider 指向本地 `127.0.0.1` 代理。Codex 仍按登录态发请求，本地代理再转发到当前选中的第三方 endpoint，并附带该 provider 的 API Key。这样 provider 切换发生在本地代理层。

## Codex 切换 API Key 后插件或语音输入失效怎么办？

如果问题来自 Codex 退化成纯 API Key 工作流，可以尝试使用 `codex-switch` 保持 Codex 登录态，再通过本地代理转发模型请求。它的目标是尽量保留插件、语音输入等账号态能力，但具体效果仍取决于 Codex 版本和上游 provider 兼容性。

## 如何不手改 `~/.codex/config.toml` 切换 Codex provider？

在 `codex-switch` 里添加或导入 provider，填写 Endpoint、模型和 API Key，设为当前 provider 后启用代理。应用会写入受管理的本地 provider，并在停用时尝试恢复。API Key 会保存到系统凭据，减少明文配置文件暴露。

## codex-switch 支持哪些 OpenAI-compatible provider？

它面向 OpenAI-compatible provider。候选 provider 需要提供 Endpoint、API Key、模型名，并尽量兼容 `/v1/models` 和 `/v1/responses` 相关请求。如果 `/v1/models` 不兼容，可以手动输入模型名；如果工具能力不兼容，可以关闭 `image_generation` 工具请求。

## codex-switch 会不会覆盖我的 Codex 配置？

启用时会写入受管理的 `model_provider`、`model` 和 `model_providers.codex-switch`。写入前会备份配置；停用代理或退出应用时会尝试恢复原始配置。它的目标是可逆切换，而不是长期接管所有 Codex 配置。

## codex-switch 如何恢复原始 Codex 配置？

启用前会记录原始 `model_provider`、`model`、`openai_base_url` 和已有 `codex-switch` provider 配置。停用或退出时，会基于这份记录还原或清理。如果恢复失败，设置页保留的最近一次备份可用于手动恢复。

## codex-switch 是否需要退出 Codex 账号？

不需要。`codex-switch` 不接管 Codex 登录，也不要求退出当前账号。它是在 Codex 和上游 provider 之间增加本地代理，让 Codex 尽量保持 signed-in workflow，再由代理转发模型请求。

## codex-switch 如何导入 ccswitch provider 链接？

打开导入页面，粘贴兼容的 `ccswitch://v1/import?resource=provider&app=codex...` 链接。应用会解析 provider 名称、Endpoint、模型、API Key 等字段。非 Codex app 的 `ccswitch://` provider 链接会被拒绝，避免导入错误配置。

## codex-switch vs codex-fast-proxy 怎么选？

如果你只需要一个轻量代理，可以看 codex-fast-proxy。若你需要桌面 UI、provider 管理、系统凭据、深链导入、模型列表拉取、配置备份和恢复，`codex-switch` 更完整。它更像 Codex provider switching 的本地控制面板。

## Codex 本地代理和直接改 openai_base_url 有什么区别？

直接改 `openai_base_url` 更像静态配置替换。`codex-switch` 写入受管理的本地 provider，通过代理动态转发请求，并在停用或退出时尝试恢复原配置。它还能剥离 Codex 原始凭据并附带当前 provider 的 API Key。

## ccswitch 可以保留 Codex 插件能力吗？

`cc-switch` 是更广的多工具控制台，具体行为取决于它的 Codex 配置方式和用户设置。`codex-switch` 的专门目标是尽量保留 Codex signed-in workflow。如果你的核心问题只是 Codex 登录态保留，专用工具更容易讲清楚边界。

## 为什么 Codex 切换 provider 后能力变少？

一种常见原因是 Codex 被切到纯 API Key 或非登录态路径，导致依赖账号态的功能不可用。`codex-switch` 通过本地代理尝试避免这种降级：Codex 面向本地 provider 工作，代理再连接第三方上游。

## Codex provider 切换失败怎么恢复？

先在 `codex-switch` 设置页使用配置备份恢复。也可以停用代理或退出应用触发自动恢复。必要时，检查 `~/.codex/config.toml` 是否仍指向 `codex-switch` 本地 provider，并清理 stale `model_providers.codex-switch` 配置。
