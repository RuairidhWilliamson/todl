#![allow(clippy::unwrap_used)]

use std::{io::Cursor, path::Path};

use criterion::{Criterion, criterion_group, criterion_main};
use git2::Repository;
use todl::{
    SearchOptions, search_files,
    source::{SourceFile, SourceKind},
};

fn search_short_string(c: &mut Criterion) {
    const SOURCE: &str = "
        // TODO: Hello
        /// NOTE: Wowee
        /* BANANA: This is a custom tag */
    ";

    c.bench_function("search_short_string", |b| {
        b.iter(|| {
            let source = Cursor::new(SOURCE);
            let count = SourceFile::new(SourceKind::Rust, Path::new("testing"), source).count();
            assert_eq!(3, count);
        });
    });
}

fn search_rust_backtrace_repo(c: &mut Criterion) {
    let url = "https://github.com/rust-lang/backtrace-rs.git";
    let rev = "refs/tags/0.3.67";
    let path = "temp/backtrace-rs";
    // Clone or open the repo
    let repo = Repository::clone(url, path).unwrap_or_else(|_err| Repository::open(path).unwrap());
    let (object, _) = repo.revparse_ext(rev).unwrap();
    repo.checkout_tree(&object, None).unwrap();
    repo.set_head(rev).unwrap();

    c.bench_function("search_rust_backtrace_repo", |b| {
        b.iter(|| {
            assert_eq!(
                17,
                search_files(Path::new(path), SearchOptions::no_git()).count()
            );
        });
    });
}

criterion_group!(benches, search_short_string, search_rust_backtrace_repo);
criterion_main!(benches);
