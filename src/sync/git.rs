use std::path::Path;

use anyhow::{Context, Result};

/// Fetch remote origin/master, returning remote FreqDb content if it exists.
pub fn fetch_remote(path: &str) -> Result<Option<String>> {
    let repo = git2::Repository::open(path)
        .with_context(|| format!("not a git repo: {path}"))?;

    let remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    log::info!("fetching from origin");
    let mut remote = remote;
    remote
        .fetch(&["master"], None, None)
        .context("git fetch failed")?;

    let fetch_head = match repo.refname_to_id("FETCH_HEAD") {
        Ok(oid) => oid,
        Err(_) => return Ok(None),
    };

    let commit = repo.find_commit(fetch_head)?;
    let tree = commit.tree()?;

    match tree.get_path(Path::new("entries.yaml")) {
        Ok(entry) => {
            let blob = repo.find_blob(entry.id())?;
            let content = std::str::from_utf8(blob.content())
                .context("remote entries.yaml is not valid UTF-8")?;
            Ok(Some(content.to_string()))
        }
        Err(_) => Ok(None),
    }
}

/// Commit all changes and push to origin.
pub fn commit_and_push(path: &str, message: &str) -> Result<()> {
    let repo = git2::Repository::open(path)
        .with_context(|| format!("not a git repo: {path}"))?;

    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let sig = repo.signature()?;

    match repo.head() {
        Ok(head) => {
            let parent = head.peel_to_commit()?;
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;
        }
        Err(_) => {
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[] as &[&git2::Commit])?;
        }
    }

    if let Ok(mut remote) = repo.find_remote("origin") {
        log::info!("pushing to origin");
        remote
            .push(&["refs/heads/master"], None)
            .context("git push failed")?;
    }

    Ok(())
}
