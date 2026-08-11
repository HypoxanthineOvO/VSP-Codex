---
kind: validation-report
cycle: C28-prompt-stack-quality-redesign
milestone: M3
status: completed-with-residual-risk
updated: 2026-08-11T14:40:46+08:00
---

# M3 Final Validation and Omissions Audit

## 结论

本次最终范围（Stage A基础提示/assembly、Stage B System Skills/lifecycle、Stage E Worker Routing）没有发现未接线的产品行为，focused regression与required smoke通过，可以形成release candidate。审计发现并补齐一个发布阻塞遗漏：preview builder会覆盖同版本其他target，且archive默认名固定Linux；现已支持三平台命名、同版本多target保留、严格manifest聚合和手动原生runner workflow。

原C28量化目标没有达到。非工具指令约55,565字符，相对77,567下降28.4%，未达到40%；真实VSPLab首轮21,321 input tokens，相对27,536下降22.6%，未达到25%，cached input仍为3,840。主要原因是用户明确延期Stage C后，根`AGENTS.md`仍占约26K字符。本轮不擅自重开该范围。

## Prompt与Token证据

- 当前`vsp-codex debug prompt-input`：1个developer item（7 parts，16,896字符）、project/environment user context 26,973字符、测试user prompt 13字符。
- 当前base instructions为接受的11,696 bytes；扣除测试prompt后，总非工具instructions约55,565字符。
- 真实VSPLab `gpt-5.6-sol`响应成功并只返回`ok`：21,321 input、3,840 cached input、5 output、0 reasoning output tokens。
- M1基线：77,567 non-tool chars、27,536 input tokens、3,840 cached input tokens。

## 行为验证

| Group | Result | Contract |
| --- | ---: | --- |
| `codex-core-skills` | 121/121 | effective winner、alias、2K cap、loader与metadata |
| initial Multi-Agent assembly | 2/2 | root/subagent policy与单一初始developer聚合 |
| Skill protocol | 2/2 | compact/full guide互斥 |
| compact/lifecycle | 19/19 | active Skill跨compaction/model switch/resume基础历史保留 |
| Worker Routing | 6/6 | trusted overlay、compatibility、secret-safe error、cross-provider child config |
| Multi-Agent V2 spawn | 19/19 | routing/role/override/fork/result/encrypted child行为 |
| explicit plugin list/install | 1/1 | 删除unsolicited recommendation后显式流程仍可用 |
| required smoke | 7/7 | current CLI build、version/help、TUI frame、app-server initialize/thread list |
| release checks | 52/52 | package、installer、session、version、三平台manifest aggregation |
| `cargo check -p codex-cli --bin vsp-codex` | pass | alpha.6 workspace与CLI全依赖图可检查 |
| format/diff/YAML | pass | Rust/Python/Markdown whitespace与workflow YAML syntax |

## 发布遗漏修复

- `build_vsp_preview_release.py`按Rust target选择canonical archive name，并用`(version, channel, target)`保留同版本其他distribution。
- 新增`merge_vsp_release_manifests.py`，拒绝version/channel/upstream/minimum/schema不一致或重复target，并可更新stable manifest。
- 新增`vsp-preview-release.yml`手动matrix：Linux x64 GNU、macOS ARM64、Windows x64 MSVC；每个平台原生构建后由Linux job聚合archive、SHA256与manifest。
- 新增default names、multi-target stable merge、three-platform aggregate、mismatch与duplicate rejection tests。

## 问题与风险

- 真实Subagent smoke连续三轮在root请求进入`spawn_agent`前收到VSPLab `/responses` 502；最后一次5次自动重连后失败。离线resolver和真实child config handler tests通过，但本次不能声明live child inference通过。
- 本机Notion MCP OAuth存在既有`invalid_grant`，不影响VSPLab或release build。
- GitLab仅有一个Linux amd64 runner；macOS/Windows构建必须使用GitHub-hosted原生runner或新增对应GitLab runners。当前GitHub账号无现成Codex mirror，创建mirror属于下一步外部动作。
- `.github` workflow尚需在真实GitHub repository通过workflow parser与三平台runner验证；本地仅证明YAML和脚本合同。
- 原token/character目标需未来Stage C拆分/条件化根AGENTS后重新测量。

## 预期发布行为

alpha.6在三平台使用同一版本、channel、upstream tag与notes URL；每个平台有独立archive与checksum，aggregate manifest同时包含三个target，updater按自己的target选择distribution。Linux仍包含bwrap，Windows包含sandbox helpers，macOS/Windows不错误复用Linux archive名称。
