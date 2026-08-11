# C9 M7 Test Worker Review

## verdict

通过。

M7 implement worker 生成的两个 artifact 均存在且非空：

- `.pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md`
- `.pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml`

验证结果显示：

- conflict inventory 覆盖了 SQLite logging、protocol/app-server、core workflow/runtime、TUI、CLI/VSCode、plugins/adapters、docs/CI/release tooling。
- equivalence matrix 可由 YAML 解析，包含 `replace` / `preserve` / `defer` 三类，且每个条目均包含 `surface`、`evidence`、`action`、`risk`、`next_milestone`。
- 高风险 VSP surface 均已被分配到 replace/preserve/defer，并带有后续 milestone。
- 官方 replacement claim 引用了具体 upstream commit 与路径证据，尤其是 SQLite logging 相关的 `d81fcdf8ef`、`e98d43ac37`、`91e6da943b`、`63f8f547c0` 及对应文件路径。
- M7 non-goals 明确声明不做源码合并、不改版本、不刷新 release artifact、不 push/tag/publish/release。

## reviewed_refs

- `.pipeline/prompts/c9-007-conflict-inventory-and-upstream-fix-baseline.md`
- `.pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md`
- `.pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml`
- upstream tag object `rust-v0.143.0-alpha.25`
- upstream commit `d634f56262d9af9398fee0573a29dd178ddcb1ac`

## commands

只读验证命令：

```bash
wc -c .pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md .pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml
file .pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md .pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml
sed -n '1,260p' .pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md
sed -n '261,520p' .pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md
sed -n '1,260p' .pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml
rg -n "SQLite|Protocol|App-Server|Core Workflow|Runtime|TUI|CLI|VSCode|Plugins|Adapters|Docs|CI|Release|Non-Goals|does not merge|does not change|does not push|tag|release|version" .pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md
rg -n "replace:|preserve:|defer:|surface:|evidence:|action:|risk:|next_milestone:|No product source merge|No package version edit|No push|tag|release|commit|codex-rs/|\\.opencode|\\.gitlab-ci" .pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml
python3 - <<'PY'
import yaml
from pathlib import Path
p=Path('.pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml')
data=yaml.safe_load(p.read_text())
print(type(data).__name__)
print('top_keys=', sorted(data.keys()))
required=['surface','evidence','action','risk','next_milestone']
all_ok=True
for bucket in ['replace','preserve','defer']:
    items=data.get(bucket)
    print(bucket, 'count=', len(items) if isinstance(items,list) else 'NOT_LIST')
    if not isinstance(items,list) or not items:
        all_ok=False
        continue
    for i,item in enumerate(items):
        missing=[k for k in required if k not in item]
        empty=[k for k in required if k in item and not item[k]]
        if missing or empty:
            all_ok=False
            print('BAD', bucket, i, 'missing=', missing, 'empty=', empty, 'surface=', item.get('surface'))
print('schema_required_fields_ok=', all_ok)
print('surfaces=')
for bucket in ['replace','preserve','defer']:
    for item in data.get(bucket,[]) or []:
        print(f"- {bucket}: {item.get('surface')} -> {item.get('next_milestone')}")
PY
git show --stat --oneline --no-renames d81fcdf8ef e98d43ac37 91e6da943b 63f8f547c0 -- codex-rs/state/src/log_db.rs codex-rs/state/src/log_db_filter_tests.rs codex-rs/app-server/src/lib.rs codex-rs/tui/src/lib.rs codex-rs/codex-api/src/endpoint/responses_websocket.rs codex-rs/otel/src/events/session_telemetry.rs
git show --no-patch --decorate --oneline rust-v0.143.0-alpha.25
git rev-parse rust-v0.143.0-alpha.25^{commit}
git cat-file -t rust-v0.143.0-alpha.25
git cat-file -p rust-v0.143.0-alpha.25
```

补充说明：一次 `python` 命令因本机无 `python` 可执行文件失败，随后用 `python3` 完成等价 YAML parse 与 schema 检查；该失败不影响 artifact 判定。

## coverage checklist

- [x] conflict inventory 存在且非空，大小为 11913 bytes。
- [x] equivalence matrix 存在且非空，大小为 12825 bytes。
- [x] YAML parse 通过，顶层 key 包含 `cycle`、`milestone`、`target_upstream_tag`、`target_upstream_commit`、`generated_from`、`non_goals`、`replace`、`preserve`、`defer`。
- [x] `replace` 存在且非空，共 2 项。
- [x] `preserve` 存在且非空，共 8 项。
- [x] `defer` 存在且非空，共 7 项。
- [x] 所有 matrix 项均包含 `surface`、`evidence`、`action`、`risk`、`next_milestone`，且未发现空值。
- [x] SQLite logging 覆盖：inventory 有专节，matrix 将 persistent log filtering 与 Responses WebSocket/session telemetry noise reduction 分配到 `replace -> M8`。
- [x] protocol/app-server 覆盖：inventory 有专节，matrix 将 Question Tool / request_user_input、workflow markdown bridge 标记为 preserve，并将 generated schemas 与 upstream app-server additions 标记为 defer。
- [x] core workflow/runtime 覆盖：inventory 有专节，matrix 将 Hypo-Workflow routing/state contract、goals/plan freshness 标记为 preserve，并将 alpha.25 core runtime improvements 标记为 defer。
- [x] TUI 覆盖：inventory 有专节，matrix 将 broad UX churn 标记为 `defer -> M11`。
- [x] CLI/VSCode 覆盖：inventory 有专节，matrix 将 VSP CLI entrypoint、VSCode Remote behavior 标记为 preserve，并将 code-mode host IPC / exec-server lifecycle 标记为 defer。
- [x] plugins/adapters 覆盖：inventory 有专节，matrix 将 OpenCode / Hypo-Workflow adapters 标记为 preserve，并将 plugin/marketplace/connector/skill-loader convergence 标记为 defer。
- [x] docs/CI/release tooling 覆盖：inventory 有专节，matrix 将 GitLab release / VSP package tooling 标记为 preserve，并将 alpha.25 docs/CI/install/release tooling 标记为 defer。
- [x] 无高风险 VSP surface 未分配：matrix 的 owner domain 集合覆盖 `SQLite logging`、`protocol/app-server`、`core workflow/runtime`、`TUI`、`CLI/VSCode`、`plugins/adapters`、`docs/CI/release tooling`。
- [x] 官方 replacement claim 有具体 upstream commit/path 证据：`git show --stat` 确认四个 SQLite/noise commits 分别触及 artifact 中列出的具体路径。
- [x] upstream pin 校验通过：`rust-v0.143.0-alpha.25` 是 annotated tag，tag object 为 `cdc56331691cbc9260be1724b775d6443c578075`，实际指向 commit `d634f56262d9af9398fee0573a29dd178ddcb1ac`，与 artifact 中的 `target_upstream_commit` 一致。
- [x] M7 non-goals 明确：inventory 和 matrix 均声明不做 product source merge、不做 package version edit、不刷新 release artifact、不 push/tag/publish/release。

## issues/risks

- 未发现阻塞性问题。
- 残余风险：本次 review 只验证 artifact 的存在性、结构、覆盖面和可追溯证据，不重新执行完整 `git diff` / `git merge-tree` 统计，因此 conflict count 的数值准确性仍依赖 implement worker 的原始分析命令。
- 残余风险：matrix 中部分 preserve/defer 证据引用 `.plan-state`、architecture decision 或 `merge-tree` 摘要，已满足 M7 planning artifact 要求；后续 M8-M12 在真正合并时仍需以对应代码 diff 和测试结果重新确认行为等价。
