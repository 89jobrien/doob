use crate::models::Note;

pub fn format_notes(notes: &[Note]) -> String {
    if notes.is_empty() {
        return "No notes found".to_string();
    }

    let mut output = String::new();
    for note in notes {
        let short_id = note
            .id
            .as_ref()
            .map(|t| t.id.to_string())
            .unwrap_or_else(|| note.uuid[..8].to_string());

        output.push_str(&format!("  {}  {}\n", short_id, note.content));

        if let Some(proj) = &note.project {
            output.push_str(&format!("         Project: {}\n", proj));
        }

        if !note.tags.is_empty() {
            output.push_str(&format!("         Tags: {}\n", note.tags.join(", ")));
        }
    }

    output
}
