# M1 当前提示词栈基线

## 结论

当前首轮模型请求在用户问题前已经承载 77,567 字符的客户端可见指令；连同工具 schema，首轮 input 为 27,536 tokens，仅 3,840 tokens 命中缓存。问题主要来自默认全量注入、跨角色分散和作用域过宽。

## 会话事实

| 项目 | 值 |
| --- | --- |
| Model | `gpt-5.6-sol` |
| Personality | `pragmatic` |
| Collaboration mode | `default` |
| Multi-agent mode | `explicitRequestOnly` |
| Approval policy | `never` |
| Permission profile | `disabled` |
| Model context window | 353,400 tokens |
| Git commit | `7af161c82dfe` |

## 指令层

| 顺序 | API 位置/角色 | 字符 | 内容 |
| --- | --- | ---: | --- |
| 1 | `instructions` | 16,270 | 基础身份、人格、写作、工作方式、技能协议 |
| 2 | `developer` | 32,515 | 权限、Default mode、77 项 Skills、Apps、Plugins、Workflow Host |
| 3 | `developer` | 1,844 | root agent 与多智能体编排 |
| 4 | `developer` | 186 | explicit-request-only 多智能体限制 |
| 5 | `user` context | 26,472 | AGENTS、environment 和 filesystem context |
| 6 | `developer` | 280 | 多 active Cycle、当前 session 未聚焦警告 |

用户消息前合计 77,567 字符；工具 schema 不在该字符统计内。

## Token 与缓存

- 首轮 input：27,536 tokens。
- 首轮 cached input：3,840 tokens。
- 首轮 output：542 tokens，其中 reasoning output 121 tokens。
- 当前基础提示 SHA-256：`3aeec1d261e8f8345f8243b233a17f95fa7a6d0f7e6693f3cede952481cafab6`。
- 当前根 AGENTS SHA-256：`d70670fad7b08efc46e59f87002e96161bb0aae4e57879c2b0a8d25e6ba54c01`。

## 构造路径

- `session/mod.rs` 按 config override、历史 session meta、当前模型目录的顺序解析基础提示。
- `session/mod.rs` 聚合权限、developer instructions、collaboration mode、personality、skills、Workflow 和扩展上下文。
- AGENTS 被包装成 `user` 角色的 contextual fragment。
- `turn.rs::build_prompt` 最终组合 `base_instructions`、`input`、`tools` 和 output schema。

## 验证方式

- 读取本次真实 rollout 的 `session_meta`、`turn_context`、首轮 `response_item` 和 `token_count`。
- 对照本地 `config.toml`、根 AGENTS 与提示构造源码。
- 本 Milestone 为只读基线，没有产品改动，不需要运行测试。

## S3 统计纠正

S3 使用 `jq` 直接解析同一 rollout 的首个聚合 developer `response_item`，确认 Skills block 有 77 个条目而不是最初人工统计的 80 个；其中唯一名称为 76，`skill-creator` 出现两次。字符总量和其他 M1 数值不变。
