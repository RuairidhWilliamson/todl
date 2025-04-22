use std::{
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use regex::Regex;

use crate::tag::{Tag, TagKind};

/// The kind of source file dictates what we search for.
/// `Rust` source files can have todo macros whereas `CLike` files cannot
#[derive(Debug)]
pub enum SourceKind {
    /// The same as `CLike` with rust `todo!` macros
    Rust,
    /// Supports many different C-style comments
    CLike,
}

impl SourceKind {
    /// Uses the file extension of a file path to determine what kind of source file it is.
    /// If the file extension is unknown or missing it will return `None`
    pub fn identify(path: &Path) -> Option<Self> {
        let ext = path.extension()?;
        match ext.to_str()? {
            "rs" => Some(Self::Rust),
            "c" | "cpp" | "cc" | "h" | "hpp" | "java" | "cs" => Some(Self::CLike),
            _ => None,
        }
    }
}

/// An iterator over an identified source file
pub struct SourceFile<R: Read> {
    path: PathBuf,
    kind: SourceKind,
    inner: BufReader<R>,
    line: String,
    line_number: usize,
}

impl<R: Read> SourceFile<R> {
    /// Create a new source file iterator specifying the kind, path and the reader
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
            let n = self
                .inner
                .read_line(&mut self.line)
                .expect("read line failed");
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
            let n = self
                .inner
                .read_line(&mut self.line)
                .expect("read line failed");
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
        Regex::new(r"/(?:/+|\*+)!? ?(?P<tag>[!a-zA-Z0-9_]+): ?(?P<msg>[^:].+)")
            .expect("could not compile clike comment regex");
    static ref RUST_TODO_MACRO: Regex =
        Regex::new(r#"todo!\((?:"([^"]*)")?\)"#).expect("could not compile rust todo macro regex");
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
            git_info: None,
        })
    }

    fn find_clike_comment(&self) -> Option<Tag> {
        let Some(caps) = CLIKE_COMMENT_TAG_REGEX.captures(&self.line) else {
            return None;
        };
        let raw_tag = caps.get(1)?.as_str();
        if raw_tag == "https" || raw_tag == "http" {
            return None;
        }
        let kind = TagKind::new(raw_tag);
        let mut message = caps.get(2)?.as_str().to_owned();
        if message.ends_with("*/") {
            message = message[..message.len() - 2].trim().to_owned();
        }
        Some(Tag {
            kind,
            line: self.line_number,
            path: self.path.clone(),
            message,
            git_info: None,
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
