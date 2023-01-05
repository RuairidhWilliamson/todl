use std::{io::Cursor, path::Path};

use git2::Repository;
use todl::{
    search_files,
    source::{SourceFile, SourceKind},
    tag::TagKind,
    SearchOptions,
};

#[test]
fn find_comments_rust() {
    const SOURCE: &str = "
        // TODO: Find the todo
        //! Optimize: Make it faster
        /*! Hack: This is hacky 😄 */
        /// fIX: Fix the bugs
        /* SAFETY: Wear a hard hat */
        /** Undone: Something has been taken away */
        /*** Bug: It is broken */
    ";

    let s = Cursor::new(SOURCE);
    let tags: Vec<_> = SourceFile::new(SourceKind::Rust, Path::new("testing"), s).collect();
    println!("{tags:#?}");
    assert_eq!(7, tags.len());

    assert_eq!(TagKind::Todo, tags[0].kind);
    assert_eq!(2, tags[0].line);
    assert_eq!("Find the todo", tags[0].message);

    assert_eq!(TagKind::Optimize, tags[1].kind);
    assert_eq!(3, tags[1].line);
    assert_eq!("Make it faster", tags[1].message);

    assert_eq!(TagKind::Hack, tags[2].kind);
    assert_eq!(4, tags[2].line);
    assert_eq!("This is hacky 😄", tags[2].message);

    assert_eq!(TagKind::Fix, tags[3].kind);
    assert_eq!(5, tags[3].line);
    assert_eq!("Fix the bugs", tags[3].message);

    assert_eq!(TagKind::Safety, tags[4].kind);
    assert_eq!(6, tags[4].line);
    assert_eq!("Wear a hard hat", tags[4].message);

    assert_eq!(TagKind::Undone, tags[5].kind);
    assert_eq!(7, tags[5].line);
    assert_eq!("Something has been taken away", tags[5].message);

    assert_eq!(TagKind::Bug, tags[6].kind);
    assert_eq!(8, tags[6].line);
    assert_eq!("It is broken", tags[6].message);
}

#[test]
fn find_todo_macro() {
    const SOURCE: &str = "
        todo!()
        todo!(\"I'll implement this later\")
    ";

    let s = Cursor::new(SOURCE);
    let tags: Vec<_> = SourceFile::new(SourceKind::Rust, Path::new("testing"), s).collect();
    println!("{tags:#?}");
    assert_eq!(2, tags.len());

    assert_eq!(TagKind::TodoMacro, tags[0].kind);
    assert_eq!(2, tags[0].line);
    assert_eq!("", tags[0].message);

    assert_eq!(TagKind::TodoMacro, tags[1].kind);
    assert_eq!(3, tags[1].line);
    assert_eq!("I'll implement this later", tags[1].message);
}

#[test]
#[ignore]
fn find_rustc_repo() {
    // We will clone the actual rust repo into temp
    let url = "https://github.com/rust-lang/rust.git";
    let path = Path::new("temp/rust");
    // Clone or open the repo
    let repo = Repository::clone(url, path).unwrap_or_else(|_err| Repository::open(path).unwrap());
    repo.set_head("refs/tags/1.64.0").unwrap();

    let tags: Vec<_> = search_files(path, SearchOptions::no_git()).collect();
    println!("Found {} tags", tags.len());
    for tag in &tags {
        println!("{tag}");
    }
    assert_eq!(11478, tags.len());
}

#[test]
fn find_backtrace_repo() {
    let url = "https://github.com/rust-lang/backtrace-rs.git";
    let path = Path::new("temp/backtrace-rs");
    // Clone or open the repo
    let repo = Repository::clone(url, path).unwrap_or_else(|_err| Repository::open(path).unwrap());
    repo.set_head("refs/tags/0.3.67").unwrap();

    let tags: Vec<_> = search_files(path, SearchOptions::no_git()).collect();
    println!("Found {} tags", tags.len());
    for tag in &tags {
        println!("{tag}");
    }
    assert_eq!(18, tags.len());
}

#[test]
fn find_this_repo() {
    let path = Path::new(".");
    let search_options = SearchOptions::default();
    let tags: Vec<_> = search_files(path, search_options).collect();
    println!("Found {} tags", tags.len());
    for tag in &tags {
        println!("{tag}");
    }
    // We test that we find some tags but not too many because that is probably wrong
    assert!(!tags.is_empty());
    assert!(tags.len() < 100);
}
