use std::path::PathBuf;

use clap::Parser;
use todl::{search_files, TagKind, TagLevel};

#[derive(Debug, Parser)]
struct Args {
    /// Paths to search for source files, defaults to `.`
    paths: Vec<PathBuf>,

    /// Only show tags of based on level
    #[arg(short, long, default_values = ["fix", "improvement"])]
    levels: Vec<TagLevel>,

    /// Only search for a specific tag
    #[arg(short, long)]
    tag: Option<TagKind>,
}

fn clamp_str(display: &str, length: usize) -> String {
    display.chars().take(length).collect()
}

fn main() {
    let args = Args::parse();

    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths
    };

    for tag in paths
        .iter()
        .flat_map(|path| search_files(path))
        .filter(|tag| args.levels.contains(&tag.kind.level()))
        .filter(|tag| {
            let Some(tag_filter) = &args.tag else {
                return true;
            };
            tag_filter == &tag.kind
        })
    {
        let tag_msg = format!("{}: {}", tag.kind, tag.message);
        let length = 40;
        let tag_msg = clamp_str(&tag_msg, length);
        println!("{:length$} {}:{}", tag_msg, tag.path.display(), tag.line);
    }
}
