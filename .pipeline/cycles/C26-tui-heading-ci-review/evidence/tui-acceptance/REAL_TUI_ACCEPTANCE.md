# C26 Real TUI Acceptance

- Date: 2026-08-09 (Asia/Shanghai)
- Entry point: `RUST_LOG=trace just vsp-codex -c log_dir=...`
- Terminal: real PTY, `xterm-256color`, truecolor, 100 columns by 34 rows
- Isolation: temporary `CODEX_HOME` with copied credentials/config; no primary session state used
- Model: `gpt-5.6-sol`, xhigh reasoning
- Exit: `/exit`, process status 0

## Prompt Surface

The prompt requested an exact Markdown response containing H1, H2, and H3 headings, a long
unbroken token, a fenced Rust code block, and a final end marker. Text and Enter were sent in
separate PTY writes as required by the TUI test procedure.

## Observed Result

- H1 rendered centered and bold.
- H2 rendered bold with balanced rules on both sides.
- H3 rendered left-aligned and bold without a rule.
- The long unbroken token wrapped within the terminal width without truncation or overlap.
- The Rust code line remained aligned and readable.
- `End marker.` rendered after the code block, proving the final content was not clipped.
- The composer and status line stayed below the transcript without covering response content.
- Trace log inspection found no `ERROR`, panic, or fatal entry.

## Conclusion

PASS. The C26 heading hierarchy and width-safety contract works in a real terminal, not only in
snapshot rendering.
