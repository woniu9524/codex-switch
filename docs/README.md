# codex-switch 文档索引

`codex-switch` 是一个面向 Codex 的本地模型路由工具。它只负责把 Codex 的模型请求路由到用户显式配置的 OpenAI-compatible 供应商。

## 推荐阅读入口

- [产品需求](./product-requirements.md)
- [技术设计](./technical-design.md)
- [实现计划](./implementation-plan.md)
- [验收清单](./acceptance-checklist.md)

`index.html` 是早期静态说明页，仅作为历史材料保留；当前契约以这些 Markdown 文档为准。

## 核心判断

用户首次打开软件后，不再提供隐藏的官方默认路由。P0 体验必须是：

1. 新安装状态下供应商列表为空，当前供应商显示“未设置供应商”。
2. 没有 active provider 时首页主动作显示“先添加”并禁用。
3. 用户必须添加或导入带 API Key 的 OpenAI-compatible 供应商后才能启用。
4. Codex 的 ChatGPT 登录态保留在 Codex 自己的 `auth.json` 中。
5. 第三方 upstream 永远不能收到 Codex/ChatGPT 登录凭据，只能收到用户为该 provider 保存的 API Key。
