# codex-switch 实现计划

## 阶段 0：项目初始化

目标：维护 `codex-switch` Tauri 项目，保持模型路由边界清晰、状态模型简洁。

任务：

1. 初始化 Tauri 2 + React + TypeScript + Vite 项目。
2. 配置 `pnpm` 脚本：`dev`、`build`、`dev:renderer`、`build:renderer`、`typecheck`。
3. 配置 Rust crate、Tauri capabilities、基础窗口和应用元信息。
4. 建立目录结构：

```text
src/
  main.tsx
  styles.css
  lib/
    api.ts
    format.ts
    types.ts
src-tauri/
  src/
    app/
      mod.rs
      models.rs
      store.rs
      codex_auth.rs
      codex_config.rs
      deeplink.rs
      secrets.rs
    proxy/
      mod.rs
```

## 阶段 1：产品骨架

目标：搭出完整可用 UI，并通过真实 Tauri 命令访问后端能力。

任务：

1. 首页：未启用、已启用、未设置供应商和已选择供应商。
2. 供应商页：空列表、用户 provider 列表和添加表单。
3. 导入页：URL 输入、解析预览、确认导入。
4. 设置页：代理端口、Codex 配置目录、配置策略。
5. 关于页：数据目录、日志目录、配置策略。

## 阶段 2：状态层

目标：实现本地状态文件和 provider 状态归一化。

任务：

1. 实现 `StoredState`、`Provider`、`Snapshot`。
2. 实现 `AppStore::load`、`save`、`update`、`snapshot_data`。
3. 加载时将无效 `activeProviderId` 修复为 `null`。
4. API Key 状态通过系统凭据动态 hydrate。

## 阶段 3：Codex 配置层

目标：安全接管和恢复 `~/.codex/config.toml`。

任务：

1. 诊断 Codex 配置目录和 `auth.json`。
2. 使用 TOML parser 写入固定 `[model_providers.codex-switch]`。
3. 写入前备份配置。
4. 保存 restore point：`model_provider`、`model`、`openai_base_url`。
5. 停用时恢复 restore point。
6. 写入 `supports_websockets = false`，默认使用 HTTP Responses。
7. 清理旧 `[model_providers.codex-switch]`。
8. 增加单元测试覆盖其他 TOML section 保留。

## 阶段 4：本地代理

目标：实现透明代理和安全分流。

任务：

1. 监听 `127.0.0.1:<port>`。
2. 支持 `/health`、`/v1/models`、`/v1/responses`、`/v1/responses/compact`。
3. 转发时剥离 Codex 登录凭据并注入 provider key。
4. 缺少 active provider 或 provider key 时返回明确错误。
5. 支持流式响应。
6. 记录非敏感请求事件到 `proxy.jsonl`。

## 阶段 5：Provider 管理

目标：添加、编辑、删除、切换第三方 provider。

任务：

1. 保存 provider 到状态文件。
2. API Key 保存到系统凭据。
3. 缺少 API Key 时允许保存但禁止切换。
4. 当前 provider 禁止删除。
5. 切换 provider 时更新 active provider。
6. 已启用状态下切换 provider 时同步更新 Codex 顶层 `model`。

## 阶段 6：导入协议

目标：支持 `codexswitch://` provider 导入，并兼容 `ccswitch://` Codex provider 链接。

任务：

1. 实现 URL parser。
2. 校验 scheme、version、resource、app、endpoint。
3. 支持 `config` 的 JSON/TOML 解码。
4. 支持从导入配置中提取 API Key、Base URL 和 model。
5. 展示预览并遮罩 API Key。
6. `enabled=true` 只作为建议切换。

## 阶段 7：验证和打包

目标：确保 P0 可发布。

任务：

1. `pnpm typecheck`
2. `pnpm build:renderer`
3. `cargo fmt --check`
4. `cargo test`
5. 浏览器检查首页和供应商页。
6. Tauri dev 检查真实命令调用。
7. Tauri build 生成安装包。
