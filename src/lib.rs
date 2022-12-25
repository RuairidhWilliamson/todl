use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

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
        .filter_map(move |e| {
            let Some(kind) = source_identifier.identify(e.path()) else {
                return None;
            };
            Some(SourceFile::new(
                kind,
                e.path(),
                File::open(e.path()).unwrap(),
            ))
        })
        .flatten()
}
