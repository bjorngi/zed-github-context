use crate::github_api;
use crate::prompt_utils::PromptPart;

pub fn pr_data(
    owner: &str,
    repo: &str,
    pr_number: u32,
    config: &crate::config::Config,
) -> Result<Vec<PromptPart>, String> {
    // Use the github_api::get_github_pull_request function
    let pull_request = github_api::get_github_pull_request(owner, repo, pr_number, config)
        .map_err(|e| format!("Error fetching PR: {}", e))?;

    // Convert the pull request to a PromptPart
    let content = format!(
        "\nPR #{}: {}\n\n{}\n",
        pull_request.number,
        pull_request.title,
        pull_request
            .body
            .unwrap_or_else(|| "No description provided.".to_string())
    );

    let pr_prompt_part = PromptPart {
        length: content.len(),
        label: format!("PR #{}: {}\n", pull_request.number, pull_request.title),
        content,
    };

    // Fetch comments
    let comments =
        github_api::get_github_pr_comments(owner, repo, pr_number.try_into().unwrap(), config)
            .map_err(|e| format!("Error fetching PR comments: {}", e))?;

    // Convert comments to a vector of PromptPart
    let mut combined_parts = vec![pr_prompt_part];

    // Add comment parts to the combined vector
    let comment_parts: Vec<PromptPart> = comments
        .into_iter()
        .map(|comment| {
            let content = format!(
                "\nComment from user: {}\n```diff\n{}\n```\n\n{}\n",
                comment.user.login, comment.diff_hunk, comment.body
            );
            let label = if comment.in_reply_to_id != 0 {
                format!("↪ Reply to comment by @{}", comment.user.login)
            } else {
                format!("Comment by @{}", comment.user.login)
            };

            PromptPart {
                length: content.len(),
                label,
                content,
            }
        })
        .collect();

    combined_parts.extend(comment_parts);

    Ok(combined_parts)
}
