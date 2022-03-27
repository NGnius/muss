# mps

![repl_demo](https://raw.githubusercontent.com/NGnius/mps/master/extras/demo.png)

Sort, filter and analyse your music to create great playlists.
This project implements the interpreter (mps-interpreter), music player (mps-player), and CLI interface for MPS (root).
The CLI interface includes a REPL for running scripts.
The REPL interactive mode also provides more details about using MPS through the `?help` command.

## Usage
To access the REPL, simply run `cargo run`. You will need the [Rust toolchain installed](https://rustup.rs/). For a bit of extra performance, run `cargo run --release` instead.

## Examples

### One-liners

All songs by artist `<artist>` (in your library), sorted by similarity to a random first song by the artist.
```mps
files().(artist? like "<artist>")~(shuffle)~(advanced bliss_next);
```

All songs with a `.flac` file extension (anywhere in their path -- not necessarily at the end).
```mps
files().(filename? like ".flac");
```

All songs by artist `<artist1>` or `<artist2>`, sorted by similarity to a random first song by either artist.
```mps
files().(artist? like "<artist1>" || artist? like "<artist2>")~(shuffle)~(advanced bliss_next);
```

### Bigger examples

For now, check out `./src/tests`, `./mps-player/tests`, and `./mps-interpreter/tests` for examples.
One day I'll add pretty REPL example pictures and some script files...
// TODO

## FAQ

### Can I use MPS right now?
**Sure!** It's not complete, but MPS is completely useable for basic music queries right now. Hopefully most of the bugs have been ironed out as well :)

### Why write a new language?
**I thought it would be fun**. I also wanted to be able to play my music without having to be at the whim of someone else's algorithm (and music), and playing just by album or artist was getting boring. Designing a language specifically for iteration seemed like a cool & novel way of doing it, too (though every approach is a novel approach for me).

### What is MPS?
**Music Playlist Script (MPS) is technically a query language for music files.** It uses an (auto-generated) SQLite3 database for SQL queries and can also directly query the filesystem. Queries can be modified by using filters, functions, and sorters built-in to MPS (see mps-interpreter's README.md).

### Is MPS a scripting language?
**Yes**. It evolved from a simple query language into something that can do arbitrary calculations. Whether it's Turing-complete is still unproven, but it's powerful enough for what I want it to do.


## License

LGPL-2.1-only OR GPL-3.0-only

**NOTE**: When advanced features are enabled, GPL-3.0 must be used.

## Contribution

This is a hobby project, so any contribution may take a while to be acknowledged and accepted.
