use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::MpsContext;
use crate::MpsMusicItem;
use crate::tokens::MpsToken;

use super::{RuntimeError, SyntaxError};
use super::{MpsOp, SimpleMpsOpFactory, MpsOpFactory, BoxedMpsOpFactory};
use super::MpsLanguageDictionary;
use super::utility::assert_token;

#[derive(Debug, Clone)]
pub struct CommentStatement {
    comment: String,
    context: Option<MpsContext>
}

impl CommentStatement {
    /*fn comment_text(&self) -> String {
        let mut clone = self.comment.clone();
        if clone.starts_with("#") {
            clone.replace_range(..1, ""); // remove "#"
        } else {
            clone.replace_range(..2, ""); // remove "//"
        }
        clone
    }*/
}

impl Display for CommentStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.comment)
    }
}

impl Iterator for CommentStatement {
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl MpsOp for CommentStatement {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
    }
}

pub struct CommentStatementFactory;

impl SimpleMpsOpFactory<CommentStatement> for CommentStatementFactory {
    fn is_op_simple(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() == 1
        && tokens[0].is_comment()
    }

    fn build_op_simple(
        &self,
        tokens: &mut VecDeque<MpsToken>,
    ) -> Result<CommentStatement, SyntaxError> {
        let comment = assert_token(|t| match t {
            MpsToken::Comment(c) => Some(c),
            _ => None
        }, MpsToken::Comment("comment".into()), tokens)?;
        Ok(CommentStatement {
            comment: comment,
            context: None
        })
    }
}

impl BoxedMpsOpFactory for CommentStatementFactory {
    fn build_op_boxed(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsOp>, SyntaxError> {
        self.build_box(tokens, dict)
    }

    fn is_op_boxed(&self, tokens: &VecDeque<MpsToken>) -> bool {
        self.is_op(tokens)
    }
}
