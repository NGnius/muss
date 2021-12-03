use std::io::Cursor;
use std::collections::VecDeque;
use mps_interpreter::*;
use mps_interpreter::lang::MpsLanguageError;
use mps_interpreter::tokens::{ParseError, MpsToken, MpsTokenizer};

#[test]
fn parse_line() -> Result<(), ParseError> {
    let cursor = Cursor::new("sql(`SELECT * FROM songs;`)");
    let correct_tokens: Vec<MpsToken> = vec![
        MpsToken::Sql,
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
        assert_eq!(buf[i], correct_tokens[i]);
    }

    tokenizer.read_line(&mut buf)?; // this should immediately return
    Ok(())
}

#[test]
fn execute_line() -> Result<(), Box<dyn MpsLanguageError>> {
    let cursor = Cursor::new("sql(`SELECT * FROM songs ORDER BY artist;`)");

    let tokenizer = MpsTokenizer::new(cursor);
    let interpreter = MpsInterpretor::with_standard_vocab(tokenizer);

    let mut count = 0;
    for result in interpreter {
        if let Ok(item) = result {
            count +=1;
            if count > 100 {continue;} // no need to spam the rest of the songs
            println!("Got song `{}` (file: `{}`)", item.title, item.filename);
        } else {
            println!("Got error while iterating (executing)");
            result?;
        }
    }
    assert_ne!(count, 0); // database is populated
    Ok(())
}
