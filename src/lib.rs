use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

struct SourceIdentifier {}

impl SourceIdentifier {
    fn new() -> Self {
        Self {}
    }
    fn identify(&self, path: &Path) -> Option<SourceKind> {
        let Some(ext) = path.extension() else {
            return None;
        };
        // OPTIMISE: Maybe use ext.to_string_lossy()
        match ext.to_str().unwrap() {
            "rs" => Some(SourceKind::Rust),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum SourceKind {
    Rust,
}

struct SourceFile {
    entry: DirEntry,
    kind: SourceKind,
}

impl SourceFile {
    fn new(source_identifier: &SourceIdentifier, entry: DirEntry) -> Option<Self> {
        let Some(kind) = source_identifier.identify(entry.path()) else {
            return None;
        };
        Some(Self { entry, kind })
    }

    fn path(&self) -> &Path {
        self.entry.path()
    }
}

impl std::fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.entry.path().display())
    }
}

// Incomplete list based on https://en.wikipedia.org/wiki/Comment_(computer_programming)#Tags
#[derive(Debug, PartialEq, Eq)]
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
        let lowercase_tag = tag.to_lowercase();
        match lowercase_tag.as_str() {
            "todo" => Self::Todo,
            "bug" | "debug" => Self::Bug,
            "fixme" | "fix" => Self::Fix,
            "note" | "nb" => Self::Note,
            "undone" => Self::Undone,
            "hack" | "bodge" | "kludge" => Self::Hack,
            "xxx" => Self::Xxx,
            "optimize" | "optimise" | "optimizeme" | "optimiseme" => Self::Optimize,
            "safety" => Self::Safety,
            "invariant" => Self::Invariant,
            "lint" => Self::Lint,
            "ignored" => Self::Ignored,
            tag => Self::Custom(tag.to_string()),
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

#[derive(Debug)]
pub struct Tag {
    pub path: PathBuf,
    pub line: usize,
    pub kind: TagKind,
    pub message: String,
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} {}:{}",
            self.kind,
            self.message,
            self.path.display(),
            self.line
        )
    }
}

pub fn search_files(path: &Path) -> impl Iterator<Item = Tag> {
    let source_identifier = SourceIdentifier::new();

    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(move |e| SourceFile::new(&source_identifier, e))
        .flat_map(|entry| {
            let f = File::open(entry.entry.path()).unwrap();
            search_reader(entry.path().to_owned(), f)
        })
}

lazy_static! {
    static ref COMMENT_TAG_REGEX: Regex =
        Regex::new(r"/(?:/+|\*+)!? ?(?P<tag>[!a-zA-Z0-9_]+): ?(?P<msg>.+)").unwrap();
    static ref RUST_TODO_MACRO: Regex = Regex::new(r#"todo!\((?:"([^"]*)")?\)"#).unwrap();
}

pub fn search_reader<R>(path: PathBuf, reader: R) -> impl Iterator<Item = Tag>
where
    R: Read,
{
    let reader = BufReader::new(reader);
    reader
        .lines()
        .filter_map(|l| l.ok())
        .enumerate()
        .flat_map(move |(i, line)| {
            let rust_todos_iter = RUST_TODO_MACRO.captures_iter(&line).map(|caps| {
                let message = caps
                    .get(1)
                    .map(|x| x.as_str().to_owned())
                    .unwrap_or_default();
                Tag {
                    kind: TagKind::TodoMacro,
                    line: i + 1,
                    path: path.to_path_buf(),
                    message,
                }
            });
            let comment_iter = COMMENT_TAG_REGEX.captures_iter(&line).filter_map(|caps| {
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
                    line: i + 1,
                    path: path.to_path_buf(),
                    message,
                })
            });
            // TODO: Remove this allocation somehow, maybe with explicit lifetimes
            rust_todos_iter.chain(comment_iter).collect::<Vec<_>>()
        })
}
