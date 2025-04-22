use std::{io::Cursor, path::Path};

use todl::{
    source::{SourceFile, SourceKind},
    tag::TagKind,
};

#[test]
fn find_comments_c() {
    const SOURCE: &str = "
        // TODO: Find the todo
        // Optimize: Make it faster
        /* Hack: This is hacky */
        // fIX: Fix the bugs
        /* SAFETY: Wear a hard hat */
        /* Undone: Something has been taken away */
        /* Bug: It is broken */
    ";

    let s = Cursor::new(SOURCE);
    let tags: Vec<_> = SourceFile::new(SourceKind::CLike, Path::new("testing"), s).collect();
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
    assert_eq!("This is hacky", tags[2].message);

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
fn dont_find_urls() {
    const SOURCE: &str = "
        http://example.com
        https://www.example.com
        file://relative-path
        file:///absolute-path
    ";
    let s = Cursor::new(SOURCE);
    let tags: Vec<_> = SourceFile::new(SourceKind::CLike, Path::new("testing"), s).collect();
    assert!(tags.is_empty(), "unexpected tags: {tags:?}");
}
