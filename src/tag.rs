use std::{str::FromStr, time::SystemTime, path::PathBuf};

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
    pub fn new(tag: &str) -> Self {
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
