use std::{
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use regex::Regex;

use crate::tag::{Tag, TagKind};

#[derive(Debug)]
pub enum SourceKind {
    Rust,
    CLike,
}

impl SourceKind {
    pub fn identify(path: &Path) -> Option<Self> {
        let ext = path.extension()?;
        match ext.to_str()? {
            "rs" => Some(Self::Rust),
            "c" | "cpp" | "cc" | "h" | "hpp" | "java" | "cs" => Some(Self::CLike),
            _ => None,
        }
    }
}

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
