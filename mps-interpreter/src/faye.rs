use std::collections::VecDeque;
use std::io::Read;
use std::iter::Iterator;

use super::lang::{MpsLanguageDictionary, MpsLanguageError, MpsOp};
use super::tokens::{MpsToken, MpsTokenizer};
use super::MpsContext;
use super::MpsError;
use super::MpsItem;

const DEFAULT_TOKEN_BUFFER_SIZE: usize = 16;

pub enum MpsDebuggableEvent {
    FileEnd,
    StatementComplete,
    NewStatementReady,
}

/// The script interpreter.
pub struct MpsFaye<'a, T>
where
    T: crate::tokens::MpsTokenReader,
{
    tokenizer: T,
    buffer: VecDeque<MpsToken>,
    current_stmt: Box<dyn MpsOp>,
    vocabulary: MpsLanguageDictionary,
    callback: &'a dyn Fn(&mut MpsFaye<'a, T>, MpsDebuggableEvent) -> Result<(), MpsError>,
}

#[inline]
fn empty_callback<'a, T: crate::tokens::MpsTokenReader>(
    _s: &mut MpsFaye<'a, T>,
    _d: MpsDebuggableEvent,
) -> Result<(), MpsError> {
    Ok(())
}

/*impl <T> MpsFaye<'static, T>
where
    T: crate::tokens::MpsTokenReader,
{
    /// Create a new interpreter for the provided token reader, using the standard MPS language.
    #[inline]
    pub fn with_standard_vocab(token_reader: T) -> Self {
        let mut vocab = MpsLanguageDictionary::default();
        super::interpretor::standard_vocab(&mut vocab);
        Self::with_vocab(vocab, token_reader)
    }

    /// Create a new interpreter with the provided vocabulary and token reader.
    #[inline]
    pub fn with_vocab(vocab: MpsLanguageDictionary, token_reader: T) -> Self {
        Self::with(vocab, token_reader, &empty_callback)
    }
}*/

impl<'a, R: Read> MpsFaye<'a, MpsTokenizer<R>> {
    pub fn with_stream(stream: R) -> Self {
        let tokenizer = MpsTokenizer::new(stream);
        Self::with_standard_vocab(tokenizer)
    }
}

impl<'a, T> MpsFaye<'a, T>
where
    T: crate::tokens::MpsTokenReader,
{
    #[inline]
    pub fn with_standard_vocab(token_reader: T) -> Self {
        let mut vocab = MpsLanguageDictionary::default();
        super::interpretor::standard_vocab(&mut vocab);
        Self::with_vocab(vocab, token_reader)
    }

    /// Create a new interpreter with the provided vocabulary and token reader.
    #[inline]
    pub fn with_vocab(vocab: MpsLanguageDictionary, token_reader: T) -> Self {
        Self::with(vocab, token_reader, &empty_callback)
    }

    /// Create a custom interpreter instance.
    #[inline]
    pub fn with(
        vocab: MpsLanguageDictionary,
        token_reader: T,
        debugger: &'a dyn Fn(&mut MpsFaye<'a, T>, MpsDebuggableEvent) -> Result<(), MpsError>,
    ) -> Self {
        Self {
            tokenizer: token_reader,
            buffer: VecDeque::with_capacity(DEFAULT_TOKEN_BUFFER_SIZE),
            current_stmt: Box::new(crate::lang::vocabulary::empty::EmptyStatement {
                context: Some(MpsContext::default()),
            }),
            vocabulary: vocab,
            callback: debugger,
        }
    }

    #[inline]
    fn new_statement(
        tokenizer: &mut T,
        buffer: &mut VecDeque<MpsToken>,
        vocab: &MpsLanguageDictionary,
    ) -> Option<Result<Box<dyn MpsOp>, MpsError>> {
        while !tokenizer.end_of_file() && buffer.is_empty() {
            let result = tokenizer.next_statement(buffer);
            match result {
                Ok(_) => {}
                Err(e) => return Some(Err(error_with_ctx(e, tokenizer.current_line()))),
            }
        }
        if buffer.is_empty() {
            return None;
        }
        let result = vocab.try_build_statement(buffer);
        let stmt = match result {
            Ok(stmt) => stmt,
            Err(e) => return Some(Err(error_with_ctx(e, tokenizer.current_line()))),
        };
        #[cfg(debug_assertions)]
        if !buffer.is_empty() {
            panic!("Token buffer was not emptied! (rem: {:?})", buffer)
        }
        Some(Ok(stmt))
    }
}

impl<'a, T> Iterator for MpsFaye<'a, T>
where
    T: crate::tokens::MpsTokenReader,
{
    type Item = Result<MpsItem, MpsError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current_stmt.next() {
                Some(item) => {
                    return Some(item.map_err(|e| error_with_ctx(e, self.tokenizer.current_line())))
                }
                None => {
                    // current_stmt has terminated
                    if self.tokenizer.end_of_file() {
                        // notify reached end of file
                        let callback_result = (self.callback)(self, MpsDebuggableEvent::FileEnd);
                        match callback_result {
                            Ok(_) => {}
                            Err(e) => return Some(Err(e)),
                        }
                        // nothing left to return
                        return None;
                    } else {
                        // notify old statement is complete
                        let callback_result =
                            (self.callback)(self, MpsDebuggableEvent::StatementComplete);
                        match callback_result {
                            Ok(_) => {}
                            Err(e) => return Some(Err(e)),
                        }
                        // build next statement
                        let result = Self::new_statement(
                            &mut self.tokenizer,
                            &mut self.buffer,
                            &self.vocabulary,
                        );
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
                            (self.callback)(self, MpsDebuggableEvent::NewStatementReady);
                        match callback_result {
                            Ok(_) => {}
                            Err(e) => return Some(Err(e)),
                        }
                    }
                }
            }
        }
    }
}

fn error_with_ctx<T: std::convert::Into<MpsError>>(error: T, line: usize) -> MpsError {
    let mut err = error.into();
    err.set_line(line);
    err
}
