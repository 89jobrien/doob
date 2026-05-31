use chrono::Utc;
use doob::models::Note;

fn make_note(content: &str) -> Note {
    Note {
        id: None,
        uuid: "test-note-uuid".to_string(),
        content: content.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        project: None,
        file_path: None,
        tags: vec![],
        metadata: None,
    }
}

#[test]
fn test_format_notes_empty_human() {
    let output = doob::output::format_notes_human(&[]);
    assert_eq!(output, "No notes found");
}

#[test]
fn test_note_human_format_with_note() {
    let note = make_note("My important note");
    let output = doob::output::format_notes_human(&[note]);
    assert!(output.contains("My important note"));
}

#[test]
fn test_format_notes_empty_json() {
    use serde_json::Value;
    let json = doob::output::format_notes_json(&[]);
    let parsed: Value = serde_json::from_str(&json).expect("Invalid JSON");
    assert_eq!(parsed["count"], 0);
    assert!(parsed["notes"].is_array());
    assert_eq!(parsed["notes"].as_array().unwrap().len(), 0);
}

#[test]
fn test_note_json_format_with_notes() {
    use serde_json::Value;
    let note = make_note("Test note content");
    let json = doob::output::format_notes_json(&[note]);
    let parsed: Value = serde_json::from_str(&json).expect("Invalid JSON");
    assert_eq!(parsed["count"], 1);
    assert_eq!(parsed["notes"][0]["content"], "Test note content");
}
