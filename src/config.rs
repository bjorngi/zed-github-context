// Configuration for GitHub API
use crate::zed;

const ENV_GITHUB_TOKEN: &str = "GITHUB_TOKEN";

pub struct Config {
    pub github_token: Option<String>,
}

impl Config {
    pub fn from_worktree(worktree: Option<&zed::Worktree>) -> Self {
        let github_token = worktree.and_then(|wt| {
            let env_vars: std::collections::HashMap<String, String> =
                wt.shell_env().into_iter().collect();
            env_vars.get(ENV_GITHUB_TOKEN).cloned()
        });

        Config { github_token }
    }

    pub fn default() -> Self {
        Config { github_token: None }
    }
}
