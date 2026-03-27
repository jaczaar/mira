use std::path::Path;
use tokio::process::Command;

use super::error::ClaudeError;
use super::types::{ChangeDiff, DiffFile, PRResult};

async fn run_git(repo_path: &Path, args: &[&str]) -> Result<String, ClaudeError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .await
        .map_err(|e| ClaudeError::GitError(format!("Failed to run git: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !stderr.trim().is_empty() {
            return Err(ClaudeError::GitError(stderr));
        }
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn get_diff(repo_path: &Path) -> Result<ChangeDiff, ClaudeError> {
    let status_output = run_git(repo_path, &["status", "--porcelain"]).await?;

    let mut files = Vec::new();

    for line in status_output.lines() {
        if line.len() < 4 {
            continue;
        }
        let status_code = line[..2].trim().to_string();
        let file_path = line[3..].to_string();

        let status = match status_code.as_str() {
            "M" | "MM" => "modified",
            "A" | "AM" => "added",
            "D" => "deleted",
            "??" => "untracked",
            "R" => "renamed",
            _ => "modified",
        }
        .to_string();

        let diff = if status_code == "??" {
            format!("(new file: {})", file_path)
        } else {
            run_git(repo_path, &["diff", "--", &file_path])
                .await
                .unwrap_or_default()
        };

        files.push(DiffFile {
            path: file_path,
            status,
            diff,
        });
    }

    let summary = if files.is_empty() {
        "No changes detected".to_string()
    } else {
        let added = files.iter().filter(|f| f.status == "added" || f.status == "untracked").count();
        let modified = files.iter().filter(|f| f.status == "modified").count();
        let deleted = files.iter().filter(|f| f.status == "deleted").count();
        format!(
            "{} file(s) changed: {} added, {} modified, {} deleted",
            files.len(),
            added,
            modified,
            deleted
        )
    };

    Ok(ChangeDiff { files, summary })
}

pub async fn create_pr(
    repo_path: &Path,
    title: &str,
    body: &str,
    github_token: &str,
) -> Result<PRResult, ClaudeError> {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .chars()
        .take(40)
        .collect();
    let branch_name = format!("claude/{}-{}", timestamp, slug);

    // Get current branch to return to later
    let current_branch = run_git(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])
        .await?
        .trim()
        .to_string();

    // Create and checkout new branch
    run_git(repo_path, &["checkout", "-b", &branch_name]).await?;

    // Stage all changes
    run_git(repo_path, &["add", "-A"]).await?;

    // Commit
    let commit_msg = format!("{}\n\n{}", title, body);
    run_git(repo_path, &["commit", "-m", &commit_msg]).await?;

    // Push
    run_git(repo_path, &["push", "-u", "origin", &branch_name]).await?;

    // Get remote info for PR creation
    let remote_url = run_git(repo_path, &["remote", "get-url", "origin"])
        .await?
        .trim()
        .to_string();

    let (owner, repo) = parse_github_remote(&remote_url)
        .ok_or_else(|| ClaudeError::GitHubError("Could not parse GitHub remote URL".to_string()))?;

    // Create PR via GitHub API
    let pr_result = create_github_pr(
        github_token,
        &owner,
        &repo,
        title,
        body,
        &branch_name,
        &current_branch,
    )
    .await?;

    // Return to original branch
    let _ = run_git(repo_path, &["checkout", &current_branch]).await;

    Ok(pr_result)
}

fn parse_github_remote(url: &str) -> Option<(String, String)> {
    // Handle HTTPS: https://github.com/owner/repo.git
    if url.contains("github.com") {
        let parts: Vec<&str> = url.trim_end_matches(".git").split('/').collect();
        if parts.len() >= 2 {
            let repo = parts[parts.len() - 1].to_string();
            let owner = parts[parts.len() - 2].to_string();
            return Some((owner, repo));
        }
    }
    // Handle SSH: git@github.com:owner/repo.git
    if url.starts_with("git@github.com:") {
        let path = url
            .trim_start_matches("git@github.com:")
            .trim_end_matches(".git");
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    None
}

async fn create_github_pr(
    token: &str,
    owner: &str,
    repo: &str,
    title: &str,
    body: &str,
    head: &str,
    base: &str,
) -> Result<PRResult, ClaudeError> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);

    let payload = serde_json::json!({
        "title": title,
        "body": body,
        "head": head,
        "base": base,
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Mira-App/1.0")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&payload)
        .send()
        .await
        .map_err(|e| ClaudeError::GitHubError(format!("Failed to create PR: {}", e)))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| ClaudeError::GitHubError(e.to_string()))?;

    if !status.is_success() {
        return Err(ClaudeError::GitHubError(format!(
            "GitHub API error {}: {}",
            status, response_text
        )));
    }

    let pr: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| ClaudeError::GitHubError(format!("Failed to parse PR response: {}", e)))?;

    Ok(PRResult {
        url: pr["html_url"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        number: pr["number"].as_i64().unwrap_or(0),
        branch: head.to_string(),
    })
}

pub async fn discard_changes(repo_path: &Path) -> Result<(), ClaudeError> {
    run_git(repo_path, &["checkout", "."]).await?;
    run_git(repo_path, &["clean", "-fd"]).await?;
    Ok(())
}
