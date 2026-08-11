use super::*;
use codex_context_fragments::ContextualUserFragment;
use codex_core_skills::SkillRenderReport;
use pretty_assertions::assert_eq;

fn available_skills() -> AvailableSkills {
    AvailableSkills {
        skill_root_lines: vec!["- `r0` = `/tmp/skills`".to_string()],
        skill_lines: vec!["- demo: Demo skill (file: r0/demo/SKILL.md)".to_string()],
        report: SkillRenderReport::default(),
        warning_message: None,
    }
}

#[test]
fn compact_protocol_is_included_when_model_omits_full_usage_instructions() {
    let rendered =
        AvailableSkillsInstructions::from_available_skills(&available_skills(), false).render();

    assert!(rendered.contains("### Skill protocol"));
    assert!(rendered.contains("across compaction and resume"));
    assert!(!rendered.contains("### How to use skills"));
}

#[test]
fn full_usage_instructions_replace_the_compact_protocol() {
    let rendered =
        AvailableSkillsInstructions::from_available_skills(&available_skills(), true).render();

    assert!(rendered.contains("### How to use skills"));
    assert!(!rendered.contains("### Skill protocol"));
    assert_eq!(rendered.matches("## Skills").count(), 1);
}
