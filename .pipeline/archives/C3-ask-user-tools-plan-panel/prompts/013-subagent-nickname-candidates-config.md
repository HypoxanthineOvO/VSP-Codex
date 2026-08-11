# M13 Subagent 昵称候选池配置化

## Objective

把 Subagent nickname candidates 从分散的内置/硬编码逻辑整理为可配置契约：

1. 用户可以通过全局配置和项目配置自定义 Subagent 名字池。
2. role-level `nickname_candidates` 继续可用，并优先覆盖默认池。
3. core spawn 返回 nickname 与 TUI active fallback 名字使用同一套配置派生结果。
4. 没有配置时保留内置默认行为，但公共默认池改为中国古代人物名，避免发布后使用私人姓名。
5. 个人自用配置示例应能保留当前中文名单并追加 `陈阳宾`、`尹一帆`。

## Source Of Truth

用户要求：

- “自定义的名字最好写到配置文件里，因为后面发布的时候，可能每个人都想自己定义不同的 Subagent 名字。”
- 配置层级已确认：项目 + 全局都支持。
- fallback 策略已确认：没有配置时保留内置默认名单。
- 覆盖范围已确认：core spawn nickname 与 TUI active fallback 都接配置。
- 默认池最终决策：使用中国古代人物名。
- 用户个人池：在当前自用中文名单基础上追加 `陈阳宾` 和 `尹一帆`。

本地现状：

- `codex-rs/config/src/config_toml.rs` 已有 `[agents.<role>].nickname_candidates` 字段。
- `codex-rs/core/src/agent/role.rs` 已有 built-in role nickname candidates，但内容包含私人中文名。
- `codex-rs/core/src/agent/control.rs` 的 spawn nickname 会优先使用 role config，缺失时 fallback `AGENT_NAMES`。
- `codex-rs/tui/src/chatwidget/active_subagents.rs` 仍有独立硬编码 `SUBAGENT_FALLBACK_NAMES`。
- `codex-rs/core/src/config/agent_roles.rs` 当前 nickname candidate 校验只允许 ASCII，但内置和用户需求都需要中文；这是 M13 必须修复的不一致。

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns red tests and validation evidence before implementation.
  - Scope:
    - `codex-rs/core/src/config/config_tests.rs`
    - `codex-rs/core/src/agent/role_tests.rs`
    - `codex-rs/core/src/agent/control_tests.rs`
    - `codex-rs/tui/src/chatwidget/tests/app_server.rs` or adjacent active Subagent fallback tests.
    - app-server/config schema fixtures if config shape changes.
  - Must prove current behavior fails where feasible:
    - Chinese configured `nickname_candidates` are rejected today.
    - TUI active fallback still uses its own hardcoded pool.
    - role-level override remains honored.
  - Expected artifact:
    - `.pipeline/reviews/C3/M13/test.md`

- `implement`
  - Owns production/runtime/config edits only.
  - Allowed likely paths:
    - `codex-rs/config/src/config_toml.rs`
    - `codex-rs/core/src/config/mod.rs`
    - `codex-rs/core/src/config/agent_roles.rs`
    - `codex-rs/core/src/agent/role.rs`
    - `codex-rs/core/src/agent/control.rs`
    - `codex-rs/tui/src/chatwidget/active_subagents.rs`
    - TUI/app wiring needed to pass effective nickname candidates into active fallback rendering.
    - Generated schema files only if config schema changes.
  - Must not edit tests/snapshots/review artifacts.
  - Expected artifact:
    - `.pipeline/reviews/C3/M13/implement.md`

- `audit`
  - Read-only final review.
  - Checks config precedence, Unicode validation, default pool contents, role override compatibility, TUI/core consistency, schema updates, and test evidence.
  - Expected artifact:
    - `.pipeline/reviews/C3/M13/audit.md`

- Main agent
  - Orchestrates workers, integrates outputs, runs validation, updates lifecycle state, and closes the milestone.
  - Must not satisfy both test and implement roles locally while strict worker separation is active.

## Technical Solution

### Configuration contract

Preserve existing role-level shape:

```toml
[agents.worker]
nickname_candidates = ["陈阳宾", "尹一帆"]
```

Add a default/global pool under `[agents]`, naming should follow existing config style. Preferred shape:

```toml
[agents]
nickname_candidates = ["孔丘", "孟轲", "墨翟"]
```

Precedence:

1. Role-level `agents.<role>.nickname_candidates`
2. Default/global `agents.nickname_candidates`
3. Built-in role candidates when preserving role-specific defaults is still desired
4. Built-in general default candidates

Implementation may choose 2/3 ordering differently only if tests and documentation make the behavior explicit. The preferred user-facing behavior is:

- A role-specific list wins for that role.
- A top-level `[agents].nickname_candidates` acts as the user's general default for roles without explicit lists.
- If no user/project config exists, built-in role lists/defaults preserve a curated experience.

### Default public pool

Replace public hardcoded/private-name defaults with Chinese ancient figures. Keep the list moderate and display-friendly:

```text
孔丘, 孟轲, 墨翟, 庄周, 屈原, 司马迁, 张衡, 蔡伦,
祖冲之, 郦道元, 李白, 杜甫, 韩愈, 柳宗元, 苏轼, 沈括,
李清照, 辛弃疾, 关汉卿, 徐霞客, 宋应星, 曹雪芹
```

Avoid controversial or highly modern political figures in the public default pool.

### Personal pool example

Do not hardcode the user's personal names into public defaults. Ensure this config works:

```toml
[agents]
nickname_candidates = [
  "娄鑫",
  "李萌",
  "杨超",
  "朱昊骋",
  "李超凡",
  "贺云翔",
  "吴浩宇",
  "丁麒涵",
  "李宇轩",
  "李铭杰",
  "闫启航",
  "张雍治",
  "蒋烨",
  "陈泉宇",
  "王珺",
  "沈博宇",
  "陈阳宾",
  "尹一帆",
]
```

### Unicode validation

Relax nickname validation to support Chinese names and other normal Unicode display names.

Keep rejecting:

- empty lists
- blank names after trim
- duplicates after trim
- control characters
- newline/tab-heavy names
- unsafe markup/control delimiters if local conventions require it

Do not keep the current ASCII-only restriction.

### Core/TUI consistency

Core spawn nickname:

- Continue using role-level config when present.
- Use top-level default candidates when role-level is absent.
- Use built-in default pool when no config exists.

TUI active fallback:

- Remove or demote `SUBAGENT_FALLBACK_NAMES` as an independent source of truth.
- Use effective configured/default nickname candidates for deterministic fallback when metadata is missing.
- If passing full config into the active panel is too invasive, introduce a narrow TUI-facing nickname pool in `ChatWidget`/`ActiveSubagentPanelState` initialized from config.
- Preserve raw id avoidance in active compact rows.

## Technical Route

1. Test worker writes red tests:
   - Config accepts Chinese `agents.nickname_candidates`.
   - Config rejects blank/duplicate/control-character names.
   - Role-level candidates still override the default pool.
   - Core spawn uses top-level configured pool when role-level is absent.
   - TUI active fallback uses configured pool instead of `SUBAGENT_FALLBACK_NAMES`.
   - Default built-in pool contains the selected ancient Chinese names and excludes the previous private list as public defaults.

2. Implement config model:
   - Add `AgentsToml.nickname_candidates: Option<Vec<String>>`.
   - Add corresponding resolved field in core `Config` if needed.
   - Normalize candidates through shared validation.
   - Update config schema if generated schema is affected.

3. Implement built-in/default pool:
   - Move default nickname list into a shared core-level helper if practical.
   - Replace public defaults with the ancient Chinese name pool.
   - Preserve role-level built-in defaults only if they are not private-name based, or replace those with slices from the new public pool.

4. Wire core spawn:
   - Update `agent_nickname_candidates(config, role_name)` precedence.
   - Keep existing reservation behavior and ordinal suffix behavior.
   - Add tests for configured default pool and role override.

5. Wire TUI fallback:
   - Add effective nickname pool storage where `ChatWidget` or active panel rendering can access it.
   - Initialize from resolved config.
   - Use configured/default pool in `fallback_subagent_name`.
   - Keep deterministic hash selection.

6. Validate:
   - Run focused core config/agent tests.
   - Run focused TUI active_subagents tests.
   - Run schema generation/check if config schema changed.
   - Run `just fmt`.
   - Run scoped `just fix` for touched crates.
   - Build `vsp-codex`.

## Research Required

- status: resolved
  evidence:
    - Existing `[agents.<role>].nickname_candidates` exists in `config_toml.rs`.
    - Existing normalization in `agent_roles.rs` is ASCII-only and conflicts with current built-in Chinese names and user requirements.
    - TUI `active_subagents.rs` has independent `SUBAGENT_FALLBACK_NAMES`.
    - Core spawn candidate resolution already exists in `agent/control.rs`.

## Risks And Alternatives

- Risk: Config shape overlaps with flattened role names under `[agents]`.
  - Mitigation: verify serde flatten behavior and unknown-field handling; choose a non-conflicting field name.
- Risk: Unicode validation becomes too permissive.
  - Mitigation: reject control characters, blank values, duplicates after trim, and unsafe separators.
- Risk: Passing config into TUI active panel causes broad plumbing.
  - Mitigation: keep a narrow nickname pool field initialized in ChatWidget construction or app config setup.
- Risk: Changing built-in defaults affects snapshots.
  - Mitigation: update focused snapshots/tests intentionally and avoid unrelated UI churn.
- Alternative rejected: keep TUI fallback hardcoded.
  - Reason: user explicitly wants published users to customize names; separate hardcoded fallback would make core/TUI inconsistent.

## Validation Path

Automated:

- `cd codex-rs && just test -p codex-core nickname_candidates`
- `cd codex-rs && just test -p codex-core agent_role_nickname_candidates`
- `cd codex-rs && just test -p codex-tui active_subagents`
- `cd codex-rs && cargo insta pending-snapshots --workspace`
- If config schema changes: `cd codex-rs && just write-config-schema`
- `cd codex-rs && just fmt`
- `cd codex-rs && just fix -p codex-core -p codex-config -p codex-tui`
- `cd codex-rs && cargo build --bin vsp-codex`

Manual:

- Put the personal pool config under `[agents].nickname_candidates` including `陈阳宾` and `尹一帆`.
- Spawn multiple Subagents.
- Confirm spawn result and active Subagent fallback use configured Chinese names.
- Remove config and confirm default pool uses ancient Chinese names.

Pass signal:

- Configured Chinese names are accepted.
- Role-level pool overrides default pool.
- Top-level pool applies to roles without explicit lists.
- TUI fallback and core spawn no longer diverge.
- No private names remain in public built-in default pool.

Fail signal:

- Chinese names are rejected by config validation.
- TUI fallback still uses the old hardcoded list.
- role-level config stops working.
- schema or config tests drift without update.

## Audit Focus

- Verify config precedence and backward compatibility.
- Verify Unicode validation is permissive enough for Chinese names but still rejects unsafe values.
- Verify public defaults contain only approved ancient Chinese names.
- Verify user's personal pool is supported by configuration rather than hardcoded.
- Verify core/TUI use the same effective candidate source.
