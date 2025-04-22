//! Todl allows for searching and working with comment tags.
//! Comment tags are comments left in code or macros that use a pattern to categorize their
//! purpose. See <https://en.wikipedia.org/wiki/Comment_(computer_programming)#Tags>
//! for more information.
//!
//! In this example there is a `TODO` comment tag and a rust `todo!` macro.
//! ```
//! // TODO: Add cool features
//! fn foo() {
//!     todo!("This is where the cool features should be")
//! }
//! ```
//!
//! # Basic usage
//! To use todl as a library
//! ```
//! use todl::{search_files, SearchOptions};
//!
//! for tag in search_files(".", SearchOptions::default()) {
//!     println!("{}", tag);
//! }
//! ```

#![warn(missing_docs, clippy::print_stdout, clippy::print_stderr)]

use std::{fs::File, path::Path};

use git2::Repository;
use walkdir::WalkDir;

/// Identify and search source files
pub mod source;
/// Progromatic representations of comment tags and similar macros
pub mod tag;

pub use source::{SourceFile, SourceKind};
pub use tag::{Tag, TagKind, TagLevel};

/// Options passed to [`search_files`]
///
/// [`SearchOptions`] allow fine grain control over how search is performed. By default all options are
/// enabled. Disabling the git integration will speed up the search speed significantly. The
/// function [`SearchOptions::no_git`] provides an easy way of specifying this.
#[derive(Debug, Clone, Copy)]
pub struct SearchOptions {
    /// When enabled will use the git ignore file to exclude files from the search
    pub git_ignore: bool,
    /// When enabled will try and use git to get the last modification to the line and return that
    /// time
    pub git_blame: bool,
}

impl SearchOptions {
    /// Disables all git features in search options which improves performance
    pub fn no_git() -> Self {
        Self {
            git_ignore: false,
            git_blame: false,
        }
    }
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            git_ignore: true,
            git_blame: true,
        }
    }
}

/// Recursively search for tags in files.
///
/// Returns an iterator of [`Tag`] which recursively searches all files of the given path (Does not
/// follow symlinks). The
/// [`SearchOptions`] change how the search is performed. Allowing git integration to be used
/// optionally. Git integration is enabled by default but slows down the search process for large
/// repositories.
///
/// # Example
/// ```
/// use todl::{search_files, SearchOptions, Tag};
///
/// // This is equivalent to default() but is defined explictly for clarity here
/// let options = SearchOptions {
///     git_ignore: true,
///     git_blame: true,
/// };
/// let tags: Vec<Tag> = search_files(".", options).collect();
/// println!("Found {} tags", tags.len());
/// println!("The first tag is {}", tags.get(0).unwrap());
/// ```
pub fn search_files<P: AsRef<Path>>(
    path: P,
    search_options: SearchOptions,
) -> impl Iterator<Item = Tag> {
    let repository = open_inside_repository(&path);
    let repository2 = open_inside_repository(&path);
    let SearchOptions {
        git_ignore,
        git_blame,
    } = search_options;

    WalkDir::new(&path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(move |e| {
            if git_ignore {
                if let Some(repo) = &repository {
                    if let Ok(ignored) = repo.status_should_ignore(try_strip_leading_dot(e.path()))
                    {
                        if ignored {
                            return None;
                        }
                    }
                }
            }
            let kind = SourceKind::identify(e.path())?;
            let Ok(file) = File::open(e.path()) else {
                return None;
            };
            Some(SourceFile::new(kind, e.path(), file))
        })
        .flatten()
        .map(move |mut tag| {
            if git_blame {
                if let Some(repo) = &repository2 {
                    tag.git_info = tag.get_blame_info(path.as_ref(), repo);
                }
            }
            tag
        })
}

/// Opens a repository if the path is inside one by checking parents
fn open_inside_repository<P: AsRef<Path>>(path: P) -> Option<Repository> {
    let path = path.as_ref().canonicalize().ok()?;
    let mut p = path.as_path();
    loop {
        if let Ok(repo) = Repository::open(p) {
            return Some(repo);
        }
        p = p.parent()?;
    }
}

/// Try to strip the leading `./` or does nothing
fn try_strip_leading_dot(path: &Path) -> &Path {
    path.strip_prefix("./").unwrap_or(path)
}
