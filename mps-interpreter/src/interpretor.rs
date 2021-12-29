use std::collections::VecDeque;
use std::fs::File;
use std::iter::Iterator;
use std::path::Path;

use super::lang::{MpsLanguageDictionary, MpsLanguageError, MpsOp};
use super::tokens::MpsToken;
use super::MpsContext;
use super::MpsMusicItem;

pub struct MpsInterpretor<T>
where
    T: crate::tokens::MpsTokenReader,
{
    tokenizer: T,
    buffer: VecDeque<MpsToken>,
    current_stmt: Option<Box<dyn MpsOp>>,
    vocabulary: MpsLanguageDictionary,
    context: Option<MpsContext>,
}

pub fn interpretor<R: std::io::Read>(stream: R) -> MpsInterpretor<crate::tokens::MpsTokenizer<R>> {
    let tokenizer = crate::tokens::MpsTokenizer::new(stream);
    MpsInterpretor::with_standard_vocab(tokenizer)
}

impl<T> MpsInterpretor<T>
where
    T: crate::tokens::MpsTokenReader,
{
    pub fn with_vocab(tokenizer: T, vocab: MpsLanguageDictionary) -> Self {
        Self {
            tokenizer: tokenizer,
            buffer: VecDeque::new(),
            current_stmt: None,
            vocabulary: vocab,
            context: None,
        }
    }

    pub fn with_standard_vocab(tokenizer: T) -> Self {
        let mut result = Self {
            tokenizer: tokenizer,
            buffer: VecDeque::new(),
            current_stmt: None,
            vocabulary: MpsLanguageDictionary::default(),
            context: None,
        };
        standard_vocab(&mut result.vocabulary);
        result
    }

    pub fn context(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    pub fn is_done(&self) -> bool {
        self.tokenizer.end_of_file() && self.current_stmt.is_none()
    }
}

impl MpsInterpretor<crate::tokens::MpsTokenizer<File>> {
    pub fn standard_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let tokenizer = crate::tokens::MpsTokenizer::new(file);
        let mut result = Self {
            tokenizer: tokenizer,
            buffer: VecDeque::new(),
            current_stmt: None,
            vocabulary: MpsLanguageDictionary::default(),
            context: None,
        };
        standard_vocab(&mut result.vocabulary);
        Ok(result)
    }
}

impl<T> Iterator for MpsInterpretor<T>
where
    T: crate::tokens::MpsTokenReader,
{
    type Item = Result<MpsMusicItem, Box<dyn MpsLanguageError>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut is_stmt_done = false;
        let result = if let Some(stmt) = &mut self.current_stmt {
            let next_item = stmt.next();
            if next_item.is_none() {
                is_stmt_done = true;
            }
            match next_item {
                Some(item) => {
                    Some(item.map_err(|e| box_error_with_ctx(e, self.tokenizer.current_line())))
                }
                None => None,
            }
        } else {
            if self.tokenizer.end_of_file() {
                return None;
            }
            // build new statement
            let token_result = self
                .tokenizer
                .next_statement(&mut self.buffer)
                .map_err(|e| box_error_with_ctx(e, self.tokenizer.current_line()));
            match token_result {
                Ok(_) => {}
                Err(x) => return Some(Err(x)),
            }
            if self.tokenizer.end_of_file() && self.buffer.len() == 0 {
                return None;
            }
            let stmt = self.vocabulary.try_build_statement(&mut self.buffer);
            match stmt {
                Ok(mut stmt) => {
                    #[cfg(debug_assertions)]
                    if self.buffer.len() != 0 {panic!("Token buffer was not emptied! (rem: {:?})", self.buffer)}
                    stmt.enter(self.context.take().unwrap_or_else(|| MpsContext::default()));
                    self.current_stmt = Some(stmt);
                    let next_item = self.current_stmt.as_mut().unwrap().next();
                    if next_item.is_none() {
                        is_stmt_done = true;
                    }
                    match next_item {
                        Some(item) => Some(
                            item.map_err(|e| box_error_with_ctx(e, self.tokenizer.current_line())),
                        ),
                        None => None,
                    }
                }
                Err(e) => {
                    Some(Err(e).map_err(|e| box_error_with_ctx(e, self.tokenizer.current_line())))
                }
            }
        };
        if is_stmt_done {
            self.context = Some(self.current_stmt.take().unwrap().escape());
        }
        result
    }
}

fn box_error_with_ctx<E: MpsLanguageError + 'static>(
    mut error: E,
    line: usize,
) -> Box<dyn MpsLanguageError> {
    error.set_line(line);
    Box::new(error) as Box<dyn MpsLanguageError>
}

pub(crate) fn standard_vocab(vocabulary: &mut MpsLanguageDictionary) {
    vocabulary
        // high-priority vocabulary (low-priority may accept this, but will not execute as expected)
        .add(crate::lang::vocabulary::filters::empty_filter())
        .add(crate::lang::vocabulary::filters::field_filter())
        // low-priority (more forgiving statements which may not parse complete statement)
        .add(crate::lang::vocabulary::SqlStatementFactory)
        .add(crate::lang::vocabulary::SimpleSqlStatementFactory)
        .add(crate::lang::vocabulary::CommentStatementFactory)
        .add(crate::lang::vocabulary::RepeatStatementFactory)
        .add(crate::lang::vocabulary::AssignStatementFactory)
        .add(crate::lang::vocabulary::SqlInitStatementFactory);
}
