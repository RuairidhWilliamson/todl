# TODL

![](https://img.shields.io/crates/v/todl)
![](https://img.shields.io/crates/l/todl)
![](https://img.shields.io/docsrs/todl)

A tool that finds comment tags in source code.

Do you like leaving todo comments in your code but want a way to easily find them? Use grep! But if you want different kinds of tags (optimize, fix, bug, etc...), want more information (when comments were last changed) or you are just lazy then todl can help.

### What are comment tags?

Comment tags are labels at the start of comments to make it easier to find and convey meaning.
```
// TODO: Make this function do something
fn foo() -> u32 {
    0
}
```

[Wikipedia Comment Tags](https://en.wikipedia.org/wiki/Comment_(computer_programming)#Tags)

### What comment tags are supported?

C style comments and rust `todo!()` macros.

Supported tags include (case insensitive):
 - TODO
 - BUG
 - DEBUG
 - FIX
 - FIXME
 - NOTE
 - NB
 - UNDONE
 - HACK
 - BODGE
 - KLUDGE
 - XXX
 - OPTIMIZE
 - OPTIMIZEME
 - OPTIMISE
 - OPTIMISEME
 - SAFETY
 - INVARIANT
 - LINT
 - IGNORED

### What if my comments aren't supported?

There is support for custom tags but if you are using a language that is not currently supported raise an issue for it here [issues](https://github.com/RuairidhWilliamson/todl/issues).

To find custom tags use `todl -l custom`

## Install
You can install todl as a cli tool
```
cargo install todl
```

## Usage

To search the current directory
```
todl
```

Example output
```
TODO: Hello                              2022-12-24 21:10:06 ./benches/search.rs:13
TODO!:                                   2022-12-26 14:39:40 ./src/tag.rs:9
TODO: Add cool features                  2022-12-26 14:39:40 ./src/lib.rs:8
TODO!: This is where the cool features s 2022-12-26 14:39:40 ./src/lib.rs:10
TODO: Clearing the line here means we ig 2022-12-26 13:17:33 ./src/source.rs:58
TODO: Find the todo                      2022-12-24 21:10:06 ./tests/rust.rs:14
OPTIMIZE: Make it faster                 2022-12-24 21:10:06 ./tests/rust.rs:15
FIX: Fix the bugs                        2022-12-24 21:10:06 ./tests/rust.rs:17
BUG: It is broken                        2022-12-24 21:10:06 ./tests/rust.rs:20
TODO!:                                   2022-12-24 21:10:06 ./tests/rust.rs:60
TODO: Find the todo                      2022-12-25 21:57:42 ./tests/c.rs:11
OPTIMIZE: Make it faster                 2022-12-25 21:57:42 ./tests/c.rs:12
FIX: Fix the bugs                        2022-12-25 21:57:42 ./tests/c.rs:14
BUG: It is broken                        2022-12-25 21:57:42 ./tests/c.rs:17
```

## Alternatives

Check out these tools that do similar things
- [cargo-todo](https://crates.io/crates/cargo-todo)
- [todo-ci](https://crates.io/crates/todo-ci)

