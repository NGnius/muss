use std::collections::VecDeque;
use std::io::Read;
use std::iter::Iterator;

use super::lang::{LanguageDictionary, LanguageError, Op};
use super::tokens::{Token, TokenReader, Tokenizer};
use super::Context;
use super::InterpreterError;
use super::Item;

const DEFAULT_TOKEN_BUFFER_SIZE: usize = 16;

pub enum InterpreterEvent {
    FileEnd,
    StatementComplete,
    NewStatementReady,
}

/// The script interpreter.
pub struct Interpreter<'a, T>
where
    T: TokenReader,
{
    tokenizer: T,
    buffer: VecDeque<Token>,
    current_stmt: Box<dyn Op>,
    vocabulary: LanguageDictionary,
    callback: &'a dyn Fn(&mut Interpreter<'a, T>, InterpreterEvent) -> Result<(), InterpreterError>,
}

#[inline]
fn empty_callback<T: TokenReader>(
    _s: &mut Interpreter<'_, T>,
    _d: InterpreterEvent,
) -> Result<(), InterpreterError> {
    Ok(())
}

/*impl <T> Interpreter<'static, T>
where
    T: TokenReader,
{
    /// Create a new interpreter for the provided token reader, using the standard MPS language.
    #[inline]
    pub fn with_standard_vocab(token_reader: T) -> Self {
        let mut vocab = LanguageDictionary::default();
        super::interpretor::standard_vocab(&mut vocab);
        Self::with_vocab(vocab, token_reader)
    }

    /// Create a new interpreter with the provided vocabulary and token reader.
    #[inline]
    pub fn with_vocab(vocab: LanguageDictionary, token_reader: T) -> Self {
        Self::with(vocab, token_reader, &empty_callback)
    }
}*/

impl<'a, R: Read> Interpreter<'a, Tokenizer<R>> {
    pub fn with_stream(stream: R) -> Self {
        let tokenizer = Tokenizer::new(stream);
        Self::with_standard_vocab(tokenizer)
    }

    pub fn with_stream_and_callback(
        stream: R,
        callback: &'a dyn Fn(
            &mut Interpreter<'a, Tokenizer<R>>,
            InterpreterEvent,
        ) -> Result<(), InterpreterError>,
    ) -> Self {
        let tokenizer = Tokenizer::new(stream);
        let vocab = LanguageDictionary::standard();
        Self::with(vocab, tokenizer, callback)
    }
}

impl<'a, T> Interpreter<'a, T>
where
    T: TokenReader,
{
    #[inline]
    pub fn with_standard_vocab(token_reader: T) -> Self {
        let vocab = LanguageDictionary::standard();
        Self::with_vocab(vocab, token_reader)
    }

    /// Create a new interpreter with the provided vocabulary and token reader.
    #[inline]
    pub fn with_vocab(vocab: LanguageDictionary, token_reader: T) -> Self {
        Self::with(vocab, token_reader, &empty_callback)
    }

    /// Create a custom interpreter instance.
    #[inline]
    pub fn with(
        vocab: LanguageDictionary,
        token_reader: T,
        callback: &'a dyn Fn(
            &mut Interpreter<'a, T>,
            InterpreterEvent,
        ) -> Result<(), InterpreterError>,
    ) -> Self {
        Self {
            tokenizer: token_reader,
            buffer: VecDeque::with_capacity(DEFAULT_TOKEN_BUFFER_SIZE),
            current_stmt: Box::new(crate::lang::vocabulary::empty::EmptyStatement {
                context: Some(Context::default()),
            }),
            vocabulary: vocab,
            callback: callback,
        }
    }

    // build a new statement
    #[inline]
    fn new_statement(&mut self) -> Option<Result<Box<dyn Op>, InterpreterError>> {
        while !self.tokenizer.end_of_file() && self.buffer.is_empty() {
            let result = self.tokenizer.next_statement(&mut self.buffer);
            match result {
                Ok(_) => {}
                Err(e) => return Some(Err(error_with_ctx(e, self.tokenizer.current_line()))),
            }
        }
        if self.buffer.is_empty() {
            let callback_result = (self.callback)(self, InterpreterEvent::FileEnd);
            match callback_result {
                Ok(_) => {}
                Err(e) => return Some(Err(e)),
            }
            return None;
        }
        let result = self.vocabulary.try_build_statement(&mut self.buffer);
        let stmt = match result {
            Ok(stmt) => stmt,
            Err(e) => return Some(Err(error_with_ctx(e, self.tokenizer.current_line()))),
        };
        #[cfg(debug_assertions)]
        if !self.buffer.is_empty() {
            panic!("Token buffer was not emptied! (rem: {:?})", self.buffer)
        }
        Some(Ok(stmt))
    }
}

pub type InterpreterItem = Result<Item, InterpreterError>;

impl<'a, T> Iterator for Interpreter<'a, T>
where
    T: TokenReader,
{
    type Item = InterpreterItem;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current_stmt.next() {
                Some(item) => {
                    return Some(item.map_err(|e| error_with_ctx(e, self.tokenizer.current_line())))
                }
                None => {
                    // current_stmt has terminated
                    if self.tokenizer.end_of_file() {
                        // always try to read at least once, in case stream gets new data (e.g. in a REPL)
                        let result = self.tokenizer.next_statement(&mut self.buffer);
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                return Some(Err(error_with_ctx(e, self.tokenizer.current_line())))
                            }
                        }
                    } else {
                        // notify old statement is complete
                        let callback_result =
                            (self.callback)(self, InterpreterEvent::StatementComplete);
                        match callback_result {
                            Ok(_) => {}
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    // build next statement
                    let result = self.new_statement();
                    let mut stmt = match result {
                        Some(Ok(stmt)) => stmt,
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None,
                    };
                    let ctx = self.current_stmt.escape();
                    stmt.enter(ctx);
                    self.current_stmt = stmt;
                    // notify new statement is ready
                    let callback_result =
                        (self.callback)(self, InterpreterEvent::NewStatementReady);
                    match callback_result {
                        Ok(_) => {}
                        Err(e) => return Some(Err(e)),
                    }
                }
            }
        }
    }
}

fn error_with_ctx<T: std::convert::Into<InterpreterError>>(
    error: T,
    line: usize,
) -> InterpreterError {
    let mut err = error.into();
    err.set_line(line);
    err
}
