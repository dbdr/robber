# Robber, the Rust Optimizing Bot Bundler

[![Build status](https://travis-ci.org/dbdr/robber.svg?branch=master)](https://travis-ci.org/dbdr/robber)
[![Coverage report](https://codecov.io/gh/dbdr/robber/branch/master/graph/badge.svg)](https://codecov.io/gh/dbdr/robber)

Robber is a source code bundler for [Rust]. It parses a main file, recursively collects all other required local source files, and bundles them together in a single source file.
The main motivation is to submit Rust code for [CodinGame] competitions, and could be useful for other situations where a codebase has to be bundled into a single file.

## Main features:

  - Supports workspaces and local crates
  - Uses a Rust parser, so any valid Rust code should be accepted regardless of formatting
  - Compacts the code to save space (c.f. 100K character limit on [CodinGame]), for instance by not including unit test code
  - Supports `include_str!` for inlining data from a file into your code ([doc](https://doc.rust-lang.org/std/macro.include_str.html))
  - Optionally rewrites some patterns in your code into more efficient forms

## Optimizing?

Rust has two main compilation modes. In release mode, a lot of optimizations are performed, and the generated code is extremely efficient.
In debug mode, most of these optimizations are disabled. In addition, a lot of checks are performed, for instance that arithmetic operations do not
overflow, since that is often the sign of a bug (you can explicitly call `wrapping_add` instead of `+` if this is what makes sense in your program).
This is wonderful when developing a program and running its tests, as this will sometimes catch bugs with zero effort on your part.

[CodinGame] correctly compiles your Rust code in release mode when competing in the arena, but it uses the debug mode in the IDE.
While this is sometimes useful to catch bugs there, it can also be problematic if your bot is performance sensitive, and performs poorly or even
times out in the IDE, making it harder for you and others to test your bot.
I've measured my implementation of [Floyd–Warshall] for [Ocean of Code] to perform 40 times slower in debug mode than in release mode
(this is an extreme case, with N³ iterations of the inner loop with plenty of missed optimizations).

The idea is to implement a [source-level optimizer](src/optimizer.rs) that will rewrite your code into a version that will perform better, even in debug mode.
You should be aware that:

 - These optimizations will go around some of the usual checking done in debug mode. You should be confident in your bot correctness before using them.
 - The optimizer might be incorrect. In the best case, the generated code might simply not compile and it will be obvious. In the worse case, it will
   introduce a bug that was not in your code!
 - The generated code will be harder to read. However, normally you don't really need to read it, just work on your source. You do store it in git for history
   anyways, right? ;)

For these reasons, the optimizations are not enabled by default (liar! actually they are for now, but this will change, TODO).

### List of optimizations:

 - [x] Remove bound checks in array accesses (only implemented for lvalues for now).
   This *will* crash your bot or make it behave strangely if your indices are out of bounds.
   This is similar to what C and C++ do, and both faster and more dangerous than Rust even in release mode (unless you use explicit unchecked access).
 - [ ] Avoid the overflow checks in arithmetic operations. This might be impossible to do completely until [this issue](https://github.com/rust-lang/rust/issues/71768)
   is fixed in the Rust standard library (I'm working on it) and [CodinGame] upgrades their version of rustc.
 - [ ] Strip out debug assertions and non-debug assertions.
 - [ ] TODO: identify more cases and implement them.

These optimizations can't reasonably remove all sources of slowness from debug mode, in particular since they cannot affect the standard library, but they should help.
And there is potential to enable optimizations even useful in release mode, like the removal of array bound checks.

## TODOs

 - Provide an option for formatting, to either produce the most compact (but hard to read) code,
   or to format it as nicely as possible (probably using cargo fmt).
 - Strip out attributes irrelevant for execution, like #[allow] and #[doc] generated from /// comments.


## Installation and usage

```sh
$ git clone git@github.com:dbdr/robber.git
$ cd [your source code directory]
$ cargo run --manifest-path [PATH/TO/robber/Cargo.toml] >bundled.rs
```

You can also pipe the bundled source directly to your clipboard. For instance on Linux:
```
$ cargo run --manifest-path [PATH/TO/robber/Cargo.toml] | xclip -selection c
```

It is of course recommended to write a small script to make this more convenient.

TODO: make robber available as a cargo subcommand to simplify this.

## Testing

Robber includes [various minimalistic projects layout examples](tests/input/) and uses them to automatically test:
  - that it can bundle them,
  - that rustc successfully compiles the bundled versions, and
  - that the compiled programs work as expected.

If your find that Robber does not work on your code, feel free to open an [issue](../../issues).
If you are able to submit a pull request with an example layout that fails (or with a fix!), that will be much appreciated.

## License
Apache License, Version 2.0

MIT

[Rust]: <https://rust-lang.org>
[CodinGame]: <https://codingame.com>
[Floyd–Warshall]: <https://en.wikipedia.org/wiki/Floyd%E2%80%93Warshall_algorithm>
[Ocean of Code]: <https://www.codingame.com/multiplayer/bot-programming/ocean-of-code>
