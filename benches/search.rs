use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use criterion::{criterion_group, criterion_main, Criterion};
use git2::Repository;
use todl::{search_files, search_reader};

fn search_short_string(c: &mut Criterion) {
    const SOURCE: &str = "
        // TODO: Hello
        /// NOTE: Wowee
        /* BANANA: This is a custom tag */
    ";

    c.bench_function("search_short_string", |b| {
        b.iter(|| {
            let source = Cursor::new(SOURCE);
            assert_eq!(3, search_reader(PathBuf::from("testing"), source).count());
        })
    });
}

fn search_rust_backtrace_repo(c: &mut Criterion) {
    let url = "https://github.com/rust-lang/backtrace-rs.git";
    let path = "temp/backtrace-rs";
    // Clone or open the repo
    let repo = Repository::clone(url, path).unwrap_or_else(|_err| Repository::open(path).unwrap());
    repo.set_head("refs/tags/0.3.67").unwrap();

    c.bench_function("search_rust_backtrace_repo", |b| {
        b.iter(|| {
            assert_eq!(18, search_files(Path::new(path)).count());
        })
    });
}

fn search_rustc_repo(c: &mut Criterion) {
    // We will clone the actual rust repo into temp
    let url = "https://github.com/rust-lang/rust.git";
    let path = "temp/rust";
    // Clone or open the repo
    let repo = Repository::clone(url, path).unwrap_or_else(|_err| Repository::open(path).unwrap());
    repo.set_head("refs/tags/1.64.0").unwrap();


    c.bench_function("search_rust_backtrace_repo", |b| {
        b.iter(|| {
            assert_eq!(11473, search_files(Path::new(path)).count());
        })
    });
}

criterion_group!(benches, search_short_string, search_rust_backtrace_repo, search_rustc_repo);
criterion_main!(benches);
