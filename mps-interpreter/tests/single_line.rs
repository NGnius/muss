use mps_interpreter::lang::MpsLanguageError;
use mps_interpreter::tokens::{MpsToken, MpsTokenizer, ParseError};
use mps_interpreter::*;
use std::collections::VecDeque;
use std::io::Cursor;

#[test]
fn parse_line() -> Result<(), ParseError> {
    let cursor = Cursor::new("sql(`SELECT * FROM songs;`)");
    let correct_tokens: Vec<MpsToken> = vec![
        MpsToken::Name("sql".into()),
        MpsToken::OpenBracket,
        MpsToken::Literal("SELECT * FROM songs;".into()),
        MpsToken::CloseBracket,
    ];

    let mut tokenizer = MpsTokenizer::new(cursor);
    let mut buf = VecDeque::<MpsToken>::new();
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
) -> Result<(), Box<dyn MpsLanguageError>> {
    println!("--- Executing MPS code: '{}' ---", line);
    let cursor = Cursor::new(line);

    let tokenizer = MpsTokenizer::new(cursor);
    let interpreter = MpsInterpretor::with_standard_vocab(tokenizer);

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
            println!("Got song `{}` (file: `{}`)", item.title, item.filename);
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
fn execute_sql_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("sql(`SELECT * FROM songs ORDER BY artist;`)", false, true)
}

#[test]
fn execute_simple_sql_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("song(`lov`)", false, true)
}

#[test]
fn execute_comment_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("// this is a comment", true, true)?;
    execute_single_line("# this is a special comment", true, true)
}

#[test]
fn execute_repeat_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("repeat(song(`Christmas in L.A.`))", false, false)?;
    execute_single_line("repeat(song(`Christmas in L.A.`), 4)", false, true)
}

#[test]
fn execute_sql_init_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "sql_init(generate = false, folder = `/home/ngnius/Music`)",
        true,
        true,
    )
}

#[test]
fn execute_assign_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "let some_var = repeat(song(`Christmas in L.A.`))",
        true,
        true,
    )?;
    execute_single_line("let some_var2 = 1234", true, true)
}

#[test]
fn execute_emptyfilter_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("song(`lov`).()", false, true)
}

#[test]
fn execute_fieldfilter_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("song(`lov`).(year >= 2020)", false, true)
}

#[test]
fn execute_files_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        r"files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`, re=``, recursive=false)",
        false,
        true,
    )
}
