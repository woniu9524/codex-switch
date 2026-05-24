[中文](./README.md) | [English](./README.en.md)

# codex-switch

> Codex provider switching without giving up normal account-based login.

`codex-switch` 是一个专为 Codex 登录态设计的本地桌面 provider switcher：Codex provider switching without giving up normal account-based login。它让 Codex 尽量保持 ChatGPT / Codex 账号登录状态，同时通过本地代理切换 OpenAI-compatible provider，并在停用代理时恢复原始 `~/.codex/config.toml`。

- 保持 Codex 已登录账号，不要求退出当前账号。
- 通过本地代理切换 OpenAI-compatible providers。
- 停用代理或退出应用时尝试恢复原始 Codex 配置。

这意味着你可以在需要第三方模型、模型中转或自定义 endpoint 时，尽量保留插件、语音输入等依赖账号态的 Codex 能力，而不是把 Codex 降级成只走第三方 API Key 的工作流。

## 效果预览

### 软件截图

![codex-switch 软件截图](./docs/images/codex-switch-screenshot.png)

### 启用效果

![codex-switch 启用效果](./docs/images/codex-switch-enabled-effect.png)

## 为什么需要它

很多 provider switcher 只回答一个问题：下一次请求应该发到哪个 API。

`codex-switch` 关注的是另一个更具体的问题：**Codex 已经登录账号时，怎样切换第三方 provider，同时尽量保留账号态能力？**

如果直接改写配置或进入纯 API Key 模式，Codex 可能失去插件、语音输入等依赖登录态的能力。多供应商切换还常常需要反复手改 `~/.codex/config.toml`，既麻烦又容易留下不可恢复的配置状态。

`codex-switch` 把 provider 管理、本地代理、模型切换、配置备份和配置恢复放到一个桌面入口里，目标是在“保留 Codex signed-in workflow”的前提下完成 provider switching。

## 核心能力

- 管理多个 OpenAI-compatible provider。
- 保存 provider 名称、Endpoint、默认模型、主页、备注和 API Key。
- 将 API Key 存入系统凭据，而不是只写进普通配置文件。
- 一键切换当前生效的 provider 和模型。
- 启用本地代理后，将 Codex provider 指向 `127.0.0.1`。
- 代理转发 `/v1/models`、`/v1/responses`、`/v1/responses/compact`。
- 自动附带当前 provider 的 API Key，并剥离 Codex 原始 `Authorization` / `Cookie`。
- 支持 `responses/compact` 模型改写。
- 可为不兼容的 provider 移除 `image_generation` 工具请求。
- 停用代理或退出应用时尝试恢复原始 Codex 配置。
- 保留最近一次配置备份，支持手动恢复。
- 识别 Codex 当前登录状态，包括 API Key、ChatGPT 登录、混合态与未知态。
- 支持 `codexswitch://` 和兼容的 `ccswitch://` provider 深链导入。
- 支持代理端口、自定义 Codex 配置目录、主题和开机启动设置。

## 与 cc-switch 的区别

`cc-switch` 是面向多 AI coding CLI 的 all-in-one manager，覆盖 Claude Code、Codex、Gemini CLI、OpenCode、OpenClaw 等工具，并提供 MCP、Skills、Prompts、用量、会话、云同步等综合管理能力。

`codex-switch` 则更窄、更专注：它只围绕 Codex 的 signed-in workflow，解决“Codex 不退出账号、不退化成纯 API Key 工作流，还能切换 OpenAI-compatible provider”的问题。

| 维度 | codex-switch | cc-switch |
|---|---|---|
| 产品定位 | Codex 登录态保留 + provider 切换 | 多 AI CLI 统一控制台 |
| 支持范围 | Codex 专项 | Claude Code、Codex、Gemini CLI、OpenCode、OpenClaw 等 |
| 核心问题 | 不退出 Codex 账号也能切换 provider | 多工具配置、MCP、Skills、Prompts、用量等统一管理 |
| 关键机制 | 本地代理 + 可逆写入 Codex 配置 | 多工具配置管理 + proxy/failover 等综合能力 |
| 最适合用户 | 已登录 Codex 且想保留账号态能力的人 | 同时管理多个 AI coding CLI 的用户 |

更多细节见：[codex-switch vs cc-switch](./docs/codex-switch-vs-ccswitch.md)。

## 工作方式

1. 应用检测 Codex 的 `auth.json` 和 `config.toml`，判断当前登录状态。
2. 启用代理后，应用启动本地服务，并把 Codex 当前模型供应商改写为 `codex-switch`。
3. 写入的 `model_providers.codex-switch` 指向 `http://127.0.0.1:{port}/v1`，并设置 `requires_openai_auth = true`。
4. Codex 发出的 `/v1/models`、`/v1/responses`、`/v1/responses/compact` 请求会先进入本地代理。
5. 本地代理根据当前选中的 provider 转发请求，附带该 provider 的 API Key，并过滤不应传给上游的 Codex 原始凭据。
6. 停用代理或退出应用时，程序会尝试恢复原始 `model_provider`、`model`、`openai_base_url` 和被管理的 provider 配置。

完整说明见：[How it works](./docs/how-it-works.md)。

## 适合谁

- 你已经登录 Codex 账号，但还想切换不同 OpenAI-compatible provider。
- 你不希望为了切换 provider 而丢掉插件、语音输入等账号态能力。
- 你同时使用多个模型服务，需要频繁切换 endpoint 和默认模型。
- 你不想继续手动维护 `~/.codex/config.toml`。
- 你希望代理启停、provider 管理、模型拉取和配置恢复都在一个入口完成。

## 不适合谁

- 你要统一管理 Claude Code、Gemini CLI、OpenCode、OpenClaw 等多个工具，`cc-switch` 这类 all-in-one manager 更合适。
- 你需要跨设备云同步、完整用量统计、MCP / Skills / Prompts 统一管理。
- 你要求所有 Codex 账号态能力在任何 Codex 版本、任何 provider 上都始终完整可用。`codex-switch` 的目标是尽量保留 signed-in workflow，而不是接管 Codex 官方行为。

## FAQ

### codex-switch 是什么？

`codex-switch` 是一个 Codex 专项 provider switcher。它通过本地代理把 Codex 请求转发到选中的 OpenAI-compatible provider，同时尽量保留 Codex 已登录账号的工作方式。

### codex-switch 会接管 Codex 登录吗？

不会。它不提供新的 Codex 登录体系，也不要求退出当前账号。它是在 Codex 与上游 provider 之间增加本地代理，让 provider switching 尽量发生在不破坏登录态的前提下。

### Codex 如何切换第三方 provider 但保持登录？

启用代理后，`codex-switch` 会把 Codex 的 active provider 指向本地 `127.0.0.1` 代理。Codex 仍按登录态发起请求，本地代理再把模型请求转发到当前选中的第三方 endpoint。

### 它会覆盖我的 Codex 配置吗？

启用时会可控地写入 `model_provider`、`model` 和 `model_providers.codex-switch`。写入前会保留备份；停用代理或退出应用时会尝试恢复原始配置，并保留最近一次备份供手动恢复。

### 支持哪些 provider？

它面向 OpenAI-compatible provider。只要 provider 提供兼容的 Endpoint、API Key、模型名，并能处理 `/v1/models` 或 `/v1/responses` 相关请求，就可以作为候选接入。

### 可以导入 ccswitch provider 链接吗？

可以。`codex-switch` 支持自己的 `codexswitch://` 深链，也兼容 `ccswitch://v1/import?resource=provider&app=codex...` 形式的 provider 导入链接。

更多问题见：[FAQ](./docs/faq.md)。

## 快速开始

### 环境要求

- Node.js
- pnpm
- Rust toolchain
- Tauri 桌面开发环境

### 安装依赖

```bash
pnpm install
```

### 本地开发

```bash
pnpm dev
```

这个命令会同时启动前端开发服务和 Tauri 桌面应用。

### 常用命令

```bash
pnpm dev
pnpm build
pnpm typecheck
pnpm build:renderer
```

## 技术栈

- Tauri 2
- React 18
- TypeScript
- Vite
- Rust

## 项目结构

```text
src/                React 应用入口与页面
src/pages/          控制页、供应商页、导入页、设置页
src/components/     通用布局与基础 UI 组件
src/features/       供应商与设置相关功能模块
src/lib/            前端 API 封装与工具函数
src-tauri/src/app/  状态存储、配置写入、登录态诊断、深链、密钥与业务逻辑
src-tauri/src/proxy 本地代理实现
```

## 推荐阅读

- [What is codex-switch?](./docs/what-is-codex-switch.md)
- [How it works](./docs/how-it-works.md)
- [codex-switch vs cc-switch](./docs/codex-switch-vs-ccswitch.md)
- [FAQ](./docs/faq.md)
- [Troubleshooting](./docs/troubleshooting.md)
- [llms.txt](./llms.txt)
- [llms-full.txt](./llms-full.txt)

## 参考与致谢

本项目在设计与实现过程中参考了以下项目：

- [farion1231/cc-switch](https://github.com/farion1231/cc-switch)
- [gaoguobin/codex-fast-proxy](https://github.com/gaoguobin/codex-fast-proxy)
