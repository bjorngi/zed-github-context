use crate::zed;

pub struct PromptPart {
    pub length: usize,
    pub label: String,
    pub content: String,
}

pub fn build_output_sections(parts: Vec<PromptPart>) -> Vec<zed::SlashCommandOutputSection> {
    let mut sections = Vec::new();
    let mut current_position = 0;

    for (i, part) in parts.iter().enumerate() {
        sections.push(zed::SlashCommandOutputSection {
            range: (current_position..(current_position + part.length)).into(),
            label: part.label.clone(),
        });

        current_position += part.length;
        if i < parts.len() - 1 {
            current_position += 2; // +2 for the "\n\n" separator
        }
    }

    sections
}
