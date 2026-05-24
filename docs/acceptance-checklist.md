# codex-switch 验收清单

## P0 产品验收

- [ ] 新安装状态下供应商列表为空。
- [ ] 首页当前供应商显示“未设置供应商”。
- [ ] 没有 active provider 时主按钮显示“先添加”并禁用。
- [ ] 添加带 API Key 的供应商并设为当前后，首页未启用时主按钮是“启用”。
- [ ] 首页已启用时主按钮是“停用”。
- [ ] `activeProviderId` 缺失或指向无效供应商时会归一化为 `null`。
- [ ] 添加、导入、切换供应商是首次启用前置条件。
- [ ] `ccswitch://` provider 链接可以导入或给出明确兼容提示。

## 配置验收

- [ ] 启用前备份 `~/.codex/config.toml`。
- [ ] 默认写入 `model_provider = "codex-switch"`。
- [ ] 默认写入 `[model_providers.codex-switch].base_url = "http://127.0.0.1:<port>/v1"`。
- [ ] 默认写入 `supports_websockets = false`。
- [ ] 停用后恢复或移除 `openai_base_url`。
- [ ] 停用后恢复原 `model_provider`。
- [ ] 停用后恢复原 `model`。
- [ ] 停用后清理旧 `[model_providers.codex-switch]`。
- [ ] 不修改 `~/.codex/auth.json`。
- [ ] TOML 写入不会破坏 `[mcp_servers.*]` 和 `[projects.*]`。

## 代理验收

- [ ] 代理只监听 `127.0.0.1`。
- [ ] `/health` 返回当前状态。
- [ ] `POST /v1/responses` 可转发。
- [ ] `POST /v1/responses/compact` 可转发。
- [ ] `GET /v1/models` 可转发。
- [ ] 其他路径拒绝。
- [ ] 第三方转发时剥离 `Authorization`、`Cookie` 等 Codex 登录凭据。
- [ ] 第三方转发时注入第三方 API Key。
- [ ] 流式响应按 chunk 返回。

## 安全验收

- [ ] 不记录 API Key。
- [ ] 不记录 Cookie。
- [ ] 不记录 Authorization。
- [ ] 不记录请求体、提示词、工具参数和模型输出。
- [ ] 删除 provider 不删除 Codex 登录态。
- [ ] 停用不修改 Codex 登录态。

## 工程验收

- [ ] `npm run typecheck` 通过。
- [ ] `npm run build:renderer` 通过。
- [ ] `cargo fmt --check` 通过。
- [ ] `cargo test` 通过。
- [ ] 本地 UI 在桌面尺寸下无明显布局错乱。
- [ ] Tauri dev 模式下命令可调用。
- [ ] Tauri build 可生成安装包。
