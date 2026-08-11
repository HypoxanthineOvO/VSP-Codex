# M11 Ask UI 卡片式体验重构与本地化

## Objective

把 Ask UI 从当前“能用但层级弱、英文混杂、列表感强”的状态，重构为清晰、中文化、卡片式、可键盘操作的问答体验。重点解决大标题、小问题标题、问题正文和选项之间区分度不足的问题。

## Source Of Truth

- C3 P2 extension discussion on 2026-05-27.
- Depends on M1-M4 Ask protocol/app-server/TUI foundations and M9 native Ask UI capability repair.
- User preference: card-based options, clear hierarchy, richer but restrained color, higher panel, Chinese UI text.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns focused TUI behavior tests and snapshots before implementation.
  - Must cover card rendering, multi-question tabs, Other/Type something, ranking insertion, unanswered confirmation, Chat About This pending behavior, localization, keyboard flow, and Ask/Plan coexistence.
  - Must define a manual TUI acceptance script for the user.
- `implement`
  - Owns Ask UI layout/render/state/key handling changes.
  - Must preserve app-server/core answer contracts and pending Ask semantics.
  - Must avoid growing central TUI orchestration files when a local Ask UI module can own the behavior.
- `audit`
  - Checks visual hierarchy, keyboard consistency, localization coverage, snapshot quality, pending Ask correctness, and no regression in fallback/native Ask behavior.
  - Must reject if UI remains list-like, English text remains in zh-CN Ask flows, unanswered submission is ambiguous, or Chat About This loses the pending Ask.

## Technical Solution

Redesign the TUI Ask overlay around four explicit information layers:

1. Ask title:
   - Uses request-level `summary`.
   - Strongest visual weight in the Ask panel.

2. Ask description:
   - Uses request-level `description`.
   - Sits below the title and is visually weaker than the title.

3. Current question:
   - `header` is the small question title.
   - `question` is the question body.
   - Question area must be visually separated from option cards.

4. Option cards:
   - Each option is a full-width, single-column card.
   - `label` is the card title.
   - `description` is card secondary text.
   - Cards use border + light background state styling.

## Confirmed Interaction Requirements

- Multi-question navigation:
  - Header area includes short-title tabs.
  - Tabs show current, answered, unanswered, and total progress.
  - Tabs must not overpower the current question body.

- Other / Type something:
  - Every question has an Other/Type something affordance.
  - Chinese UI should show equivalent Chinese text such as `其他` / `输入其他答案`.
  - SingleChoice and MultiChoice support Other.
  - Ranking also supports Other and must allow insertion into the ranking position.
  - Other appears as a special card and expands inline input when selected.

- Chat About This:
  - This is a question-level action, not an answer option.
  - Behavior: pause Ask and return to normal conversation for discussion.
  - The Ask remains pending and can be resumed after the discussion.
  - Chinese UI should show equivalent text such as `讨论这个问题`.

- Unanswered submission:
  - Do not silently submit unanswered questions.
  - Show an inline warning panel inside the Ask panel.
  - List unanswered questions.
  - Allow returning to modify or explicitly confirming submission with empty answers.

- Keyboard model:
  - `Space` selects/toggles the focused card.
  - `Enter` advances to the next question or submits when ready.
  - `Tab` moves focus through tabs, cards, inputs, and action controls.
  - `Esc` is two-step:
    - first exits input/confirmation state or returns one layer;
    - second interrupts Ask.
  - When multiple options exist, show numeric/letter shortcuts on cards.

- Layout and coexistence:
  - Ask bottom pane defaults taller than the current compact rendering.
  - Options scroll when content exceeds available height.
  - When Ask and Plan are both active, Plan renders as a summary-only state to avoid squeezing Ask.
  - Ask submit, tool result resolution, Plan refresh, and input restoration should avoid visible flicker or avoidable blocking.

- Localization:
  - zh-CN Ask UI must not show English footer/action/status text.
  - Localize Other/Type something, Chat About This, unanswered warning, return/modify/continue submit, current/answered/unanswered, keyboard hints, and empty-answer confirmation.

## Visual Direction

- Single-column full-width option cards.
- Border + light background hybrid style.
- Clear state distinction for:
  - current question;
  - answered question;
  - unanswered question;
  - focused card;
  - selected card;
  - Other input expanded state;
  - warning/confirmation state.
- Richer than the current one-note palette, but restrained and consistent with TUI style conventions.
- Question text and option descriptions must never look like the same layer.

## Technical Route

1. Audit current Ask UI render/state/key modules and snapshots.
2. Add red tests/snapshots for hierarchy, card rendering, localization, and unanswered confirmation.
3. Introduce option-card render primitives local to the Ask UI module.
4. Rework request summary/description/current-question rendering into explicit sections.
5. Add short-title tab navigation and answered/unanswered indicators.
6. Implement Other cards for single/multi/ranking, including ranking insertion.
7. Implement Chat About This as a pending-Ask-preserving pause path.
8. Implement inline unanswered warning panel with return/confirm actions.
9. Normalize keyboard flow for Space/Enter/Tab/Esc and shortcuts.
10. Increase Ask panel height policy and ensure scroll behavior.
11. Add Ask/Plan coexistence summary behavior.
12. Localize visible Ask strings for zh-CN.
13. Run focused tests, review snapshots, and prepare manual TUI script.

## Constraints

- Preserve existing app-server/core request/response schemas unless a schema change is explicitly required and covered by app-server protocol updates.
- Preserve unsupported-client fallback.
- Preserve pending Ask resume behavior.
- Do not leak secret/freeText values in visible history.
- Avoid UI cards inside unrelated cards; card styling is for repeated option items inside the Ask tool surface.
- Do not use hardcoded white; follow local TUI style conventions.

## Validation Path

Automated:

- `just test -p codex-tui request_user_input`
- Additional focused tests/snapshots for:
  - card hierarchy;
  - multi-question tabs;
  - singleChoice cards;
  - multiChoice cards;
  - freeText input card;
  - ranking + Other insertion;
  - unanswered inline confirmation;
  - Chat About This pending behavior;
  - Ask/Plan summary coexistence;
  - zh-CN localized strings;
  - Space/Enter/Tab/Esc keyboard behavior.
- `cargo insta pending-snapshots -p codex-tui`
- `just fmt`
- Scoped `just fix -p codex-tui` and any other changed crates.

Manual:

- User runs a real TUI Ask with multiple questions.
- User tests selecting cards, using Other, leaving a question blank, returning to modify, Chat About This, and keyboard navigation.
- User verifies Plan summary coexistence and no obvious card/submit/refresh lag.

Pass signal:

- Ask title, description, current question, and option cards are visually distinct.
- All visible Ask UI text in zh-CN flow is localized.
- Other and Chat About This behave as specified.
- Unanswered submission cannot happen silently.
- Keyboard flow is predictable.
- Manual TUI acceptance passes.

Fail signal:

- UI still looks like a flat list.
- English hints remain in zh-CN Ask UI.
- Question text and options remain visually ambiguous.
- Other/ranking insertion is confusing or broken.
- Chat About This resolves or loses the pending Ask accidentally.
- Ask/Plan coexistence causes severe squeezing, flicker, or blocking.

## Completion Criteria

- M11 report is written to `.pipeline/reports/011-ask-ui-card-experience.md`.
- Snapshot changes are reviewed and accepted when intentional.
- Manual acceptance script and result are recorded.
