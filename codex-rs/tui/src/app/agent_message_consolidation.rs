//! Transcript consolidation for finalized streaming agent messages.
//!
//! During streaming, the chat widget emits transient `AgentMessageCell`s so it
//! can animate stable lines into scrollback while keeping the active mutable
//! tail in the bottom pane. Once the answer finishes, the app replaces that
//! trailing run with a single source-backed `AgentMarkdownCell`. This makes the
//! transcript the canonical owner of the raw markdown source used for future
//! resize re-renders.

use std::path::PathBuf;
use std::sync::Arc;

use color_eyre::eyre::Result;

use super::App;
use super::resize_reflow::trailing_run_start;
use crate::app_event::ConsolidationScrollbackReflow;
use crate::history_cell;
use crate::history_cell::HistoryCell;
use crate::pager_overlay::Overlay;
use crate::tui;

impl App {
    pub(super) fn handle_consolidate_agent_message(
        &mut self,
        tui: &mut tui::Tui,
        source: String,
        cwd: PathBuf,
        scrollback_reflow: ConsolidationScrollbackReflow,
    ) -> Result<()> {
        // Walk backward to find the contiguous run of streaming AgentMessageCells that
        // belong to the just-finalized stream.
        let end = self.transcript_cells.len();
        tracing::debug!(
            "ConsolidateAgentMessage: transcript_cells.len()={end}, source_len={}",
            source.len()
        );
        let start = trailing_run_start::<history_cell::AgentMessageCell>(&self.transcript_cells);
        if start < end {
            tracing::debug!(
                "ConsolidateAgentMessage: replacing cells [{start}..{end}] with AgentMarkdownCell"
            );
            let consolidated: Arc<dyn HistoryCell> =
                Arc::new(history_cell::AgentMarkdownCell::new(source, &cwd));
            self.transcript_cells
                .splice(start..end, std::iter::once(consolidated.clone()));

            if let Some(Overlay::Transcript(t)) = &mut self.overlay {
                t.consolidate_cells(start..end, consolidated.clone());
                tui.frame_requester().schedule_frame();
            }

            self.finish_agent_message_consolidation(tui, scrollback_reflow)?;
        } else {
            tracing::debug!(
                "ConsolidateAgentMessage: no cells to consolidate(start={start}, end={end})",
            );
            self.finish_agent_message_consolidation(tui, scrollback_reflow)?;
        }

        Ok(())
    }

    fn finish_agent_message_consolidation(
        &mut self,
        tui: &mut tui::Tui,
        scrollback_reflow: ConsolidationScrollbackReflow,
    ) -> Result<()> {
        match scrollback_reflow {
            ConsolidationScrollbackReflow::IfResizeReflowRan => {
                self.maybe_finish_stream_reflow(tui)?;
            }
            ConsolidationScrollbackReflow::RequiredForCanonicalMismatch => {
                self.finish_required_stream_reflow(tui)?;
            }
        }
        Ok(())
    }
}
