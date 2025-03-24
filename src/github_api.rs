use serde::Serialize;
use zed_extension_api as zed;

use crate::Config;

#[derive(Debug, Serialize)]
pub struct PullRequest {
    pub number: u32,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub body: Option<String>,
    pub user: User,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub login: String,
    pub id: u32,
    pub avatar_url: String,
}

#[derive(Debug, Serialize)]
pub struct PullRequestComment {
    pub id: u32,
    pub body: String,
    pub user: User,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
    pub path: String,
    pub diff_hunk: String,
    pub in_reply_to_id: u32,
}

fn parse_github_pr_comments(
    data: &[serde_json::Value],
) -> Result<Vec<PullRequestComment>, Box<dyn std::error::Error>> {
    let mut comments = Vec::new();

    for comment_data in data {
        let user_data = comment_data.get("user").ok_or("Missing user field")?;

        let user = User {
            login: user_data
                .get("login")
                .and_then(|v| v.as_str())
                .ok_or("Missing user login")?
                .to_string(),
            id: user_data
                .get("id")
                .and_then(|v| v.as_u64())
                .ok_or("Missing user id")? as u32,
            avatar_url: user_data
                .get("avatar_url")
                .and_then(|v| v.as_str())
                .ok_or("Missing avatar_url")?
                .to_string(),
        };

        let comment = PullRequestComment {
            id: comment_data
                .get("id")
                .and_then(|v| v.as_u64())
                .ok_or("Missing comment id")? as u32,
            body: comment_data
                .get("body")
                .and_then(|v| v.as_str())
                .ok_or("Missing comment body")?
                .to_string(),
            user,
            created_at: comment_data
                .get("created_at")
                .and_then(|v| v.as_str())
                .ok_or("Missing comment created_at")?
                .to_string(),
            updated_at: comment_data
                .get("updated_at")
                .and_then(|v| v.as_str())
                .ok_or("Missing comment updated_at")?
                .to_string(),
            html_url: comment_data
                .get("html_url")
                .and_then(|v| v.as_str())
                .ok_or("Missing comment html_url")?
                .to_string(),
            path: comment_data
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing comment path")?
                .to_string(),
            diff_hunk: comment_data
                .get("diff_hunk")
                .and_then(|v| v.as_str())
                .ok_or("Missing comment diff_hunk")?
                .to_string(),
            in_reply_to_id: comment_data
                .get("in_reply_to_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        };

        comments.push(comment);
    }

    Ok(comments)
}

pub fn get_github_pr_comments(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u32,
    config: &Config,
) -> Result<Vec<PullRequestComment>, Box<dyn std::error::Error>> {
    // We need to get both issues comments and PR review comments to include outdated ones

    let review_comments_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/comments",
        repo_owner, repo_name, pr_number
    );

    let request_builder = |url: &str| {
        let mut builder = zed::http_client::HttpRequest::builder()
            .method(zed::http_client::HttpMethod::Get)
            .url(url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "zed-app");

        // Only add Authorization header if token exists
        if let Some(token) = &config.github_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }

        builder
    };

    // Fetch review comments (including outdated ones)
    let review_request = request_builder(&review_comments_url)
        .build()
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    let review_response = zed::http_client::fetch(&review_request)
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    let review_status_code = review_response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("status"))
        .and_then(|(_, v)| v.split_whitespace().next())
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);

    let review_data: Vec<serde_json::Value> = serde_json::from_slice(&review_response.body)?;

    // Check for error responses
    if review_status_code >= 400 {
        let error_message = if !review_data.is_empty() && review_data[0].get("message").is_some() {
            review_data[0]["message"]
                .as_str()
                .unwrap_or("Unknown GitHub API error")
        } else {
            "Unknown GitHub API error"
        };
        return Err(format!(
            "GitHub API error: {} ({})",
            error_message, review_status_code
        )
        .into());
    }

    parse_github_pr_comments(&review_data)
}

fn parse_github_pull_request(
    data: &serde_json::Value,
) -> Result<PullRequest, Box<dyn std::error::Error>> {
    let user_data = data.get("user").ok_or("Missing user field")?;

    let user = User {
        login: user_data
            .get("login")
            .and_then(|v| v.as_str())
            .ok_or("Missing user login")?
            .to_string(),
        id: user_data
            .get("id")
            .and_then(|v| v.as_u64())
            .ok_or("Missing user id")? as u32,
        avatar_url: user_data
            .get("avatar_url")
            .and_then(|v| v.as_str())
            .ok_or("Missing avatar_url")?
            .to_string(),
    };

    let pull_request = PullRequest {
        number: data
            .get("number")
            .and_then(|v| v.as_u64())
            .ok_or("Missing PR number")? as u32,
        title: data
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or("Missing PR title")?
            .to_string(),
        state: data
            .get("state")
            .and_then(|v| v.as_str())
            .ok_or("Missing PR state")?
            .to_string(),
        html_url: data
            .get("html_url")
            .and_then(|v| v.as_str())
            .ok_or("Missing PR html_url")?
            .to_string(),
        body: data
            .get("body")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        user,
        created_at: data
            .get("created_at")
            .and_then(|v| v.as_str())
            .ok_or("Missing PR created_at")?
            .to_string(),
        updated_at: data
            .get("updated_at")
            .and_then(|v| v.as_str())
            .ok_or("Missing PR updated_at")?
            .to_string(),
    };

    Ok(pull_request)
}

pub fn get_github_pull_request(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u32,
    config: &Config,
) -> Result<PullRequest, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        repo_owner, repo_name, pr_number
    );

    let mut request_builder = zed::http_client::HttpRequest::builder()
        .method(zed::http_client::HttpMethod::Get)
        .url(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "zed-app");

    // Only add Authorization header if token exists
    if let Some(token) = &config.github_token {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
    }

    let request = request_builder
        .build()
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    let response = zed::http_client::fetch(&request)
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    // Check status code from headers
    let status_code = response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("status"))
        .and_then(|(_, v)| v.split_whitespace().next())
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);

    let data: serde_json::Value = serde_json::from_slice(&response.body)?;

    // Check for error responses
    if status_code >= 400 {
        let error_message = data["message"]
            .as_str()
            .unwrap_or("Unknown GitHub API error");
        return Err(format!("GitHub API error: {} ({})", error_message, status_code).into());
    }

    parse_github_pull_request(&data)
}

pub fn get_github_open_pull_requests(
    repo_owner: &str,
    repo_name: &str,
    config: &Config,
    branch: Option<&str>,
) -> Result<Vec<PullRequest>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls?state=open",
        repo_owner, repo_name
    );

    let mut request_builder = zed::http_client::HttpRequest::builder()
        .method(zed::http_client::HttpMethod::Get)
        .url(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "zed-app");

    // Only add Authorization header if token exists
    if let Some(token) = &config.github_token {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
    }

    // Add base branch filter if provided
    if let Some(base_branch) = branch {
        request_builder = request_builder.header("base", base_branch);
    }

    let request = request_builder
        .build()
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    let response = zed::http_client::fetch(&request)
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    // Check status code from headers
    let status_code = response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("status"))
        .and_then(|(_, v)| v.split_whitespace().next())
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);

    let data: Vec<serde_json::Value> = serde_json::from_slice(&response.body)?;

    // Check for error responses
    if status_code >= 400 {
        let error_message = if !data.is_empty() && data[0].get("message").is_some() {
            data[0]["message"]
                .as_str()
                .unwrap_or("Unknown GitHub API error")
        } else {
            "Unknown GitHub API error"
        };
        return Err(format!("GitHub API error: {} ({})", error_message, status_code).into());
    }

    let mut pull_requests = Vec::new();
    for pr_data in data {
        // Filter by branch if specified
        if let Some(branch_name) = branch {
            if let Some(head) = pr_data.get("head") {
                if let Some(ref_name) = head.get("ref").and_then(|v| v.as_str()) {
                    if ref_name != branch_name {
                        continue; // Skip if branch doesn't match
                    }
                }
            }
        }

        match parse_github_pull_request(&pr_data) {
            Ok(pr) => pull_requests.push(pr),
            Err(e) => {
                // Log error but continue processing other PRs
                eprintln!("Failed to parse pull request: {}", e);
            }
        }
    }

    Ok(pull_requests)
}
