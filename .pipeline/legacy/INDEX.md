---
kind: legacy-index
status: read-only
---

# Legacy 索引

这些路径保持原样，不是普通 Session 的默认上下文：

- Archives：`.pipeline/archives/`
- Knowledge：`.pipeline/knowledge/`
- Chats：`.pipeline/chats/`
- Patches：`.pipeline/patches/`
- PR：`.pipeline/pr/`
- 旧 Manifest 与 live Delivery：`.pipeline/manifest.yaml`、`.pipeline/runtime/objects/delivery/`

## Live Deliveries

| ID | 类型 | 状态 | 来源 |
| --- | --- | --- | --- |
| chat-compatible-backend-deepseek-explorer | `cycle` | `accepted` | `.pipeline/runtime/objects/delivery/chat-compatible-backend-deepseek-explorer` |
| local-compaction-state-continuity | `cycle` | `accepted` | `.pipeline/runtime/objects/delivery/local-compaction-state-continuity` |
| release-workflow-alpha3-vsp-10-alpha1 | `cycle` | `accepted` | `.pipeline/runtime/objects/delivery/release-workflow-alpha3-vsp-10-alpha1` |
| vsp-10-alpha2-clean-release | `cycle` | `cancelled` | `.pipeline/runtime/objects/delivery/vsp-10-alpha2-clean-release` |
| vsp-11-alpha4-compaction-release | `cycle` | `accepted` | `.pipeline/runtime/objects/delivery/vsp-11-alpha4-compaction-release` |
| vsp-11-responses-stability | `goal` | `accepted` | `.pipeline/runtime/objects/delivery/vsp-11-responses-stability` |
| vsp-codex-long-term-architecture-audit | `cycle` | `accepted` | `.pipeline/runtime/objects/delivery/vsp-codex-long-term-architecture-audit` |
| vsp-daily-stability-repair | `goal` | `accepted` | `.pipeline/runtime/objects/delivery/vsp-daily-stability-repair` |
| vsp-semantic-worker-routing | `goal` | `accepted` | `.pipeline/runtime/objects/delivery/vsp-semantic-worker-routing` |
| vsp-thinking-hooks-repair | `goal` | `accepted` | `.pipeline/runtime/objects/delivery/vsp-thinking-hooks-repair` |
| vsp-workspace-mainline-consolidation | `goal` | `accepted` | `.pipeline/runtime/objects/delivery/vsp-workspace-mainline-consolidation` |

非终态项必须通过旧兼容入口单独处理，不自动转成 Cycle，也不自动接受或拒绝。
