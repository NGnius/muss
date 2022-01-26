# mps

![repl_demo](https://raw.githubusercontent.com/NGnius/mps/master/extras/demo.png)

A language all about iteration to play your music files.
This project implements the interpreter (mps-interpreter), music player (mps-player), and CLI interface for MPS (root).
The CLI interface includes a REPL for running scripts.
The REPL interactive mode also provides more details about using MPS through the `?help` command.

## Usage
To access the REPL, simply run `cargo run`. You will need the [Rust toolchain installed](https://rustup.rs/).

## Examples
For now, check out `./src/tests`, `./mps-player/tests`, and `./mps-interpreter/tests` for examples.
One day I'll add pretty REPL example pictures and some script files...
// TODO

## FAQ
### Is MPS Turing-Complete?
**No**. It can't perform arbitrary calculations (yet), which easily disqualifies MPS from being Turing-complete.

### Can I use MPS right now?
**Sure!** It's not complete, but MPS is completely useable for basic music queries right now. Hopefully most of the bugs have been ironed out as well...

### Why write a new language?
**I thought it would be fun**. I also wanted to be able to play my music without having to be at the whim of someone else's algorithm (and music), and playing just by album or artist was getting boring. I also thought designing a language specifically for iteration would be a novel approach to a language (though every approach is a novel approach for me).

### What is MPS?
**Music Playlist Script (MPS) is technically a query language for music files.** It uses an (auto-generated) SQLite3 database for SQL queries and can also directly query the filesystem. Queries can be modified by using filters, functions, and sorters built-in to MPS (see mps-interpreter's README.md).

### Is MPS a scripting language?
**No**. Technically, it was designed to be one, but it doesn't meet the requirements of a scripting language (yet). One day, I would like it be Turing-complete and then it could be considered a scripting language. At the moment it is barely a query language.


### Contribution

This is a hobby project, so any contribution may take a while to be acknowledged and accepted.
