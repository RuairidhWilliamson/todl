use std::{
    fs::File,
    path::Path,
    time::{Duration, SystemTime},
};

use git2::Repository;
use source::{SourceFile, SourceKind};
use tag::Tag;
use walkdir::WalkDir;

pub mod source;
pub mod tag;

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchOptions {
    pub git_ignore: bool,
    pub git_blame: bool,
}

/// Recursively search for tags in files
pub fn search_files(path: &Path, search_options: SearchOptions) -> impl Iterator<Item = Tag> {
    let repository = open_inside_repository(path);
    let repository2 = open_inside_repository(path);
    let SearchOptions {
        git_ignore,
        git_blame,
    } = search_options;

    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(move |e| {
            if git_ignore {
                if let Some(repo) = &repository {
                    if repo
                        .status_should_ignore(try_strip_leading_dot(e.path()))
                        .unwrap()
                    {
                        return None;
                    }
                }
            }
            let kind = SourceKind::identify(e.path())?;
            Some(SourceFile::new(
                kind,
                e.path(),
                File::open(e.path()).unwrap(),
            ))
        })
        .flatten()
        .map(move |mut tag| {
            if git_blame {
                if let Some(repo) = &repository2 {
                    tag.time = get_blame_time(&tag, repo);
                }
            }
            tag
        })
}

/// Opens a repository if the path is inside one by checking parents
fn open_inside_repository(path: &Path) -> Option<Repository> {
    let path = path.canonicalize().unwrap();
    let mut p = path.as_path();
    loop {
        if let Ok(repo) = Repository::open(p) {
            return Some(repo);
        }
        p = p.parent()?;
    }
}

fn try_strip_leading_dot(path: &Path) -> &Path {
    path.strip_prefix("./").unwrap_or(path)
}

fn get_blame_time(tag: &Tag, repo: &Repository) -> Option<SystemTime> {
    let blame = repo
        .blame_file(try_strip_leading_dot(&tag.path), Default::default())
        .ok()?;
    let blame_hunk = blame.get_line(tag.line)?;
    let commit = repo.find_commit(blame_hunk.final_commit_id()).ok()?;
    let seconds = commit.time().seconds();
    let duration = Duration::new(seconds as u64, 0);
    Some(SystemTime::UNIX_EPOCH + duration)
}
