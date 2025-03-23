use crate::zed;

pub struct PromptPart {
    pub length: usize,
    pub label: String,
    pub content: String,
}

pub fn build_output_sections(parts: &Vec<PromptPart>) -> Vec<zed::SlashCommandOutputSection> {
    let mut sections = Vec::new();
    let mut current_position = 0;

    for (_i, part) in parts.iter().enumerate() {
        sections.push(zed::SlashCommandOutputSection {
            range: (current_position..(current_position + part.length)).into(),
            label: part.label.clone(),
        });

        current_position += part.length;
    }

    sections
}

pub fn build_output_text(parts: &Vec<PromptPart>) -> String {
    let mut text = String::new();
    for part in parts.iter() {
        text.push_str(&part.content);
    }
    text
}
