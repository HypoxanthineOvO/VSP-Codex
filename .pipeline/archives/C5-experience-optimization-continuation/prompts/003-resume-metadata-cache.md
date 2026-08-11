# M3 Resume Metadata Cache + Progressive Verification

## Objective

实现 resume metadata cache、recent preview cache 和 progressive stale-cache verification，让历史列表先快速显示可用名称/时间/摘要，再后台校验 stale 数据。

## Source Of Truth

- C4 顺延项：resume metadata cache、recent preview cache、progressive stale-cache verification。
- C4 已修 canonical Rename 和空 `model_provider` hotfix；C5 要把历史列表性能和一致性产品化。

## Subworker Assignment Plan

项目配置为 strict worker separation，但当前 Codex 子代理工具仅允许在用户明确要求 sub-agent/parallel agent 时使用。除非用户补充授权，本 Milestone 由主代理执行并在报告中记录降级证据。

## Technical Solution

在现有 rollout/session index 或 TUI resume list 附近增加轻量 metadata cache。cache 保存 thread id、canonical name、updated_at、preview snippet、provider/model display、source mtime/hash 等校验字段。resume 首屏先读 cache/recent preview，再渐进式校验实际 thread metadata；stale 项更新 UI，不阻塞首屏。

## Technical Route

1. 梳理 TUI resume picker、app-server thread list、rollout session index 的 metadata 来源。
2. 设计 cache 文件位置和 schema，避免存储敏感 transcript 正文；snippet 长度受限。
3. 在 rename、thread read/list 或 session close 路径更新 cache。
4. resume picker 首屏优先读取 cache，并以 progressive verification 校验 mtime/hash/canonical metadata。
5. 处理 stale/ missing/corrupt cache：静默 fallback 到现有扫描，并重建 cache。
6. 补大历史 fixture、rename 后 cache 更新、空 provider 兼容、corrupt cache fallback、渐进校验测试。

## Validation Path

- `just test -p codex-tui resume`
- `just test -p codex-rollout`
- `just test -p codex-app-server thread`
- 性能证据脚本更新：resume cached first paint vs full verification。

## Audit Focus

- cache 不成为 canonical truth；canonical metadata 仍是最终来源。
- stale verification 必须能修正 UI。
- cache 不保存过长或敏感正文。
- 失败时不能比现有 resume 更脆弱。
