use std::path::PathBuf;

use clap::Parser;
use todl::{
    search_files,
    tag::{TagKind, TagLevel},
    SearchOptions,
};

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

    /// Disables git ignore to skip files, this will improve performance
    #[arg(short = 'i', long, default_value_t = false)]
    no_ignore: bool,

    /// Disables git blame to get the time comments were last modified, this will improve
    /// performance
    #[arg(short = 'b', long, default_value_t = false)]
    no_blame: bool,
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

    let search_options = SearchOptions {
        git_ignore: !args.no_ignore,
        git_blame: !args.no_blame,
    };

    for tag in paths
        .iter()
        .flat_map(|path| search_files(path, search_options))
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
        if let Some(git_info) = &tag.git_info {
            println!(
                "{:length$} {} {}:{}",
                tag_msg,
                git_info,
                tag.path.display(),
                tag.line
            );
        } else {
            println!("{:length$} {}:{}", tag_msg, tag.path.display(), tag.line);
        }
    }
}
