# Troubleshooting

[中文](./troubleshooting.md) | [English](./troubleshooting.en.md)

## 启用后 Codex 没有生效

启用代理后，请新开 Codex 会话。多数 Codex 配置变化需要新的会话或进程读取 `~/.codex/config.toml` 后才会生效。

同时检查当前 provider 是否有 API Key、默认模型是否为空、本地代理端口是否被占用。

## Codex 仍然指向 codex-switch 但代理没有运行

打开 `codex-switch` 后停用代理，应用会尝试恢复配置。如果无法打开应用，可以在设置页使用最近一次备份恢复，或手动检查 `~/.codex/config.toml` 中的 `model_provider = "codex-switch"` 和 `[model_providers.codex-switch]`。

## provider 请求返回鉴权错误

确认该 provider 的 API Key 已保存，并且系统凭据可读取。也确认 Endpoint 是否为 OpenAI-compatible 地址，常见形式是 `https://example.com/v1`。

## 模型列表拉取失败

模型列表依赖 provider 兼容 `/v1/models`。如果 provider 不支持该接口，可以手动输入模型名。模型名为空时无法启用或切换到该 provider。

## image_generation 工具不兼容

部分 OpenAI-compatible provider 不支持 `image_generation` 工具。编辑 provider，打开“关闭图片生成工具”，代理会在发送请求前移除该工具。

## responses/compact 请求失败

`codex-switch` 会把 `/v1/responses/compact` 转为 `/v1/responses`，并在模型名是 Codex compact 模型时改写为当前 provider 默认模型。仍失败时，请检查上游是否兼容 Responses API。

## 端口被占用

默认代理端口是 `8787`。如果端口被占用，应用会尝试寻找后续可用端口，也可以在设置页手动指定端口。

## 插件或语音输入仍不可用

`codex-switch` 的目标是尽量保留 Codex signed-in workflow，但无法让所有 Codex 账号态能力在所有版本和 provider 上都始终完整可用。请确认 Codex 本身已登录，并检查当前 Codex 版本和 provider 兼容性。
