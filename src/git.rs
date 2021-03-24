use crate::commit::models::Commit;
use chrono::{DateTime, Local};
use git2::{BranchType, Error, Repository, Time};

use std::fs::remove_dir_all;

/// Return the temporary folder of the git repository
fn get_tmp_dir(repo_name: &String) -> String {
    format!("/tmp/{}", &repo_name)
}

/// Clone a repository `repo` in a temporary folder. It uses only GitHub at
/// this moment.
pub fn clone_repo(repo_name: &String) -> Result<Repository, Error> {
    let url: &str =
        &format!("https://github.com/{}", &repo_name).to_owned()[..];
    let tmp_dir: String = get_tmp_dir(&repo_name);

    let cloned = Repository::clone(url, tmp_dir)?;

    Ok(cloned)
}

/// Check if a `branch` exists inside the repository and then set the head of
/// `repo` to that branch
pub fn get_branch(repo: &Repository, branch: &str) -> Result<(), Error> {
    if let Err(e) = repo.find_branch(&branch, BranchType::Local) {
        return Err(e);
    }

    repo.set_head(&format!("refs/heads/{}", branch).to_owned()[..])
}

/// Get a `git2::Commit` and returns a date in RFC2822 format
pub fn get_date(commit: &git2::Commit) -> String {
    let t: Time = commit.time();
    let (offset, sign) = match t.offset_minutes() {
        n if n < 0 => (-n, '-'),
        n => (n, '+'),
    };
    let (hours, minutes) = (offset / 60, offset % 60);
    let ts =
        time::Timespec::new(t.seconds() + (t.offset_minutes() as i64) * 60, 0);
    let time = time::at(ts);

    let result = format!(
        "{} {}{:02}{:02}",
        time.strftime("%a, %e %b %Y %T").unwrap(),
        sign,
        hours,
        minutes
    );

    result
}

/// Get a `git2::Commit` and returns a valid `Commit` to upload to the database
fn get_commit(gcommit: &git2::Commit, repo_name: &String) -> Commit {
    let hash = gcommit.id().to_string();
    let tree = match gcommit.parent_id(0) {
        Ok(parent) => Some(parent.to_string()),
        Err(_) => None,
    };

    let mut text = "".to_string();
    for line in String::from_utf8_lossy(gcommit.message_bytes()).lines() {
        text += line;
        text += "\n";
    }

    // Remove the last "\n"
    let _ = text.pop();

    let date: DateTime<Local> =
        match DateTime::parse_from_rfc2822(&get_date(&gcommit)) {
            Ok(date) => date.into(),
            Err(e) => {
                // This is a real problem!
                // TODO: manage this error
                panic!(e)
            }
        };
    let author_email = gcommit.author().email().unwrap().to_string();
    let author_name = gcommit.author().name().unwrap().to_string();
    let committer_email = gcommit.committer().email().unwrap().to_string();
    let committer_name = gcommit.committer().name().unwrap().to_string();

    Commit {
        hash,
        tree,
        text,
        date,
        author_email,
        author_name,
        committer_email,
        committer_name,
        repository_url: repo_name.clone(),
    }
}

/// Get all commits from a Git repository. Returns an array of `Commit`.
/// First, clone the repo into a temporary folder.
/// Then, open the repository.
/// Then, get commits
/// Finally, remove the temporary folder
pub fn repo_commits(
    repo_name: &String,
    branch: &String,
) -> Result<Vec<Commit>, Error> {
    // Remove a possible already cloned repository
    let _ = remove_dir_all(get_tmp_dir(&repo_name));

    // Try to clone the repo. If it returns an error, it's useless to go ahead:
    // raises an error.
    let repo = match clone_repo(&repo_name) {
        Ok(r) => r,
        Err(e) => {
            return Err(e);
        }
    };

    if let Err(e) = get_branch(&repo, branch) {
        return Err(e);
    }

    let mut revwalk = repo.revwalk()?;
    let head = repo.head().unwrap().target().unwrap();

    if let Err(e) = revwalk.push(head) {
        return Err(e);
    }

    let mut commits: Vec<Commit> = vec![];
    for commit in revwalk {
        let hash = repo.find_commit(commit?)?;
        commits.push(get_commit(&hash, &repo_name));
    }

    // Remove the cloned repository folder
    let _ = remove_dir_all(get_tmp_dir(&repo_name));

    Ok(commits)
}
