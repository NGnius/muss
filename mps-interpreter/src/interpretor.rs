use std::collections::VecDeque;
use std::fs::File;
use std::iter::Iterator;
use std::path::Path;

use super::lang::{MpsLanguageDictionary, MpsLanguageError, MpsOp};
use super::tokens::MpsToken;
use super::MpsContext;
use super::MpsItem;
use super::MpsError;

/// The script interpreter.
/// Use MpsRunner for a better interface.
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
            tokenizer,
            buffer: VecDeque::new(),
            current_stmt: None,
            vocabulary: vocab,
            context: None,
        }
    }

    pub fn with_standard_vocab(tokenizer: T) -> Self {
        let mut result = Self {
            tokenizer,
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
            tokenizer,
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
    type Item = Result<MpsItem, MpsError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut is_stmt_done = false;
        let result = if let Some(stmt) = &mut self.current_stmt {
            let next_item = stmt.next();
            if next_item.is_none() {
                is_stmt_done = true;
            }
            next_item
                .map(|item| item.map_err(|e| error_with_ctx(e, self.tokenizer.current_line())))
        } else {
            /*if self.tokenizer.end_of_file() {
                return None;
            }*/
            //println!("try get next statement");
            // build new statement
            let token_result = self
                .tokenizer
                .next_statement(&mut self.buffer)
                .map_err(|e| error_with_ctx(e, self.tokenizer.current_line()));
            match token_result {
                Ok(_) => {}
                Err(x) => return Some(Err(x)),
            }
            if self.tokenizer.end_of_file() && self.buffer.is_empty() {
                return None;
            }
            let stmt = self.vocabulary.try_build_statement(&mut self.buffer);
            match stmt {
                Ok(mut stmt) => {
                    #[cfg(debug_assertions)]
                    if !self.buffer.is_empty() {
                        panic!("Token buffer was not emptied! (rem: {:?})", self.buffer)
                    }
                    stmt.enter(self.context.take().unwrap_or_default());
                    self.current_stmt = Some(stmt);
                    let next_item = self.current_stmt.as_mut().unwrap().next();
                    if next_item.is_none() {
                        is_stmt_done = true;
                    }
                    next_item.map(|item| {
                        item.map_err(|e| error_with_ctx(e, self.tokenizer.current_line()))
                    })
                }
                Err(e) => {
                    Some(Err(e).map_err(|e| error_with_ctx(e, self.tokenizer.current_line())))
                }
            }
        };
        if is_stmt_done {
            self.context = Some(self.current_stmt.take().unwrap().escape());
        }
        result
    }
}

fn error_with_ctx<T: std::convert::Into<MpsError>>(error: T, line: usize) -> MpsError {
    let mut err = error.into();
    err.set_line(line);
    err
}

/// Builder function to add the standard statements of MPS.
pub(crate) fn standard_vocab(vocabulary: &mut MpsLanguageDictionary) {
    vocabulary
        // filters
        .add(crate::lang::vocabulary::filters::empty_filter())
        .add(crate::lang::vocabulary::filters::unique_filter()) // accepts .(unique)
        .add(crate::lang::vocabulary::filters::field_filter()) // accepts any .(something)
        .add(crate::lang::vocabulary::filters::field_filter_maybe())
        .add(crate::lang::vocabulary::filters::index_filter())
        .add(crate::lang::vocabulary::filters::range_filter())
        .add(crate::lang::vocabulary::filters::field_like_filter())
        .add(crate::lang::vocabulary::filters::field_re_filter())
        .add(crate::lang::vocabulary::filters::unique_field_filter())
        // sorters
        .add(crate::lang::vocabulary::sorters::empty_sort())
        .add(crate::lang::vocabulary::sorters::shuffle_sort()) // accepts ~(shuffle)
        .add(crate::lang::vocabulary::sorters::bliss_sort())
        .add(crate::lang::vocabulary::sorters::bliss_next_sort())
        .add(crate::lang::vocabulary::sorters::field_sort()) // accepts any ~(something)
        // iter blocks
        .add(
            crate::lang::MpsItemBlockFactory::new()
                .add(crate::lang::vocabulary::item_ops::ConstantItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::VariableAssignItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::FieldAssignItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::FileItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::VariableDeclareItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::InterpolateStringItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::BranchItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::IterItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::ConstructorItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::EmptyItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::RemoveItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::VariableRetrieveItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::NegateItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::NotItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::CompareItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::AddItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::SubtractItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::OrItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::AndItemOpFactory)
                .add(crate::lang::vocabulary::item_ops::BracketsItemOpFactory),
        )
        // functions and misc
        // functions don't enforce bracket coherence
        // -- function().() is valid despite the ).( in between brackets
        .add(crate::lang::vocabulary::sql_function_factory())
        .add(crate::lang::vocabulary::simple_sql_function_factory())
        .add(crate::lang::vocabulary::CommentStatementFactory)
        .add(crate::lang::vocabulary::repeat_function_factory())
        .add(crate::lang::vocabulary::AssignStatementFactory)
        .add(crate::lang::vocabulary::sql_init_function_factory())
        .add(crate::lang::vocabulary::files_function_factory())
        .add(crate::lang::vocabulary::empty_function_factory())
        .add(crate::lang::vocabulary::empties_function_factory())
        .add(crate::lang::vocabulary::reset_function_factory())
        .add(crate::lang::vocabulary::union_function_factory())
        .add(crate::lang::vocabulary::intersection_function_factory());
}
