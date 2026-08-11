# C9 Deep Research Built-In And Codex Fork Assessment

## Timeline

- 2026-06-14T08:44:37+08:00：创建 C9 `Deep Research Built-In And Codex Fork Assessment`。目标包含两个方向：一是新增内置 Deep Research 功能，通过 Question Tool 澄清需求、派发多个 Subagent 并汇总研究结果，默认输出到 `.pipeline` 且支持按需求指定其他位置；二是检查 Codex 上游更新，评估是否有值得 fork/cherry-pick 的版本或改动。
- 2026-06-14T08:44:37+08:00：在开新 Cycle 前整理历史归档：补齐 C6/C7 reconstructed `cycle.yaml` 与摘要，将 C6/C7/C8 的 prompts、reports、reviews、plan-state 和 C8 state/PROGRESS/cycle 归档到 `.pipeline/archives/`，保留持久 config/log/knowledge/release/patches 文件原位。
- 2026-06-14T08:49:50+08:00：启动 C9 `/hw:plan`。读取项目配置、全局配置、C9 cycle/state 和架构索引；进入 P0 Configure，准备确认是否复用项目级 `strict` worker separation、`/hw:start` 与 `/hw:resume` 执行子工作器授权、balanced automation 和默认交互式规划。
- 2026-06-14T08:52:58+08:00：P0 Configure 已确认复用当前项目配置。Discover 第一轮完成：用户希望 Deep Research 支持原始报告、进度面板、完整详细报告、递归探索、追问和深入研究；验证场景为手动调研“当前大模型市场最新模型、性能和定价”。初步架构判断：Hypo-Workflow 适合承载可移植语义和产物契约，Codex-VSP 负责平台相关 Subagent 调度、进度 UI、递归执行和报告呈现。
- 2026-06-14T08:55:25+08:00：Discover 第二轮完成：Deep Research 入口归属由代码调研后推荐；递归执行需要预算上限、重要性剪枝、最大深度候选 10、Subagent 上限候选 128；产物布局由 Agent 设计，优先可读性、原始报告保留和可追踪性。
- 2026-06-14T08:56:26+08:00：完成 Discover 代码调研。当前本仓已有 `/hw:*` routing helper、Workflow runtime guidance、`request_user_input`、Subagent 执行基础、active Subagent 面板、Prompt/Report viewer、Workflow Markdown surface 和 `.pipeline` 文件 helper。推荐路线：C9 先实现 Codex-VSP 可复用 Deep Research engine、UI surface 和 artifact writer，同时产出 Hypo-Workflow `/hw:research` 接入建议；是否真正新增 `/hw:research` 留给 Hypo-Workflow 仓处理。
- 2026-06-14T08:58:33+08:00：Discover 第三轮完成：Deep Research 必须使用 Question Tool 澄清，必须支持 follow-up，并需要进度面板手动控制；验证允许真实联网；Codex 更新目标提升为直接把 fork 基础版本更新到合适的上游 Codex 版本，而不是仅产出调研报告。
- 2026-06-14T08:58:33+08:00：检查当前 git/版本基线：本地 `main` 位于 `af11d8a308`，版本为 `0.134.0-vsp.7.2`，本地最新 VSP tag 为 `v0.134.0-vsp.7.2`；远端包含 GitLab `origin` 与 GitHub `upstream`。当前工作区有归档整理和既有未提交改动，后续上游基线升级必须规划干净工作区或隔离 worktree/branch 门禁。
- 2026-06-14T09:01:34+08:00：Discover 完成并收束范围：C9 主交付为完整上游 Codex 基线升级；Deep Research 在本 Cycle 做设计/原型，不作为完整生产交付。执行升级前需要先提交或检查点当前归档与规划改动。进入 Technical Stack 阶段，先确认 upstream 最新版本/标签和本仓升级技术边界。
- 2026-06-14T09:02:46+08:00：Technical Stack 初稿完成。上游 GitHub latest release/tag 为 `0.140.0-alpha.18` / `rust-v0.140.0-alpha.18`，latest upstream main 为 `51316ead4a434e97928d1331e446d4bf274d44a5`；本地 VSP 版本为 `0.134.0-vsp.7.2`。建议在执行前先提交/检查点当前归档与规划改动，再建升级分支；目标基线默认选 latest alpha，除非用户要求 latest stable `0.139.0`。
- 2026-06-14T09:08:14+08:00：Technical Stack 已确认：上游目标基线为 `rust-v0.140.0-alpha.18`；C9 完成后需要发布 GitLab release；Deep Research 在 C9 只做 Workflow 需求简报/设计，不做 Codex-VSP 原型或生产实现。
