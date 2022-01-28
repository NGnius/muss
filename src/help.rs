/// Standard help string containing usage information for MPS.
pub const HELP_STRING: &str =
"This language is all about iteration. Almost everything is an iterator or operates on iterators. By default, any operation which is not an assignment will cause the script runner to handle (play/save) the items which that statement contains.

To view the currently-supported operations, try ?functions, ?filters, or ?sorters";

pub const FUNCTIONS: &str =
"FUNCTIONS (?functions)
Similar to most other languages: function_name(param1, param2, etc.)
These always return an iterable which can be manipulated.

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
    Retrieve all files from a folder, matching a regex pattern.

 empty()
    Empty iterator. Useful for deleting items using replacement filters.";

pub const FILTERS: &str =
"FILTERS (?filters)
Operations to reduce the items in an iterable: iterable.(filter)

 field == something
 field like something
 field != something
 field >= something
 field > something
 field <= something
 field < something -- e.g. iterable.(title == `Romantic Traffic`)
    Compare all items, keeping only those that match the condition. Valid field names change depending on what information is available when the MpsItem is populated, but usually title, artist, album, genre, track, filename are valid fields. Optionally, a ? or ! can be added to the end of the field name to skip items whose field is missing/incomparable, or keep all items whose field is missing/incomparable (respectively).

 start..end -- e.g. iterable.(0..42)
    Keep only the items that are at the start index up to the end index. Start and/or end may be omitted to start/stop at the iterable's existing start/end (respectively). This stops once the end condition is met, leaving the rest of the iterator unconsumed.

 start..=end -- e.g. iterable.(0..=42)
    Keep only the items that are at the start index up to and including the end index. Start may be omitted to start at the iterable's existing start. This stops once the end condition is met, leaving the rest of the iterator unconsumed.

 index -- e.g. iterable.(4)
    Keep only the item at the given index. This stops once the index is reached, leaving the rest of the iterator unconsumed.

 filter1 || filter2 -- e.g. iterable.(4 || 5)
    Keep only the items that meet the criteria of filter1 or filter2. This will always consume the full iterator.

 [empty] -- e.g. iterable.()
    Matches all items

 if filter: operation1 else operation2 -- e.g. iterable.(if title == `Romantic Traffic`: repeat(item, 2) else item.())
    Replace items matching the filter with operation1 and replace items not matching the filter with operation2. The `else operation2` part may be omitted to preserve items not matching the filter. To perform operations with the current item, use the special variable `item`. The replacement filter may not contain || -- instead, use multiple filters chained together.";

pub const SORTERS: &str =
"SORTERS (?sorters)
Operations to sort the items in an iterable: iterable~(sorter) OR iterable.sort(sorter)

 field -- e.g. iterable~(filename)
    Sort by a MpsItem field. Valid field names change depending on what information is available when the MpsItem is populated, but usually title, artist, album, genre, track, filename are valid fields. Items with a missing/incomparable fields will be sorted to the end.

 advanced bliss_first -- e.g. iterable~(advanced bliss_first)
    Sort by the distance (similarity) from the first song in the iterator. Songs which are more similar (lower distance) to the first song in the iterator will be placed closer to the first song, while less similar songs will be sorted to the end. This uses the bliss music analyser, which is a very slow operation and can cause music playback interruptions for large iterators. Requires `advanced` mps-interpreter feature.

 advanced bliss_next -- e.g. iterable~(advanced bliss_next)
    Sort by the distance (similarity) between the last played song in the iterator. Similar to bliss_first. Songs which are more similar (lower distance) to the first song in the iterator will be placed closer to the first song, while less similar songs will be sorted to the end. This uses the bliss music analyser, which is a very slow operation and can cause music playback interruptions for large iterators. Requires `advanced` mps-interpreter feature.";
