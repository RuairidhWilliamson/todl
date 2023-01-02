use std::{
    path::PathBuf,
    str::FromStr,
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Local};
use crossterm::style::Color;
use git2::Repository;

use crate::try_strip_leading_dot;

// Incomplete list based on https://en.wikipedia.org/wiki/Comment_(computer_programming)#Tags
/// The kind of tag found. (Tags are not case sensitive)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagKind {
    /// `TODO`
    Todo,
    /// Rust `todo!()` macro
    TodoMacro,
    /// `BUG` or `DEBUG`
    Bug,
    /// `FIXME` or `FIX`
    Fix,
    /// `NOTE` or `NB`
    Note,
    /// `UNDONE`
    Undone,
    /// `HACK` or `BODGE` or `KLUDGE`
    Hack,
    /// `XXX`
    Xxx,
    /// `OPTIMIZE` or `OPTIMIZEME` or `OPTIMISE` or `OPTIMISEME`
    Optimize,
    /// `SAFETY`
    Safety,
    /// `INVARIANT`
    Invariant,
    /// `LINT`
    Lint,
    /// `IGNORED`
    Ignored,
    /// Anything that doesn't match one of the TagKind variants but still looks like a comment tag
    /// Specifically excluded from this are `http` and `https`
    Custom(String),
}

impl TagKind {
    /// Parses a tag from a string
    pub fn new(tag: &str) -> Self {
        let Ok(tag) = Self::from_str(tag) else {
            return Self::Custom(tag.to_owned());
        };
        tag
    }

    /// Gets the tag level for a tag
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

    /// Gets the terminal color for a tag kind
    pub fn color(&self) -> Color {
        match self {
            TagKind::TodoMacro => Color::Magenta,
            _ => self.level().color(),
        }
    }
}

/// Represents an error when trying to parse a tag that doesn't match one of the known enum
/// variants. This will normally be handled by using `TagKind::Custom`.
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

/// The level of severity or urgency behind a tag. Useful for filtering tags quickly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagLevel {
    /// Something is broken and needs fixing
    ///
    /// Includes:
    /// - [`TagKind::Bug`]
    /// - [`TagKind::Fix`]
    Fix,
    /// Something needs to be improved
    ///
    /// Includes:
    /// - [`TagKind::Todo`]
    /// - [`TagKind::TodoMacro`]
    /// - [`TagKind::Optimize`]
    Improvement,
    /// Extra information about the code
    ///
    /// Includes:
    /// - [`TagKind::Note`]
    /// - [`TagKind::Undone`]
    /// - [`TagKind::Hack`]
    /// - [`TagKind::Xxx`]
    /// - [`TagKind::Safety`]
    /// - [`TagKind::Invariant`]
    /// - [`TagKind::Lint`]
    /// - [`TagKind::Ignored`]
    Information,
    /// Custom tag did not match known tags
    ///
    /// Includes:
    /// - [`TagKind::Custom`]
    Custom,
}

impl TagLevel {
    /// Returns the terminal color for the tag level
    pub fn color(&self) -> Color {
        match self {
            TagLevel::Fix => Color::Red,
            TagLevel::Improvement => Color::Blue,
            TagLevel::Information => Color::Grey,
            TagLevel::Custom => Color::Yellow,
        }
    }
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

/// Parsing tag level from a string failed, the tag level provided did not match one of the tag
/// levels.
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

/// Tag represents a comment tag found in a source file.
#[derive(Debug)]
pub struct Tag {
    /// The relative path of the source file
    pub path: PathBuf,
    /// The line number of the tag in the source file
    pub line: usize,
    /// The kind of tag
    pub kind: TagKind,
    /// The message provided by the tag. The message will only contain information on the same line
    /// as the tag comment.
    pub message: String,
    /// An optional git info when the tag was last changed. Only present if [`super::SearchOptions::git_blame`] is
    /// enabled in [`super::SearchOptions`], a git repository is found and the source file is not ignored in git.
    pub git_info: Option<GitInfo>,
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(git_info) = &self.git_info {
            write!(
                f,
                "{}: {} {} {}:{}",
                self.kind,
                self.message,
                git_info,
                self.path.display(),
                self.line
            )
        } else {
            write!(
                f,
                "{}: {} {}:{}",
                self.kind,
                self.message,
                self.path.display(),
                self.line,
            )
        }
    }
}

impl Tag {
    /// Get the blame for a tag. Gets the time and author for the final commit
    pub fn get_blame_info(&self, repo: &Repository) -> Option<GitInfo> {
        let blame = repo
            .blame_file(try_strip_leading_dot(&self.path), Default::default())
            .ok()?;
        let blame_hunk = blame.get_line(self.line)?;
        let commit = repo.find_commit(blame_hunk.final_commit_id()).ok()?;
        let seconds = commit.time().seconds();
        let duration = Duration::new(seconds as u64, 0);
        let git_info = GitInfo {
            time: SystemTime::UNIX_EPOCH + duration,
            author: commit.author().name()?.to_owned(),
        };
        Some(git_info)
    }
}

/// Git information about a tag
#[derive(Debug)]
pub struct GitInfo {
    /// The last time the tag line was modified
    pub time: SystemTime,
    /// The author of the last modification
    pub author: String,
}

impl std::fmt::Display for GitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time: DateTime<Local> = self.time.into();
        write!(f, "{} {}", time.format("%F %T"), self.author)
    }
}
