mod commands;
mod config;
mod git;
mod github_api;
mod prompt_utils;

use config::Config;
use zed_extension_api as zed;

struct SlashCommandsExampleExtension;

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
            "pr-link" => Ok(vec![]),
            "pr-open" => {
                // TODO: Figure how to get this dynamically, missing workspace
                let owner = "zed-industries";
                let repo = "zed";

                // Fetch open pull requests
                match github_api::get_github_open_pull_requests(
                    owner,
                    repo,
                    &Config::default(),
                    None,
                ) {
                    Ok(prs) => {
                        let completions = prs
                            .iter()
                            .map(|pr| zed_extension_api::SlashCommandArgumentCompletion {
                                label: format!("#{}: {}", pr.number, pr.title),
                                new_text: format!("{},{},{}", owner, repo, pr.number),
                                run_command: true,
                            })
                            .collect();
                        Ok(completions)
                    }
                    Err(e) => Err(format!("Failed to fetch pull requests: {}", e)),
                }
            }
            "pr-current" => Ok(vec![]),
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        let config = Config::from_worktree(worktree);
        match command.name.as_str() {
            "pr-open" => {
                // Get owner and repo from args if provided
                let parts: Vec<&str> = if !args.is_empty() && args[0].contains(',') {
                    args[0].split(',').collect()
                } else {
                    vec![]
                };

                let owner = parts
                    .get(0)
                    .map(|s| *s)
                    .ok_or("Owner not provided in args")?;
                let repo = parts
                    .get(1)
                    .map(|s| *s)
                    .ok_or("Repository not provided in args")?;

                let pr_number = parts
                    .get(2)
                    .ok_or("No PR number provided. Please provide a PR number.")?
                    .parse::<u32>()
                    .map_err(|_| "Invalid PR number")?;

                // Use the pr_data function from the commands module to get PR details and comments
                let pr_prompt_parts = commands::pr_data(owner, repo, pr_number, &config)?;
                let (text, sections) = prompt_utils::build_slash_command_output(pr_prompt_parts);

                Ok(zed::SlashCommandOutput { sections, text })
            }
            "pr-link" => {
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

                let pr_prompt_parts = commands::pr_data(owner, repo, pr_number, &config)?;

                // Create sections from parts
                let (text, sections) = prompt_utils::build_slash_command_output(pr_prompt_parts);

                Ok(zed::SlashCommandOutput { text, sections })
            }
            "pr-current" => {
                let cwd = worktree
                    .map(|worktree| worktree.root_path())
                    .unwrap_or_else(|| "No worktree".to_string());

                // Extract owner and repo from git remote URL
                let (owner, repo) = match crate::git::get_repo(&cwd) {
                    Ok(parts) if parts.len() >= 2 => (parts[0].clone(), parts[1].clone()),
                    Ok(_) => {
                        return Err(
                            "Could not extract owner and repo from git remote URL".to_string()
                        )
                    }
                    Err(e) => return Err(format!("Failed to get repository info: {}", e)),
                };
                // Get the current branch name
                let branch = match git::get_current_branch(&cwd) {
                    Ok(branch) => Some(branch),
                    Err(_) => None,
                };

                // Get open PRs for this repo and branch
                let prs = github_api::get_github_open_pull_requests(
                    &owner,
                    &repo,
                    &config,
                    branch.as_deref(),
                )
                .map_err(|e| format!("Failed to get pull requests: {}", e))?;

                // Check if there are any PRs
                if prs.is_empty() {
                    return Err(format!(
                        "No open pull requests found for branch: {:?}",
                        branch
                    ));
                }

                // Use the first PR (most recent)
                let pr_number = prs[0].number;

                let pr_prompt_parts = commands::pr_data(&owner, &repo, pr_number, &config)?;

                // Create sections from parts
                let (text, sections) = prompt_utils::build_slash_command_output(pr_prompt_parts);

                Ok(zed::SlashCommandOutput { text, sections })
            }
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }
}

zed::register_extension!(SlashCommandsExampleExtension);
