# How it works

[中文](./how-it-works.md) | [English](./how-it-works.en.md)

`codex-switch` 的工作方式是：让 Codex 继续处在正常账号登录工作流中，把模型请求先送到本地 `127.0.0.1` 代理，再由本地代理转发到当前选中的 OpenAI-compatible provider。启用前会备份配置，停用或退出时会尝试恢复原始 Codex 配置。

## 1. 读取 Codex 状态

应用会定位 Codex 配置目录，默认是用户目录下的 `.codex`，也可以在设置中指定自定义 Codex 目录。

它会检查：

- `auth.json`：判断当前是 API Key、ChatGPT 登录、混合态还是未知态。
- `config.toml`：读取当前 `model_provider`、`model`、`openai_base_url` 和已有 provider 配置。

## 2. 准备配置租约和备份

启用代理前，`codex-switch` 会读取当前 `config.toml`，记录原始配置，并在 `~/.codex-switch/backups/` 下保存最近一次配置备份。

这一步的目标是让切换过程尽量可逆。后续停用代理、退出应用或手动恢复时，都可以基于这份记录恢复。

## 3. 写入本地 provider

启用代理后，应用会把 Codex 的 active provider 改成 `codex-switch`，并写入一个本地 provider：

```toml
model_provider = "codex-switch"
model = "当前选中的 provider 默认模型"

[model_providers.codex-switch]
name = "codex-switch"
base_url = "http://127.0.0.1:{port}/v1"
wire_api = "responses"
requires_openai_auth = true
supports_websockets = false
```

这里的重点是 `base_url` 指向本地代理，`requires_openai_auth = true` 保留 Codex 所需的登录态工作方式。

## 4. 本地代理接收 Codex 请求

本地代理只处理明确支持的路径：

- `GET /health`
- `GET /v1/models`
- `POST /v1/responses`
- `POST /v1/responses/compact`

其他路径会返回 `path_not_allowed`，避免代理变成不受控的通用 HTTP 转发器。

## 5. 转发到当前 provider

代理会读取当前选中的 provider，取出它的 Endpoint、模型和系统凭据中的 API Key，然后构造上游请求。

转发时会做几件事：

- 将请求发送到当前 provider 的 OpenAI-compatible Endpoint。
- 附带当前 provider 的 API Key。
- 剥离 Codex 原始 `Authorization` 和 `Cookie`，避免把 Codex 凭据转发给第三方。
- 支持 zstd 请求体解压。
- 如果请求是 `/v1/responses/compact`，会转成 `/v1/responses` 并在需要时改写模型。
- 如果 provider 开启了“关闭图片生成工具”，会移除 `image_generation` 工具请求。

## 6. 切换 provider 和模型

切换 provider 时，应用会更新当前 active provider。代理已启用时，会同步更新 Codex 当前模型为新 provider 的默认模型。

因此，Codex 请求仍进入同一个本地代理入口，但代理会转发到新的上游 Endpoint。

## 7. 停用和恢复

停用代理或退出应用时，`codex-switch` 会停止本地代理，并尝试恢复启用前记录的配置：

- 恢复原来的 `model_provider`。
- 恢复原来的 `model`。
- 恢复原来的 `openai_base_url`。
- 移除或还原被管理的 `model_providers.codex-switch` 配置。

如果没有可用租约，也会清理遗留的 stale `codex-switch` provider 配置，尽量避免 Codex 被留在无效本地代理状态。

## 能力边界

`codex-switch` 不接管 Codex 官方登录，也无法让所有账号态能力在所有 Codex 版本和所有上游 provider 上都始终完整可用。它的设计目标是尽量保留 Codex signed-in workflow，同时把模型请求转发到选中的 OpenAI-compatible provider。
