---
kind: execution
cycle: C27-fork-test-ci-simplification
status: completed
updated: 2026-08-09T21:30:08+08:00
---

# C27 Execution

## M1 Start

- 用户明确选择 fork 迭代效率优先，不复刻上游全面 blocking CI。
- 使用 C26 exhaustive inventory 作为只读证据，不继承 C26 的 M4-M5 任务或“327 failures 必须在 C27 修完”的目标。
- 当前工作区包含 C26 未验收 Rust/schema/snapshot 改动；C27 不回滚、不覆盖，所有删除必须能与其区分。
- 当前仓库没有 `hypo-workflow` CLI；按已安装 Cycle/Goal skill 的语义文件规范写入记录。
- 不执行 commit、push、发布、安装、部署或重启。

## M1 Result

- GitLab VSP gate只有`vsp-smoke`一个job：locked build `vsp-codex`后运行CLI/TUI/app-server smoke。
- 最近成功pipeline #346约4分12秒；已有增量binary的完整smoke实测1.27秒，等待主要来自Rust冷编译。
- 29个GitHub workflows与Bazel矩阵不进入GitLab pipeline，删除不会改善VSP CI时间，反而增加upstream sync冲突，因此保留并明确为inherited/manual。
- 现有smoke已覆盖`--version`、`--help`、`exec/resume --help`、TUI可见首帧、app-server initialize与thread/list；符合用户要求的最小正常交互。

## M2 Implementation

- `just test`改为必须提供package/target/filter；无参数时快速拒绝，避免误启动12,000+ cases。新增显式`just test-all`。
- 新增`just vsp-ci`复现required build/smoke；新增`just vsp-release-check`运行package、installer、session compatibility与preview release的快速Python合同。
- 删除21个已确认orphan TUI snapshots，以及与完整字符串assertion重复的1个doctor snapshot。
- 删除3个无价值tests：重复doctor snapshot assertion、只比较静态Bazel label、仅测试自身且生产不使用的SemVer helper。
- TypeScript aborted-stream test改为要求iterator自身reject，正常结束不再被测试自抛错误伪装成成功。
- 4个读取external sibling Hypo-Workflow fixture/manifest的tests加显式ignore reason，保留为手工compatibility检查；3个只检查本仓源码的快速合同继续进入default run。

## M3-M5 Result

- GitLab required job保持单一build+smoke，增加10分钟硬timeout；GitLab CI lint通过。
- `just vsp-ci`从current source重新编译并完成全部smoke，95.57秒；单独smoke 1.27秒。
- 临时GNU package directory 1.16秒生成，package binary与source SHA256一致；包内smoke 1.24秒。
- release Python 46/46、CLI doctor 17/17、V8 5/5、TypeScript abort 5/5通过。
- Hypo default 3 passed/4 ignored；显式manual在缺sibling checkout时3 passed/4 failed，前置条件缺失可见。
- Rustfmt、Prettier、Justfile、YAML/front matter、diff、snapshot hygiene通过；GitLab CI YAML valid。
- `RESULT.md`写入完整结论、变更、验证、预期、问题与风险；Goal进入waiting-review。

## Final Result Rejected

- 用户指出v1没有给本地测试建立用途分组，也没有把选择规则写给AI。
- 之前错误假设是“有focused命令就等于AI知道该跑什么”；实际仍可能把`test-all`当兜底，报出大量旧失败后不处理。
- M1-M3结果保留；M4恢复in_progress，M5恢复pending。
- 修订范围：机器可读group manifest、统一runner、Just入口、AGENTS测试选择与失败责任规则；不扩展到C26产品修复。

## M4 Revision Result

- `scripts/vsp_test_groups.json`定义14个behavior groups：smoke、CLI、core、app-server、exec、protocol、TUI、persistence、release、TypeScript SDK、security、V8 runtime、Hypo external和upstream-all。
- 每组记录purpose、value、useWhen、prerequisites、ciBlocking与commands；focused groups要求filter，external要求env，全量要求reason。
- `scripts/vsp_test_groups.py`提供list/explain/validate/run，使用argv而非shell执行；运行前打印合同和失败责任；TUI检查pending snapshots。
- focused Rust groups使用nextest `--no-tests=fail`，不存在的filter退出4，不允许0-test假绿。
- `just test-groups`和`just test-group`成为首选入口；`just test-all`通过runner且无reason拒绝；原`just test`保留为manifest缺组时的focused escape hatch。
- 根`AGENTS.md`要求AI先选最小组、禁止全量兜底、修复相关失败、只在真实阻塞时停止，并禁止把原始全量失败清单当结果。
- 实际通过：smoke；CLI 16；core 3；app-server 1；exec 4；protocol 273；TUI 1且0 pending；persistence 107 Rust+7 Python；release 46；SDK 5；security 86；V8 5；Hypo external 4。
- gate验证：空filter、缺env、缺reason和不存在test-name都非零失败；upstream-all仅做reason+dry-run，没有启动全仓。

## M5 Revision Result

- Prettier格式化manifest与CI文档，Ruff格式化runner；重新验证manifest与全部静态检查。
- 复核Windows可用性后，persistence Python command改为`{python}`并由runner解析为`sys.executable`；smoke显式记录Unix PTY前置条件。
- RESULT v2删除“全量失败数作为交付结果”的叙述，改为明确AI选择链、失败责任、分组实跑证据与剩余边界。
- C27重新进入waiting-review。

## Final Acceptance

- 用户接受C27 v2：14组本地测试目录、AI选择与失败责任、单job required CI、精确清理和剩余边界均确认。
- `SUMMARY.md`固化接受结果与证据；C27关闭。
- 接受不授权也未执行commit、push、release、install、deploy或restart。
