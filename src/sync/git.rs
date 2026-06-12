use anyhow::Result;

pub fn init_repo(path: &str) -> Result<()> {
    git2::Repository::init(path)?;
    Ok(())
}

pub fn pull_origin(path: &str) -> Result<()> {
    let repo = git2::Repository::open(path)?;
    let _remote = repo.find_remote("origin")?;
    // TODO: fetch + merge
    Ok(())
}

pub fn push_origin(path: &str) -> Result<()> {
    let repo = git2::Repository::open(path)?;
    let _remote = repo.find_remote("origin")?;
    // TODO: push
    Ok(())
}

pub fn commit_all(path: &str, message: &str) -> Result<()> {
    let repo = git2::Repository::open(path)?;
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let head = repo.head()?.peel_to_commit()?;
    let sig = repo.signature()?;
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&head])?;
    Ok(())
}
