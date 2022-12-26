use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, SystemTime},
};

use git2::Repository;
use lazy_static::lazy_static;
use regex::Regex;
use walkdir::WalkDir;

struct SourceIdentifier {}

impl SourceIdentifier {
    fn new() -> Self {
        Self {}
    }

    fn identify(&self, path: &Path) -> Option<SourceKind> {
        let Some(ext) = path.extension() else {
            return None;
        };
        match ext.to_str().unwrap() {
            "rs" => Some(SourceKind::Rust),
            "c" | "cpp" | "cc" | "h" | "hpp" | "java" | "cs" => Some(SourceKind::CLike),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum SourceKind {
    Rust,
    CLike,
}

impl SourceKind {}

pub struct SourceFile<R: Read> {
    path: PathBuf,
    kind: SourceKind,
    inner: BufReader<R>,
    line: String,
    line_number: usize,
}

impl<R: Read> SourceFile<R> {
    pub fn new(kind: SourceKind, path: &Path, reader: R) -> Self {
        Self {
            path: path.to_owned(),
            kind,
            inner: BufReader::new(reader),
            line: String::new(),
            line_number: 0,
        }
    }

    fn next_rust(&mut self) -> Option<Tag> {
        loop {
            if let Some(tag) = self.find_rust_todo_macro() {
                // TODO: Clearing the line here means we ignore all other possible matches on this
                // line. It would be better to remove the part of the line that we have scanned, or
                // have a slice into the line to represent the part still to search
                self.line.clear();
                return Some(tag);
            }
            if let Some(tag) = self.find_clike_comment() {
                self.line.clear();
                return Some(tag);
            }
            self.line.clear();
            let n = self.inner.read_line(&mut self.line).unwrap();
            // EOF
            if n == 0 {
                return None;
            }
            self.line_number += 1;
        }
    }

    fn next_clike(&mut self) -> Option<Tag> {
        loop {
            self.line.clear();
            let n = self.inner.read_line(&mut self.line).unwrap();
            // EOF
            if n == 0 {
                return None;
            }
            self.line_number += 1;
            if let Some(tag) = self.find_clike_comment() {
                return Some(tag);
            }
        }
    }
}

lazy_static! {
    static ref CLIKE_COMMENT_TAG_REGEX: Regex =
        Regex::new(r"/(?:/+|\*+)!? ?(?P<tag>[!a-zA-Z0-9_]+): ?(?P<msg>.+)").unwrap();
    static ref RUST_TODO_MACRO: Regex = Regex::new(r#"todo!\((?:"([^"]*)")?\)"#).unwrap();
}

impl<R: Read> SourceFile<R> {
    fn find_rust_todo_macro(&self) -> Option<Tag> {
        let Some(caps) = RUST_TODO_MACRO.captures(&self.line) else {
            return None;
        };
        let message = caps
            .get(1)
            .map(|x| x.as_str().to_owned())
            .unwrap_or_default();
        Some(Tag {
            kind: TagKind::TodoMacro,
            line: self.line_number,
            path: self.path.clone(),
            message,
            time: None,
        })
    }

    fn find_clike_comment(&self) -> Option<Tag> {
        let Some(caps) = CLIKE_COMMENT_TAG_REGEX.captures(&self.line) else {
            return None;
        };
        let raw_tag = caps.get(1).unwrap().as_str();
        if raw_tag == "https" || raw_tag == "http" {
            return None;
        }
        let kind = TagKind::new(raw_tag);
        let mut message = caps.get(2).unwrap().as_str().to_owned();
        if message.ends_with("*/") {
            message = message[..message.len() - 2].trim().to_owned();
        }
        Some(Tag {
            kind,
            line: self.line_number,
            path: self.path.clone(),
            message,
            time: None,
        })
    }
}

impl<R: Read> Iterator for SourceFile<R> {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        match self.kind {
            SourceKind::Rust => self.next_rust(),
            SourceKind::CLike => self.next_clike(),
        }
    }
}

impl<R: Read> std::fmt::Debug for SourceFile<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.path.display())
    }
}

// Incomplete list based on https://en.wikipedia.org/wiki/Comment_(computer_programming)#Tags
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagKind {
    Todo,
    TodoMacro,
    Bug,
    Fix,
    Note,
    Undone,
    Hack,
    Xxx,
    Optimize,
    Safety,
    Invariant,
    Lint,
    Ignored,
    Custom(String),
}

impl TagKind {
    fn new(tag: &str) -> Self {
        let Ok(tag) = Self::from_str(tag) else {
            return Self::Custom(tag.to_owned());
        };
        tag
    }

    pub fn level(&self) -> TagLevel {
        match self {
            TagKind::Todo => TagLevel::Improvement,
            TagKind::TodoMacro => TagLevel::Improvement,
            TagKind::Bug => TagLevel::Fix,
            TagKind::Fix => TagLevel::Fix,
            TagKind::Note => TagLevel::Information,
            TagKind::Undone => TagLevel::Information,
            TagKind::Hack => TagLevel::Information,
            TagKind::Xxx => TagLevel::Information,
            TagKind::Optimize => TagLevel::Improvement,
            TagKind::Safety => TagLevel::Information,
            TagKind::Invariant => TagLevel::Information,
            TagKind::Lint => TagLevel::Information,
            TagKind::Ignored => TagLevel::Information,
            TagKind::Custom(_) => TagLevel::Custom,
        }
    }
}

#[derive(Debug)]
pub struct UnknownTagKind;

impl std::fmt::Display for UnknownTagKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown tag kind")
    }
}

impl std::error::Error for UnknownTagKind {}

impl FromStr for TagKind {
    type Err = UnknownTagKind;

    fn from_str(tag: &str) -> Result<Self, Self::Err> {
        let lowercase_tag = tag.to_lowercase();
        match lowercase_tag.as_str() {
            "todo" => Ok(Self::Todo),
            "todo!" => Ok(Self::TodoMacro),
            "bug" | "debug" => Ok(Self::Bug),
            "fixme" | "fix" => Ok(Self::Fix),
            "note" | "nb" => Ok(Self::Note),
            "undone" => Ok(Self::Undone),
            "hack" | "bodge" | "kludge" => Ok(Self::Hack),
            "xxx" => Ok(Self::Xxx),
            "optimize" | "optimise" | "optimizeme" | "optimiseme" => Ok(Self::Optimize),
            "safety" => Ok(Self::Safety),
            "invariant" => Ok(Self::Invariant),
            "lint" => Ok(Self::Lint),
            "ignored" => Ok(Self::Ignored),
            _ => Err(UnknownTagKind),
        }
    }
}

impl std::fmt::Display for TagKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Todo => "TODO",
                Self::TodoMacro => "TODO!",
                Self::Bug => "BUG",
                Self::Fix => "FIX",
                Self::Note => "NOTE",
                Self::Undone => "UNDONE",
                Self::Hack => "HACK",
                Self::Xxx => "XXX",
                Self::Optimize => "OPTIMIZE",
                Self::Safety => "SAFETY",
                Self::Invariant => "INVARIANT",
                Self::Lint => "LINT",
                Self::Ignored => "IGNORED",
                Self::Custom(custom) => custom,
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagLevel {
    Fix,
    Improvement,
    Information,
    Custom,
}

impl std::fmt::Display for TagLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Fix => "Fix",
                Self::Improvement => "Improvement",
                Self::Information => "Information",
                Self::Custom => "Custom",
            }
        )
    }
}

#[derive(Debug)]
pub struct UnknownTagLevel;

impl std::fmt::Display for UnknownTagLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown tag level")
    }
}

impl std::error::Error for UnknownTagLevel {}

impl FromStr for TagLevel {
    type Err = UnknownTagLevel;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fix" => Ok(Self::Fix),
            "improvement" => Ok(Self::Improvement),
            "information" => Ok(Self::Information),
            "custom" => Ok(Self::Custom),
            _ => Err(UnknownTagLevel),
        }
    }
}

#[derive(Debug)]
pub struct Tag {
    pub path: PathBuf,
    pub line: usize,
    pub kind: TagKind,
    pub message: String,
    pub time: Option<SystemTime>,
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} {}:{} {:?}",
            self.kind,
            self.message,
            self.path.display(),
            self.line,
            self.time,
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchOptions {
    pub git_ignore: bool,
    pub git_blame: bool,
}

/// Recursively search for tags in files
pub fn search_files(path: &Path, search_options: SearchOptions) -> impl Iterator<Item = Tag> {
    let repository = open_inside_repository(path);
    let repository2 = open_inside_repository(path);
    let source_identifier = SourceIdentifier::new();
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
            let kind = source_identifier.identify(e.path())?;
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
