# Robber, the Rust Optimizing Bot Bundler

[![Build status](https://travis-ci.org/dbdr/robber.svg?branch=master)](https://travis-ci.org/dbdr/robber)
[![Coverage report](https://codecov.io/gh/dbdr/robber/branch/master/graph/badge.svg)](https://codecov.io/gh/dbdr/robber)

Robber is a source code bundler for [Rust]. It parses a main file, recursively collects all other required local source files, and bundles them together in a single source file.
The main motivation is to submit Rust code for [CodinGame] competitions, and could be useful for other situations where a codebase has to be bundled into a single file.

## Main features:

  - Supports workspaces and local crates
  - Uses a Rust parser, so any valid Rust code should be accepted regardless of formatting
  - Compacts the code to save space, for instance by not including test code (WIP)
  - Supports `include_str!` ([doc](https://doc.rust-lang.org/std/macro.include_str.html))

## TODOs

  - Perform source-level optimizations to help work around the compilation in debug mode on [CodinGame]
  - Save more space, for instance by removing indentation, as an option


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

Robber includes various minimalistic projects layout examples and uses them to automatically test:
  - that it can bundle them,
  - that rustc successfully compiles the bundled versions, and
  - that the compiled programs work as expected.

If your find that Robber does not work on your code, feel free to open an issue.
If you are able to submit a pull request with an example layout that fails (or with a fix!), that will be much appreciated.

## License
Apache License, Version 2.0
MIT

[Rust]: <https://rust-lang.org>
[CodinGame]: <https://codingame.com>
