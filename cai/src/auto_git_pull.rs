//! Yes, this stupid module is for auto git pull.

use std::process::Command;
use std::io::{self, Write};

#[derive(Debug)]
pub enum AutoGitStatus {
    AlreadyUpToDate,
    Pulled,
    NotAGitRepo,
    GitNotInstalled,
    DetachedHead,
    LocalChanges,
    NoRemoteTracking,
    NetworkIssue,
    Warning
}

pub fn check_and_pull() -> AutoGitStatus {
    // 0. Check git installed
    if Command::new("git").arg("--version").output().is_err() {
        log_warn("Git not installed.");
        return AutoGitStatus::GitNotInstalled;
    }

    // 1. Check if in a git repo
    let repo_check = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();
    if !matches!(repo_check, Ok(ref o) if o.status.success() && o.stdout == b"true\n") {
        log_warn("Not in a git repository.");
        return AutoGitStatus::NotAGitRepo;
    }

    // 2. Check for detached HEAD
    let head_check = Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .output();
    if let Ok(ref o) = head_check {
        if !o.status.success() {
            log_warn("Detached HEAD state, skipping pull.");
            return AutoGitStatus::DetachedHead;
        }
    } else {
        log_warn("Could not determine HEAD state.");
        return AutoGitStatus::DetachedHead;
    }

    // 3. Check for local changes (staged or unstaged)
    let changes = Command::new("git")
        .args(["status", "--porcelain"])
        .output();
    if let Ok(ref o) = changes {
        if !o.stdout.is_empty() {
            log_warn("Uncommitted local changes detected, skipping pull.");
            return AutoGitStatus::LocalChanges;
        }
    } else {
        log_warn("Could not check local changes.");
        return AutoGitStatus::Warning;
    }

    // 4. Get current branch
    let branch = get_current_branch();
    if branch.is_none() {
        log_warn("Could not get current branch.");
        return AutoGitStatus::Warning;
    }
    let branch = branch.unwrap();

    // 5. Get remote tracking branch
    let remote = get_remote_for_branch(&branch);
    if remote.is_none() {
        log_warn("No remote tracking branch found, skipping pull.");
        return AutoGitStatus::NoRemoteTracking;
    }
    let remote = remote.unwrap();

    // 6. Fetch updates
    let fetch = Command::new("git").arg("fetch").output();
    if fetch.is_err() || !fetch.as_ref().unwrap().status.success() {
        log_warn("Failed to fetch remote. Possible network issue.");
        return AutoGitStatus::NetworkIssue;
    }

    // 7. Compare with remote
    let cmp = Command::new("git")
        .args(["rev-list", "--count", &format!("HEAD..{}/{}", remote, branch)])
        .output();
    if let Ok(ref o) = cmp {
        let count = String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().unwrap_or(0);
        if count > 0 {
            // Try to pull
            let pull = Command::new("git").arg("pull").output();
            if let Ok(ref p) = pull {
                if p.status.success() {
                    log_info("Pulled latest changes.");
                    return AutoGitStatus::Pulled;
                } else {
                    log_warn("Failed to pull changes.");
                    return AutoGitStatus::Warning;
                }
            } else {
                log_warn("Failed to execute git pull.");
                return AutoGitStatus::Warning;
            }
        } else {
            log_info("Already up to date.");
            return AutoGitStatus::AlreadyUpToDate;
        }
    } else {
        log_warn("Could not compare with remote.");
        return AutoGitStatus::Warning;
    }
}

fn get_current_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

// Gets the remote name for the current branch (usually "origin")
fn get_remote_for_branch(branch: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", &format!("branch.{}.remote", branch)])
        .output()
        .ok()?;
    if output.status.success() {
        let remote = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !remote.is_empty() {
            Some(remote)
        } else {
            None
        }
    } else {
        None
    }
}

fn log_warn(msg: &str) {
    eprintln!("[auto-git-pull WARNING]: {}", msg);
}

fn log_info(msg: &str) {
    println!("[auto-git-pull]: {}", msg);
}
