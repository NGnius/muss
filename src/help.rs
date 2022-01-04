/// Standard help string containing usage information for MPS.
pub fn help() -> String {
"This language is all about iteration. Almost everything is an iterator or operates on iterators. By default, any operation which is not an assignment will cause the script runner to handle (play/save) the items which that statement contains.

To view the currently-supported operations, try ?functions and ?filters".to_string()
}

pub fn functions() -> String {
"FUNCTIONS (?functions)
Similar to most other languages: function_name(param1, param2, etc.)

 sql_init(generate = true|false, folder = `path/to/music`)
    Initialize the SQLite database connection using the provided parameters. This must be performed before any other database operation (otherwise the database will be connected with default settings).

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
    Retrieve all files from a folder, matching a regex pattern.".to_string()
}

pub fn filters() -> String {
"FILTERS (?filters)
Operations to reduce the items in an iterable: iterable.(filter)

 field == something
 field >= something
 field > something
 field <= something
 field < something
    Compare all items, keeping only those that match the condition.

 [empty]
    Matches all items".to_string()
}
