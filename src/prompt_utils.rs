use crate::zed;

pub struct PromptPart {
    pub length: usize,
    pub label: String,
    pub content: String,
}

pub fn build_slash_command_output(
    parts: Vec<PromptPart>,
) -> (String, Vec<zed::SlashCommandOutputSection>) {
    let mut sections = Vec::new();
    let mut text = String::new();
    let mut current_position = 0;

    for (i, part) in parts.iter().enumerate() {
        sections.push(zed::SlashCommandOutputSection {
            range: (current_position..(current_position + part.length)).into(),
            label: part.label.clone(),
        });

        text.push_str(&part.content);
        current_position += part.length;

        // Add newlines after each section except the last one
        if i < parts.len() - 1 {
            text.push_str("\n\n");
            current_position += 2; // Increment position but don't include in range
        }
    }

    (text, sections)
}
