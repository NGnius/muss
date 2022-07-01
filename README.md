# muss

![repl_demo](https://raw.githubusercontent.com/NGnius/mps/master/extras/demo.png)

Sort, filter and analyse your music to create great playlists.
This project implements the interpreter (`./interpreter`), music player (`./player`), and CLI interface for Muss (`./`).
The CLI interface includes a REPL for running scripts.
The REPL interactive mode also provides more details about using Muss through the `?help` command.

## Usage
To access the REPL, simply run `cargo run`. You will need the [Rust toolchain installed](https://rustup.rs/). For a bit of extra performance, run `cargo run --release` instead.

## Examples

### One-liners

All songs by artist `<artist>` (in your library), sorted by similarity to a random first song by the artist.
```muss
files().(artist? like "<artist>")~(shuffle)~(advanced bliss_next);
```

All songs with a `.flac` file extension (anywhere in their path -- not necessarily at the end).
```muss
files().(filename? like ".flac");
```

All songs by artist `<artist1>` or `<artist2>`, sorted by similarity to a random first song by either artist.
```muss
files().(artist? like "<artist1>" || artist? like "<artist2>")~(shuffle)~(advanced bliss_next);
```

### Bigger examples

For now, check out `./src/tests`, `./player/tests`, and `./interpreter/tests` for examples.
One day I'll add pretty REPL example pictures and some script files...
// TODO

## FAQ

### Can I use Muss right now?
**Sure!** It's never complete, but Muss is completely useable right now. Hopefully most of the bugs have been ironed out as well :)

### Why write a new language?
**I thought it would be fun**. I also wanted to be able to play my music without having to be at the whim of someone else's algorithm (and music), and playing just by album or artist was getting boring. Designing a language specifically for iteration seemed like a cool & novel way of doing it, too (though every approach is a novel approach for me).

### What is Muss?
**Music Set Script (MuSS) is a language for describing a playlist of music.** It uses an (auto-generated) SQLite3 database for SQL queries and can also directly query the filesystem. Queries can be modified by using filters, functions, and sorters built-in to Muss (see interpreter's README.md).

### Is Muss a scripting language?
**Yes**. It evolved from a simple query language into something that can do arbitrary calculations. Whether it's Turing-complete is still unproven, but it's powerful enough for what I want it to do.


## License

LGPL-2.1-only OR GPL-3.0-only

**NOTE**: When advanced features are enabled, GPL-3.0 must be used.

## Contribution

This is a hobby project, so any contribution may take a while to be acknowledged and accepted.
