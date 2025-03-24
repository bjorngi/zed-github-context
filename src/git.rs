pub fn get_current_branch(cwd: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = zed_extension_api::Command::new("git")
        .arg("-C")
        .arg(cwd)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?;

    let branch = match String::from_utf8(output.stdout) {
        Ok(text) => text.trim().to_string(),
        Err(e) => return Err(format!("Failed to get current branch: {}", e).into()),
    };

    Ok(branch)
}

pub fn get_repo(cwd: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = zed_extension_api::Command::new("git")
        .arg("-C")
        .arg(cwd)
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url")
        .output()?;

    let url = match String::from_utf8(output.stdout) {
        Ok(text) => text.trim().to_string(),
        Err(e) => return Err(format!("Failed to get remote origin URL: {}", e).into()),
    };

    // Parse the URL to extract owner and repo
    let parts: Vec<String> = if url.contains("github.com") {
        if url.starts_with("git@github.com:") {
            // SSH format: git@github.com:owner/repo.git
            let path = url
                .trim_start_matches("git@github.com:")
                .trim_end_matches(".git");
            path.split('/').map(String::from).collect()
        } else if url.starts_with("https://github.com/") {
            // HTTPS format: https://github.com/owner/repo.git
            let path = url
                .trim_start_matches("https://github.com/")
                .trim_end_matches(".git");
            path.split('/').map(String::from).collect()
        } else {
            return Err("Unsupported GitHub URL format".into());
        }
    } else {
        return Err("Only GitHub repositories are supported".into());
    };

    if parts.len() < 2 {
        return Err("Could not extract owner and repo from URL".into());
    }

    Ok(parts.into_iter().take(2).collect())
}
