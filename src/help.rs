/// Standard help string containing usage information for Muss.
pub const HELP_STRING: &str =
"This language is all about iteration. Almost everything is an iterator or operates on iterators. By default, any operation which is not an assignment will cause the script runner to handle (play/save) the items which that statement contains.

To view Muss syntax, try ?functions, ?filters, ?procedures, or ?sorters
To view REPL commands, try ?commands";

pub const FUNCTIONS: &str =
"FUNCTIONS (?functions)
Similar to most other languages: function_name(param1, param2, etc.)
These always return an iterable which can be manipulated.

 sql_init(generate = true|false, folder = `path/to/music`)
    Initialize the SQLite database connection using the provided parameters. This must be performed before any other database operation (otherwise the database will already be connected with default settings). Returns an empty iterable.

 sql(`SQL query here`)
    Perform a raw SQLite query on the database which Muss auto-generates. An iterator of the results is returned.

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

 mpd(address, term = value, term2 = value2, ...);

    Retrieve songs from a music player daemon at address. If compiled without the music_library feature, this is equivalent to the empty() function.

 reset(iterable)
    Explicitly reset an iterable. This useful for reusing an iterable variable.

 interlace(iterable1, iterable2, ...)
    Combine multiple iterables in an interleaved pattern. This is a variant of union(...) where the first item in iterable1, then iterable2, ... is returned, then the second item, etc. until all iterables are depleted. There is no limit to the amount of iterables which can be provided as parameters.

 union(iterable1, iterable2, ...)
    Combine multiple iterables in a sequential pattern. All items in iterable1 are returned, then all items in iterable2, ... until all provided iterables are depleted. There is no limit to the amount of iterables which can be provided as parameters.

 intersection(iterable1, iterable2, ...);
    Combine multiple iterables such that only items that exist in iterable1 and iterable2 and ... are returned. The order of items from iterable1 is maintained. There is no limit to the amount of iterables which can be provided as parameters.

 empty()
    Empty iterator. Useful for deleting items using replacement filters.

 empties(count);
    Iterate over count empty items. The items in this iterator have no fields (i.e. are empty).";

pub const FILTERS: &str =
"FILTERS (?filters)
Operations to reduce the items in an iterable: iterable.(filter)

 .field == something
 .field like something
 .field unlike something
 .field matches some_regex
 .field != something
 .field >= something
 .field > something
 .field <= something
 .field < something -- e.g. iterable.(.title == `Romantic Traffic`)
    Compare all items, keeping only those that match the condition. Valid field names change depending on what information is available when the Item is populated, but usually title, artist, album, genre, track, filename are valid fields. Optionally, a ? or ! can be added to the end of the field name to skip items whose field is missing/incomparable, or keep all items whose field is missing/incomparable (respectively).

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
    Replace items matching the filter with operation1 and replace items not matching the filter with operation2. The `else operation2` part may be omitted to preserve items not matching the filter. To perform operations with the current item, use the special variable `item`. The replacement filter may not contain || -- instead, use multiple filters chained together.

 unique
 unique field -- e.g. iterable.(unique title)
    Keep only items which are do not duplicate another item, or keep only items whoes specified field does not duplicate another item's same field. The first non-duplicated instance of an item is always the one that is kept.

 ??
    Keep only the items that contain at least one field (not including the filename field).";

pub const SORTERS: &str =
"SORTERS (?sorters)
Operations to sort the items in an iterable: iterable~(sorter)

 .field -- e.g. iterable~(.filename)
    Sort by an Item field. Valid field names change depending on what information is available when the Item is populated, but usually title, artist, album, genre, track, filename are valid fields. Items with a missing/incomparable fields will be sorted to the end.

 shuffle
 random shuffle -- e.g. iterable~(shuffle)
    Shuffle the songs in the iterator. This is random for up to 2^16 items, and then the randomness degrades (but at that point you won't notice).

 ~radio
 ~radio qualifier -- e.g. iterable~(~radio)
    Sort by musical similarity, starting with a random first song from the iterator. The optional qualifier may be chroma, loudness, spectrum, or tempo. When the qualifier is omitted, they are all considered for comparing audio similarity.

 advanced bliss_first -- e.g. iterable~(advanced bliss_first)
    Sort by the distance (similarity) from the first song in the iterator. Songs which are more similar (lower distance) to the first song in the iterator will be placed closer to the first song, while less similar songs will be sorted to the end. This uses the bliss music analyser, which is a very slow operation and can cause music playback interruptions for large iterators. Requires `advanced` interpreter feature.

 advanced bliss_next -- e.g. iterable~(advanced bliss_next)
    Sort by the distance (similarity) between the last played song in the iterator. Similar to bliss_first. Songs which are more similar (lower distance) to the first song in the iterator will be placed closer to the first song, while less similar songs will be sorted to the end. This uses the bliss music analyser, which is a very slow operation and can cause music playback interruptions for large iterators. Requires `advanced` interpreter feature.";

pub const PROCEDURES: &str =
"PROCEDURES (?procedures)
Operations to apply to each item in an iterable: iterable.{step1, step2, ...}
Comma-separated procedure steps will be executed sequentially (like a for loop in regular programming languages). The variable item contains the current item of the iterable.

 let variable = something -- e.g. let my_var = 42
    Declare the variable and (optionally) set the initial value to something. The assignment will only be performed when the variable has not yet been declared. When the initial value (and equals sign) is omitted, the variable is initialized as empty().

 variable = something -- e.g. my_var = 42
    Assign something to the variable. The variable must have already been declared.

 empty() -- e.g. empty()
    The empty or null constant.

 if condition { something } else { something_else } -- e.g.
    if item.title == `Romantic Traffic` {

    } else {
        remove item
    }
    Branch based on a boolean condition. Multiple comma-separated procedure steps may be supplied in the if and else branches. This does not currently support if else chains, but they can be nested to accomplish similar behaviour.

 something1 == something2
 something1 != something2
 something1 >= something2
 something1 > something2
 something1 <= something2
 something1 < something2 -- e.g. item.filename != item.title
    Compare something1 to something2. The result is a boolean which is useful for branch conditions.

 op iterable_operation -- e.g. op files().(0..=42)~(shuffle)
    An iterable operation inside of the procedure. When assigned to item, this can be used to replace item with multiple others. Note that iterable operations are never executed inside the procedure; when item is iterable, it will be executed immediately after the end of the procedure for the current item.

 (something1)
 -something1
 something1 - something2
 something1 + something2
 something1 || something2
 something1 && something2 -- e.g. 42 + (128 - 64)
    Various algebraic operations: brackets (order of operations), negation, subtraction, addition, logical OR, logical AND; respectively.

 Item(field1 = something1, field2 = something2, ...) - e.g. item = Item(title = item.title, filename = `/dev/null`)
    Constructor for a new item. Each function parameter defines a new field and it's value.

 ~`string_format` something -- e.g. ~`{filename}` item
    Format a value into a string. This behaves differently depending on the value's type: When the value is an Item, the item's corresponding field will replace all `{field}` instances in the format string. When the value is a primitive type (String, Int, Bool, etc.), the value's text equivalent will replace all `{}` instances in the format string. When the value is an iterable operation (Op), the operation's script equivalent will replace all `{}` instances in the format string.

 file(filepath) -- e.g. file(`~/Music/Romantic Traffic.flac`)
    Load an item from file, populating the item with the song's tags.";

pub const REPL_COMMANDS: &str =
"COMMANDS (?commands)
REPL-specific operations to help with writing Muss scripts: ?command

 help
 filters
 functions
 procedures
 sorters
    Show various help strings for the Muss language and syntax.

 list
    List remaining items to be played instead of playing them. If no items are about to be played, list all items in the next statement. In playlist mode, only the latter functionality applies.

 skip
    Skip remaining items to be played. If no items are about to be played, skip all items in the next statement. This is a no-output version of ?list.

 normal
    Cancel any active ?skip or ?list operation.

 verbose
    Toggle verbose messages, like extra information on items in ?list and other goodies.";
