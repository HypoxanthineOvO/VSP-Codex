use crate::host_contract::HostContractStatus;
use crate::host_contract::read_host_status;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkflowWorkspaceKind {
    Semantic,
    Legacy,
}

pub fn find_workflow_root(cwd: impl AsRef<Path>) -> Option<PathBuf> {
    let cwd = cwd.as_ref();
    cwd.ancestors()
        .find(|candidate| is_safe_directory(&candidate.join(".pipeline")))
        .map(Path::to_path_buf)
}

pub fn is_workflow_project(cwd: impl AsRef<Path>) -> bool {
    find_workflow_root(cwd).is_some()
}

pub fn workflow_runtime_guidance(cwd: impl AsRef<Path>) -> Option<String> {
    let root = find_workflow_root(cwd)?;
    if workflow_workspace_kind(&root) == WorkflowWorkspaceKind::Semantic {
        return Some(
            "<hypo_workflow_host>\nThe installed Hypo-Workflow plugin owns workflow semantics and lifecycle writes; VSP-Codex provides host UX and a trusted state projection. In this semantic workspace, use `.pipeline/INDEX.md` and the focused Cycle records. If no trusted focus is available, remain unbound and do not block ordinary work.\n</hypo_workflow_host>"
                .to_string(),
        );
    }
    let status = read_host_status(&root).status;
    let diagnostic = match status {
        HostContractStatus::Current => "Host Contract v1 projection is current.",
        HostContractStatus::Invalidated => {
            "Host Contract v1 projection is invalidated; fail closed and use the installed Hypo-Workflow plugin to refresh it."
        }
        HostContractStatus::Legacy => {
            "Legacy Workflow state was detected. Do not write it; use /hw:init to adopt the workspace."
        }
        HostContractStatus::Missing => {
            "No Host Contract projection is available. Use the installed Hypo-Workflow plugin as the only Workflow authority."
        }
        HostContractStatus::Invalid(_) => {
            "Host Contract v1 projection is invalid or unsupported; fail closed and do not fall back to legacy writers."
        }
    };
    Some(format!(
        "<hypo_workflow_host>\nHypo-Workflow semantics, lifecycle writes, Records, Receipts, and recovery are owned by the installed plugin. VSP-Codex provides host UX only. {diagnostic}\n</hypo_workflow_host>"
    ))
}

fn workflow_workspace_kind(root: &Path) -> WorkflowWorkspaceKind {
    let pipeline = root.join(".pipeline");
    if is_safe_regular_file(&pipeline.join("INDEX.md"))
        && is_safe_directory(&pipeline.join("cycles"))
    {
        WorkflowWorkspaceKind::Semantic
    } else {
        WorkflowWorkspaceKind::Legacy
    }
}

fn is_safe_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

fn is_safe_directory(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
}
