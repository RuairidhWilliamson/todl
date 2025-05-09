use std::{io::Write as _, path::PathBuf, sync::LazyLock, time::SystemTime};

use chrono::{DateTime, Local};
use clap::Parser;
use crossterm::{
    QueueableCommand as _,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use todl::{
    SearchOptions, Tag, search_files,
    tag::{TagKind, TagLevel},
};
use unicode_segmentation::UnicodeSegmentation as _;

#[derive(Debug, Parser)]
#[command(version, about)]
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

    /// Disables outputting the comment count on the last line
    #[arg(long, default_value_t = false)]
    no_count: bool,

    /// Sort the tags by the time they were changed
    #[arg(short, long, default_value_t = false)]
    sort: bool,

    /// Reverse the sorted list of tags (only applied if sort is enabled)
    #[arg(short, long, default_value_t = false)]
    reverse: bool,

    /// Output as json
    #[arg(short, long, default_value_t = false)]
    json: bool,
}

static STDOUT_ATTY: LazyLock<bool> = LazyLock::new(|| atty::is(atty::Stream::Stdout));
static TERMINAL_WIDTH: LazyLock<usize> = LazyLock::new(|| {
    crossterm::terminal::size()
        .map(|s| s.0 as usize)
        .unwrap_or(120)
});

macro_rules! color_print {
    ($color:expr, $($arg:tt)*) => {
        do_colour_print($color, format_args!($($arg)*))
    };
}

fn do_colour_print(color: Color, args: std::fmt::Arguments) {
    // Uses STDOUT_ATTY to conditionally print colours
    if *STDOUT_ATTY && inner_colour_print(color, args).is_ok() {
        return;
    }
    // Fallback to normal print
    print!("{args}");
}

fn inner_colour_print(color: Color, args: std::fmt::Arguments) -> crossterm::Result<()> {
    std::io::stdout()
        .queue(SetForegroundColor(color))?
        .queue(Print(args))?
        .queue(ResetColor)?
        .flush()?;

    Ok(())
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

    let mut tags: Box<dyn Iterator<Item = Tag>> = Box::new(
        paths
            .iter()
            .flat_map(|path| search_files(path, search_options))
            .filter(|tag| args.levels.contains(&tag.kind.level()))
            .filter(|tag| {
                let Some(tag_filter) = &args.tag else {
                    return true;
                };
                tag_filter == &tag.kind
            }),
    );
    if args.sort {
        let mut tag_vec: Vec<Tag> = tags.collect();
        tag_vec.sort_by(|a, b| {
            let ordering = b.git_info.cmp(&a.git_info);
            if args.reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });

        tags = Box::new(tag_vec.into_iter());
    }

    if args.json {
        let tags_vec: Vec<Tag> = tags.collect();
        println!(
            "{}",
            serde_json::ser::to_string_pretty(&tags_vec).expect("could not serialize to json")
        );
        return;
    }
    let tags = tags.inspect(print_tag);

    if !args.no_count {
        let count = tags.count();
        println!();
        println!("Found {count} results");
    }
}

fn print_tag(tag: &Tag) {
    let min_tag_length = 9;
    let tag_kind = tag.kind.to_string();
    color_print!(tag.kind.color(), "{:min_tag_length$} ", tag_kind);

    // Calculate the length of the message by subtracting the length of everything else we will
    // print in the line
    let tag_kind_length = tag_kind.graphemes(true).count().max(min_tag_length) + 1;
    let path_length = format_path_line(tag).graphemes(true).count() + 1;
    let git_length = tag
        .git_info
        .as_ref()
        .map(|g| {
            format!("{} {}", format_system_time(g.time), g.author)
                .graphemes(true)
                .count()
        })
        .unwrap_or(0);
    let length = *TERMINAL_WIDTH - 2 - tag_kind_length - path_length - git_length;

    // FIX: Using some charaters breaks this alignment by 1 character 😐😬
    let msg = tag
        .message
        .graphemes(true)
        .chain(std::iter::once(" ").cycle())
        .take(length)
        .collect::<String>();
    debug_assert_eq!(msg.graphemes(true).count(), length);
    color_print!(Color::White, "{}", msg);

    color_print!(Color::Yellow, "{} ", format_path_line(tag));

    if let Some(git_info) = &tag.git_info {
        color_print!(Color::Blue, "{} ", format_system_time(git_info.time));
        color_print!(Color::Green, "{}", git_info.author);
    }
    println!();
}

fn format_system_time(time: SystemTime) -> impl std::fmt::Display {
    let time: DateTime<Local> = time.into();
    time.format("%F %T")
}

fn format_path_line(tag: &Tag) -> String {
    format!("{}:{}", tag.path.display(), tag.line)
}
