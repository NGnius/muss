//! Integration tests for every syntax feature

use muss_interpreter::tokens::{ParseError, Token, Tokenizer};
use muss_interpreter::*;
use std::collections::VecDeque;
use std::io::Cursor;

#[test]
fn parse_line() -> Result<(), ParseError> {
    let cursor = Cursor::new("sql(`SELECT * FROM songs;`)");
    let correct_tokens: Vec<Token> = vec![
        Token::Name("sql".into()),
        Token::OpenBracket,
        Token::Literal("SELECT * FROM songs;".into()),
        Token::CloseBracket,
    ];

    let mut tokenizer = Tokenizer::new(cursor);
    let mut buf = VecDeque::<Token>::new();
    tokenizer.read_line(&mut buf)?; // operation being tested

    // debug output
    println!("Token buffer:");
    for i in 0..buf.len() {
        println!("  Token #{}: {}", i, &buf[i]);
    }

    // validity tests
    assert_eq!(buf.len(), correct_tokens.len());
    for i in 0..buf.len() {
        assert_eq!(
            buf[i], correct_tokens[i],
            "Tokens at position {} do not match ()",
            i
        );
    }

    tokenizer.read_line(&mut buf)?; // this should immediately return
    Ok(())
}

#[inline(always)]
fn execute_single_line(
    line: &str,
    should_be_emtpy: bool,
    should_complete: bool,
) -> Result<(), InterpreterError> {
    if line.contains('\n') {
        println!(
            "--- Executing MPS code ---\n{}\n--- Executing MPS code ---",
            line
        );
    } else {
        println!("--- Executing MPS code: '{}' ---", line);
    }
    let cursor = Cursor::new(line);

    let tokenizer = Tokenizer::new(cursor);
    let interpreter = Interpreter::with_standard_vocab(tokenizer);

    let mut count = 0;
    for result in interpreter {
        if let Ok(item) = result {
            count += 1;
            if count > 100 {
                if should_complete {
                    continue; // skip println, but still check for errors
                } else {
                    println!("Got 100 items, stopping to avoid infinite loop");
                    break;
                }
            } // no need to spam the rest of the songs
            println!(
                "Got song `{}` (filename: `{}`)",
                item.field("title")
                    .expect("Expected field `title` to exist")
                    .clone()
                    .to_str()
                    .expect("Expected field `title` to be String"),
                item.field("filename")
                    .expect("Expected field `filename` to exist")
                    .clone()
                    .to_str()
                    .expect("Expected field `filename` to be String")
            );
        } else {
            println!("!!! Got error while iterating (executing) !!!");
            eprintln!("{}", result.as_ref().err().unwrap());
            result?;
        }
    }
    if should_be_emtpy {
        assert_eq!(
            count, 0,
            "{} music items found while iterating over line which should be None",
            count
        );
    } else {
        println!(
            "Got {} items, execution complete (no songs were harmed in the making of this test)",
            count
        );
        assert_ne!(
            count, 0,
            "0 music items found while iterating over line which should have Some results"
        ); // assumption: database is populated
    }
    Ok(())
}

#[test]
fn execute_sql_line() -> Result<(), InterpreterError> {
    execute_single_line("sql(`SELECT * FROM songs WHERE artist IS NOT NULL ORDER BY artist;`)", false, true)?;
    execute_single_line("sql(`SELECT * FROM songs WHERE artist IS NOT NULL AND format = 'flac' ORDER BY title DESC;`)", false, true)
}

#[test]
fn execute_simple_sql_line() -> Result<(), InterpreterError> {
    execute_single_line("song(`lov`)", false, true)
}

#[test]
fn execute_comment_line() -> Result<(), InterpreterError> {
    execute_single_line("// this is a comment", true, true)?;
    execute_single_line("# this is a special comment", true, true)
}

#[test]
fn execute_repeat_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "repeat(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`))",
        false,
        false,
    )?;
    execute_single_line(
        "repeat(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), 4)",
        false,
        true,
    )?;
    execute_single_line(
        "repeat(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), 0)",
        true,
        true,
    )
}

#[test]
fn execute_sql_init_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "sql_init(generate = false, folder = `/home/ngnius/Music`)",
        true,
        true,
    )
}

#[test]
fn execute_assign_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "let some_var = repeat(song(`Christmas in L.A.`))",
        true,
        true,
    )?;
    execute_single_line("let some_var2 = 1234", true, true)
}

#[test]
fn execute_emptyfilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).().().()",
        false,
        true,
    )
}

#[test]
fn execute_fieldfilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year >= 2000)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year <= 2020)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year == 2016)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year != 2048)",
        false,
        true,
    )
}

#[test]
fn execute_fieldfiltermaybe_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year? >= 2000)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year? <= 2020)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year! == 2016)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year! != `test`)",
        false,
        true,
    )
}

#[test]
fn execute_files_line() -> Result<(), InterpreterError> {
    execute_single_line(
        r"files(folder=`~/Music/MusicFlac/Bruno Mars/24K Magic/`, re=``, recursive=false)",
        false,
        true,
    )?;
    execute_single_line(
        r"files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)",
        false,
        true,
    )?;
    execute_single_line(r"files()", false, true)
}

#[test]
fn execute_indexfilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(2)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(0)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(!0)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(200)",
        true,
        true,
    )
}

#[test]
fn execute_rangefilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(..)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(0..=4)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(..=4)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(0..5)",
        false,
        true,
    )
}

#[test]
fn execute_orfilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(4 || 5)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year != 2020 || 5)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year != 2020 || 5 || 4 || 12)",
        false,
        true,
    )
}

#[test]
fn execute_replacefilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(if 4: files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(5))",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(if 4: files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(5) else item.())",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(if 4: item.() else files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(0 || 1).(if 200: files() else repeat(item.(), 2)))",
        false,
        true,
    )
}

#[test]
fn execute_emptysort_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).sort()",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~()",
        false,
        true,
    )
}

#[test]
fn execute_likefilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(not_a_field? like `24K Magic`)",
        true,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(not_a_field! like `24K Magic`)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(album like `24K Magic`)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(album unlike `24K Magic`)",
        true,
        true,
    )
}

#[test]
fn execute_fieldsort_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(title)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).sort(not_a_field)",
        false,
        true,
    )
}

#[test]
fn execute_blissfirstsort_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(advanced bliss_first)",
        false,
        true,
    )
}

#[test]
fn execute_blissnextsort_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(advanced bliss_next)",
        false,
        true,
    )
}

#[test]
fn execute_emptyfn_line() -> Result<(), InterpreterError> {
    execute_single_line("empty()", true, true)
}

#[test]
fn execute_resetfn_line() -> Result<(), InterpreterError> {
    execute_single_line("reset(empty())", true, true)
}

#[test]
fn execute_shufflesort_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(random shuffle)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(shuffle)",
        false,
        true,
    )?;
    execute_single_line("empty()~(shuffle)", true, true)
}

#[test]
fn execute_unionfn_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "union(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`))",
        false,
        true,
    )?;
    execute_single_line(
        "u(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), union(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)))",
        false,
        true,
    )?;
    execute_single_line(
        "interleave(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`))",
        false,
        true
    )?;
    execute_single_line(
        "interlace(empty(), files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`))",
        false,
        true,
    )
}

#[test]
fn execute_regexfilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(title matches `24K\\\\s+Magic`)", // note: quad-escape not required in scripts
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(artist? matches `Bruno Mars`)",
        false,
        true,
    )?;
    // regex options
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(artist? matches `bruno mars`, `i`)",
        false,
        true,
    )
}

#[test]
fn execute_intersectionfn_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "intersection(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`))",
        false,
        true,
    )?;
    execute_single_line(
        "n(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), n(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)))",
        false,
        true,
    )?;
    execute_single_line(
        "intersection(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`))",
        false,
        true
    )?;
    execute_single_line(
        "n(empty(), files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`))",
        true,
        true,
    )
}

#[test]
fn execute_declareitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{let x = empty()}",
        false,
        true,
    )
}

#[test]
fn execute_removeitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{remove item.title, remove item}",
        true,
        true,
    )
}

#[test]
fn execute_multiitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    let x = empty(),
    remove item,
    remove x
}",
        true,
        true,
    )
}

#[test]
fn execute_fieldassignitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.potato = empty(),
    .test = empty()
}",
        false,
        true,
    )
}

#[test]
fn execute_constitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    `str const`,
    1234,
    false,
    item.test_field = 1234,
    let foo = false
}",
        false,
        true,
    )
}

#[test]
fn execute_retrieveitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.path = item.filename,
    item.new_field = 42,
    item.title = item.path,
}",
        false,
        true,
    )
}

#[test]
fn execute_additemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
            item.title = `TEST` + item.title,
            item.test = 1234 + 94,
}",
        false,
        true,
    )
}

#[test]
fn execute_subtractitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.test = 1234 - 94,
}",
        false,
        true,
    )
}

#[test]
fn execute_negateitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.test = 1234,
    item.test = -item.test,
    item.test = -42,
}",
        false,
        true,
    )
}

#[test]
fn execute_notitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.test = false,
    item.test = !item.test,
    item.test = !true,
}",
        false,
        true,
    )
}

#[test]
fn execute_orlogicalitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.test = true || true,
    item.test = !true || false,
}",
        false,
        true,
    )
}

#[test]
fn execute_andlogicalitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.test = true && true,
    item.test = !true && false,
}",
        false,
        true,
    )
}

#[test]
fn execute_bracketsitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.test = true && true && (false || false),
    item.test = (!true && false || (false || !false)),
}",
        false,
        true,
    )
}

#[test]
fn execute_stringifyitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.filepath = ~`test out: {test}` item,
    item.test = true && true && (false || false),
    item.test = item.test || ((!true && false) || (false || !false)),
    item.title = ~`test out: {test}` item
}",
        false,
        true,
    )
}

#[test]
fn execute_branchitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    if false {
        item.title = 42,
        item.title = `THIS IS WRONG ` + item.title
    } else {
        item.title = `OK `+ item.title,
        if true {item.filename = `RIGHT`},
        if true {} else {item.filename = `WRONG`},
    }
}",
        false,
        true,
    )
}

#[test]
fn execute_compareitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    if 42 != 42 {
        item.title = `THIS IS WRONG ` + item.title
    } else {
        item.title = `OK `+ item.title
    }
}",
        false,
        true,
    )
}

#[test]
fn execute_computeitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    let count = 1,
    item.track = count,
    item.title = ~`Song #{track}` item,
    if count > 5 {
        item.filename = `¯\\\\_(ツ)_/¯`
    } else {
        item.filename = `/shrug`,
    },
    count = count + 1,
}",
        false,
        true,
    )
}

#[test]
fn execute_complexitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).().{
    let count = 1,
    item.track = count,
    item.title = ~`Song #{track}` item,
    if count > 5 {
        item.filename = `¯\\\\_(ツ)_/¯`
    } else {
        item.filename = `/shrug`,
    },
    count = count + 1,
}.()",
        false,
        true,
    )
}

#[test]
fn execute_constructitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    let other_item = Item (),
    let temp_item = Item (
        filename= `???`,
        title= `???`,
    ),
    other_item = temp_item,
    temp_item = item,
    item = other_item,
}",
        false,
        true,
    )
}

#[test]
fn execute_iteritemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item = iter empty()
}",
        true,
        true,
    )
}

#[test]
fn execute_commentitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "empty().{
    // this is a comment
    // this is another comment
    # this is also a comment
}",
        true,
        true,
    )
}

#[test]
fn execute_uniquefieldfilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "repeat(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), 3).(unique title?)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(unique album!)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(unique album)",
        false,
        true,
    )
}

#[test]
fn execute_uniquefilter_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(unique)",
        false,
        true,
    )
}

#[test]
fn execute_fileitemop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).{
    item.title = `something else`,
    item = file(item.filename),
}",
        false,
        true,
    )
}

#[test]
fn execute_emptiesop_line() -> Result<(), InterpreterError> {
    execute_single_line(
        "empties(1).{let count = 0, item.title = ~`title #{}` count+1, item.filename = ~`filename_{}` count, count = count + 1}",
        false,
        true,
    )?;
    execute_single_line(
        "empties(42).{let count = 0, item.title = ~`title #{}` count+1, item.filename = ~`filename_{}` count, count = count + 1}",
        false,
        true,
    )?;
    execute_single_line("empties(0)", true, true)
}

#[test]
fn execute_nonemptyfilter_line() -> Result<(), InterpreterError> {
    execute_single_line("files().(??)", false, true)?;
    execute_single_line("empties(42).(??)", true, true)
}

#[test]
fn execute_mpdfunction_line() -> Result<(), InterpreterError> {
    execute_single_line("mpd(`127.0.0.1:6600`, artist=`Bruno Mars`)", false, true)?;
    execute_single_line(
        "mpd(`127.0.0.1:6600`, title=`something very long that should match absolutely nothing, probably, hopefully...`)",
        true,
        true,
    )?;
    #[cfg(feature = "ergonomics")]
    execute_single_line("mpd(`localhost:6600`)", false, true)?;
    #[cfg(feature = "ergonomics")]
    execute_single_line("mpd(`default`)", false, true)?;
    execute_single_line("mpd(`127.0.0.1:6600`)", false, true)
}
