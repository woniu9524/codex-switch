# codex-switch 产品需求

## 1. 产品定位

`codex-switch` 是一个只面向 Codex 的本地模型路由工具。它不管理 Claude、Gemini、OpenCode 等生态，也不接管 Codex 登录流程；它只做一件事：让 Codex 的模型请求先进入本地代理，再由用户选择转发到用户配置的 OpenAI-compatible 供应商。

一句话：`codex-switch` 是 Codex 的登录态保护器和模型路由切换器。

## 2. 产品目标

当前最重要的修正是移除内置官方默认路由。首次没有供应商时，用户必须先添加或导入一个带 API Key 的 OpenAI-compatible 供应商，才能启用本地代理。

## 3. 用户故事

作为一个已经使用 Codex 的用户，我希望打开 `codex-switch` 后先配置明确的 OpenAI-compatible 供应商，再启用本地代理。这样 Codex 请求只会转发到我主动配置的上游，不会出现隐藏的官方默认路由。

## 4. P0 流程

### 首次启用

1. 用户首次打开 `codex-switch`。
2. 应用读取 Codex 配置目录和登录态，只做诊断，不修改配置。
3. 首页展示状态为“未启用”，当前供应商为“未设置供应商”。
4. 如果没有 active provider，主按钮显示“先添加”并保持禁用。
5. 用户添加或导入带 API Key 的供应商，并设为当前路由目标。
6. 用户点击“启用”。
7. 应用启动本地代理，例如 `http://127.0.0.1:8787/v1`。
8. 应用备份 `~/.codex/config.toml`。
9. 应用写入固定 `model_provider = "codex-switch"`，并将 `[model_providers.codex-switch].base_url` 指向本地代理。
10. 应用写入 `supports_websockets = false`，让 Codex 直接使用 HTTP Responses。
11. 首页进入“已启用”状态。

### 添加第三方供应商

1. 用户进入“供应商”页。
2. 用户点击“添加供应商”。
3. 用户填写名称、Base URL、API Key、默认模型、官网和备注。
4. 保存后 provider 进入列表。
5. 如果当前没有 active provider，表单默认建议“保存后设为当前路由目标”。
6. 如果用户选择“保存并切换”，应用将其设为 active provider。
7. provider 缺少 API Key 时允许保存为草稿，但不能设为当前。

### 导入供应商

1. 用户进入“导入”页。
2. 用户粘贴 `codexswitch://` 或兼容的 `ccswitch://` provider 链接。
3. 应用解析并展示预览：名称、Endpoint、模型、API Key 遮罩、来源协议。
4. 用户确认导入。
5. 如果链接包含 `enabled=true`，应用只显示“建议切换”，必须由用户确认后才切换。

### 切换供应商

1. 用户在供应商列表点击“设为当前”。
2. 应用展示供应商名称、Endpoint、默认模型和 Key 状态。
3. 用户确认后更新 active provider。
4. 代理无需重启即可转发到新 provider。
5. 如果默认模型变化，应用只更新 `config.toml` 顶层 `model`；`model_provider` 始终保持 `codex-switch`。

### 停用

1. 用户点击“停用”。
2. 应用停止本地代理。
3. 应用恢复启用前的 `model_provider`、`model`、`openai_base_url`，并清理 `codex-switch` provider 残留。
4. 应用不修改 `~/.codex/auth.json`。
5. 首页回到“未启用”，保留当前已选择的用户供应商；如果没有供应商，则显示“未设置供应商”。

## 5. UI 要求

### 首页

- 未启用时，主按钮必须是“启用”。
- 没有 active provider 时，主按钮显示“先添加”并禁用。
- 已启用时，主按钮变为“停用”。
- 当前供应商显示 active provider；为空时显示“未设置供应商”。
- 首次首页要求用户先添加或导入 provider。
- 展示 Codex 配置目录、登录态诊断、本地代理端口和配置接管状态。

### 供应商页

- provider 展示 Endpoint、默认模型、Key 状态和健康状态。
- 切换动作使用“设为当前”，不使用开关。

### 导入页

- 支持 `codexswitch://`。
- 兼容 `ccswitch://`。
- API Key 永远遮罩显示。
- `enabled=true` 不允许静默切换当前 provider。

### 设置页

- 保留代理端口、Codex 配置目录、开机启动、备份恢复。
- 配置策略默认是固定 `codex-switch` 本地 provider。
- 高级设置默认收起。
- 用户可手动切换到“自定义 Provider”，仅用于兼容特殊 Codex 版本。

## 6. 异常要求

- Codex 未登录：允许启用代理，但提示“Codex 登录态未确认”。
- 本地端口被占用：提示端口占用，并允许一键换用可用端口。
- provider 缺少 API Key：允许保存，不允许设为当前。
- 导入链接无效：不保存 provider，展示字段级错误。
- 配置写入失败：不进入已启用状态，尝试回滚备份。
