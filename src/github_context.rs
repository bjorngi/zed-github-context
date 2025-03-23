mod config;
mod github_api;

use config::Config;
use zed_extension_api as zed;

struct SlashCommandsExampleExtension;

struct PromptPart {
    length: usize,
    label: String,
    content: String,
}

impl zed::Extension for SlashCommandsExampleExtension {
    fn new() -> Self {
        SlashCommandsExampleExtension
    }

    fn complete_slash_command_argument(
        &self,
        command: zed::SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed_extension_api::SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "gh-pr" => Ok(vec![]),
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        match command.name.as_str() {
            "gh-pr" => {
                let config = Config::from_worktree(worktree);

                let pr_url = args
                    .first()
                    .ok_or("No URL provided. Please provide a GitHub pull request URL.")?;

                // Parse PR URL to extract owner, repo, and PR number
                let url_parts: Vec<&str> = pr_url.split('/').collect();
                let pr_number_str = url_parts.last().unwrap_or(&"");
                let pr_number = pr_number_str
                    .parse::<u32>()
                    .map_err(|_| "Invalid PR number in URL")?;

                // Extract owner and repo from URL
                let repo_parts: Vec<&str> = url_parts
                    .iter()
                    .skip_while(|part| **part != "github.com")
                    .skip(1) // Skip github.com
                    .take(2) // Take owner and repo
                    .cloned()
                    .collect();

                if repo_parts.len() != 2 {
                    return Err("Invalid GitHub PR URL format".to_string());
                }

                let owner = repo_parts[0];
                let repo = repo_parts[1];

                // Use the github_api::get_github_pull_request function
                let pull_request =
                    github_api::get_github_pull_request(owner, repo, pr_number, &config)
                        .map_err(|e| format!("Error fetching PR: {}", e))?;

                // Convert the pull request to a PromptPart
                let pr_prompt_part = PromptPart {
                    length: pull_request.body.as_ref().map_or(0, |body| body.len()),
                    label: format!("PR #{}: {}", pull_request.number, pull_request.title),
                    content: pull_request
                        .body
                        .unwrap_or_else(|| "No description provided.".to_string()),
                };

                // Fetch comments
                let comments = github_api::get_github_pr_comments(owner, repo, pr_number, &config)
                    .map_err(|e| format!("Error fetching PR comments: {}", e))?;

                // Convert comments to a vector of PromptPart
                let comment_prompt_parts: Vec<PromptPart> = comments
                    .into_iter()
                    .map(|comment| {
                        let content =
                            format!("```diff\n{}\n```\n\n{}", comment.diff_hunk, comment.body);
                        PromptPart {
                            length: content.len(),
                            label: format!("Comment by @{}", comment.user.login),
                            content,
                        }
                    })
                    .collect();

                // Combine PR description and comments into a single vector
                let mut all_prompt_parts = vec![pr_prompt_part];
                all_prompt_parts.extend(comment_prompt_parts);

                // Create a combined string of all parts
                let mut text = String::new();
                for (i, part) in all_prompt_parts.iter().enumerate() {
                    if i > 0 {
                        text.push_str("\n\n");
                    }
                    text.push_str(&part.content);
                }

                // Create sections from parts
                let mut sections = Vec::new();
                let mut current_position = 0;

                for (i, part) in all_prompt_parts.iter().enumerate() {
                    sections.push(zed::SlashCommandOutputSection {
                        range: (current_position..(current_position + part.length as u32)).into(),
                        label: part.label.clone(),
                    });

                    current_position += part.length as u32;
                    if i < all_prompt_parts.len() - 1 {
                        current_position += 2; // +2 for the "\n\n" separator
                    }
                }

                Ok(zed::SlashCommandOutput { sections, text })
            }
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }
}

zed::register_extension!(SlashCommandsExampleExtension);
