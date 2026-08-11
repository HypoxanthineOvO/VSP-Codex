---
kind: stage-report
cycle: C28-prompt-stack-quality-redesign
milestone: M2
stage: E-worker-routing
status: accepted
updated: 2026-08-11T14:14:30+08:00
---

# M2 Stage E Worker Routing Report

## 结论

Worker Routing 已完整接入 Multi-Agent V2。`spawn_agent` 现在接受 `mechanical | long_context | standard | explore | critical | escalation`；未提供显式model/reasoning/service-tier override时默认`standard`。运行时只从built-in VSP defaults和有效`CODEX_HOME/worker-routing.toml`读取策略，不读取repo/project routing配置。

## 技术实现

- 新增独立`agent/worker_routing`模块，加载built-in policy并按class覆盖用户policy；缺失class继续使用built-in候选。
- 候选按声明顺序用offline provider/model catalog检查provider、model、reasoning、minimum context window与service tier。
- 第一个兼容候选写入child config；全部不可用时继承parent，不跨class升级，并返回结构化skip reason。
- TOML解析错误只报告文件与byte offset，不回显源内容；unknown fields被拒绝，credentials/raw auth不进入结果或telemetry。
- semantic routing与显式model/reasoning/service-tier override互斥；trusted policy可禁用显式override。
- role config若固定provider/model/reasoning/service tier，则与semantic routing显式冲突；不固定模型的role仍可应用instructions/nickname等配置。
- routing改变provider/model/reasoning时，省略`fork_turns`默认最近3轮；最终模型配置与parent一致时默认full history。仅service-tier变化仍允许full history。
- 成功结果包含requested class、policy version、candidate id、effective provider/model/reasoning/tier、fallback和skipped reasons；同一组字段进入结构化trace，bounded class/fallback进入metric tags。

## 修改模块

- `codex-rs/core/src/agent/worker_routing.rs`：policy、overlay、validation、offline resolver、selection/result。
- `codex-rs/core/src/agent/worker_routing_tests.rs`：built-in、partial overlay、secret-safe invalid TOML与compatibility tests。
- `codex-rs/core/src/agent/role.rs`：worker-routed role application与pin conflict。
- `codex-rs/core/src/tools/handlers/multi_agents_v2/spawn.rs`：argument precedence、resolver、fork、telemetry与output接线。
- `codex-rs/core/src/tools/handlers/multi_agents_spec.rs`：公开`routing_class` enum与routing output schema。
- `multi_agents_tests.rs`、`multi_agents_spec_tests.rs`、`spec_plan_tests.rs`：handler、schema与tool-plan回归。

## 主要逻辑与用户行为测试清单

1. 六个semantic class及默认`standard`保持可解析。
2. user policy只覆盖声明class，缺失`long_context`继续使用built-in default。
3. cross-provider candidate使用自己的offline catalog并写入真实child snapshot。
4. unknown provider/model、unsupported reasoning/context/tier按顺序跳过；全部失败继承parent并报告reason code。
5. `routing_class`与显式model/reasoning/service-tier互斥。
6. role-pinned model配置与semantic routing冲突，不再静默覆盖。
7. explicit override与partial/full-history既有兼容行为保持；service-tier-only full history不回退。
8. metadata隐藏时仍返回routing决策，但不返回nickname/model目录等被隐藏输入元数据。
9. invalid policy不回显source或secret内容。
10. tool schema和tool-family planner持续公开`routing_class`，output schema要求routing对象。

## 验证结果

- `core --filter worker_routing`: 6/6 passed，包括真实cross-provider child config。
- `core --filter inherited_fallback`: 1/1 passed。
- `core --filter routing_with_explicit`: 1/1 passed。
- `core --filter spawn_agent_tool`: 6/6 passed。
- `core --filter multi_agent_v2_spawn`: 19/19 passed。
- `core --filter multi_agent_v2_full_history`: 1/1 passed。
- `core --filter multi_agent_feature_selects_one_agent_tool_family`: 1/1 passed。
- `cargo fmt --all -- --check` passed；仅stable rustfmt对nightly-only imports_granularity输出既有warning。
- `git diff --check` passed；Workflow YAML syntax passed。
- 未修改`ConfigToml`/nested config、Cargo依赖、app-server API或UI，因此不需要config schema、Bazel lock、app-server schema或snapshot更新。

## 实施中发现并解决的问题

- 初版复用了绑定parent provider的service-tier helper；cross-provider routing会用错catalog。现改为candidate resolver直接用candidate `ModelInfo`处理tier，fallback/explicit才走旧helper。
- 一个旧测试等待V2从未发出的legacy spawn begin/end events，在routing output断言更新后暴露固定timeout。测试改为验证当前真实V2 output和child config snapshot，不新增不存在的legacy产品行为。

## 预期结果

root可用稳定semantic class表达任务难度、上下文和成本需求；机器级trusted policy决定具体provider/model/reasoning。repository无法注入昂贵provider选择；unsupported policy候选不会造成隐式能力升级；每次spawn都能解释实际选择。

## 剩余风险与后续

- 当前Stage证明offline catalog与child config行为，尚未用本机真实VSPLab请求执行一次Subagent推理；该项进入最终smoke。
- policy在每次spawn时读取，便于即时更新；频繁spawn会增加一次小文件I/O，当前文件规模可忽略。
- 最终遗漏审计仍需覆盖真实prompt/token、Stage A/B回归、release脚本多target manifest聚合和三平台CI artifact发布。
