/// Standard help string containing usage information for MPS.
pub const HELP_STRING: &str =
"This language is all about iteration. Almost everything is an iterator or operates on iterators. By default, any operation which is not an assignment will cause the script runner to handle (play/save) the items which that statement contains.

To view the currently-supported operations, try ?functions and ?filters";

pub const FUNCTIONS: &str =
"FUNCTIONS (?functions)
Similar to most other languages: function_name(param1, param2, etc.)

 sql_init(generate = true|false, folder = `path/to/music`)
    Initialize the SQLite database connection using the provided parameters. This must be performed before any other database operation (otherwise the database will already be connected with default settings).

 sql(`SQL query here`)
    Perform a raw SQLite query on the database which MPS auto-generates. An iterator of the results is returned.

 song(`something`)
    Retrieve all songs in the database with a title like something.

 album(`something`)
    Retrieve all songs in the database with an album title like something.

 artist(`something`)
    Retrieve all songs in the database with an artist name like something.

 genre(`something`)
    Retrieve all songs in the database with a genre title like something.

 repeat(iterable, count)
    Repeat the iterable count times, or infinite times if count is omitted.

 files(folder = `path/to/music`, recursive = true|false, regex = `pattern`)
    Retrieve all files from a folder, matching a regex pattern.";

pub const FILTERS: &str =
"FILTERS (?filters)
Operations to reduce the items in an iterable: iterable.(filter)

 field == something
 field != something
 field >= something
 field > something
 field <= something
 field < something -- e.g. iterable.(title == `Romantic Traffic`)
    Compare all items, keeping only those that match the condition. Valid field names are those of the MpsMusicItem (title, artist, album, genre, track, etc.), though this will change when proper object support is added. Optionally, a ? or ! can be added to the end of the field name to skip items whose field is missing/incomparable, or keep all items whose field is missing/incomparable (respectively).

 start..end -- e.g. iterable.(0..42)
    Keep only the items that are at the start index up to the end index. Start and/or end may be omitted to start/stop at the iterable's existing start/end (respectively). This stops once the end condition is met, leaving the rest of the iterator unconsumed.

 start..=end -- e.g. iterable.(0..=42)
    Keep only the items that are at the start index up to and including the end index. Start may be omitted to start at the iterable's existing start. This stops once the end condition is met, leaving the rest of the iterator unconsumed.

 index -- e.g. iterable.(4)
    Keep only the item at the given index. This stops once the index is reached, leaving the rest of the iterator unconsumed.

 filter1 || filter2 -- e.g. iterable.(4 || 5)
    Keep only the items that meet the criteria of filter1 or filter2. This will always consume the full iterator.

 [empty] -- e.g. iterable.()
    Matches all items";
