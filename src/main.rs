use std::path::PathBuf;

use clap::Parser;
use todl::search_files;

#[derive(Debug, Parser)]
struct Args {
    /// Paths to search for source files, defaults to `.`
    paths: Vec<PathBuf>,
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
    for tag in paths.iter().flat_map(|path| search_files(path)) {
        let tag_msg = format!("{}: {}", tag.kind, tag.message);
        let tag_msg = clamp_str(&tag_msg, 40);
        println!("{:40} {}:{}", tag_msg, tag.path.display(), tag.line);
    }
}
