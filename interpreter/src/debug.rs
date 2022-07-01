use std::iter::Iterator;

use super::tokens::TokenReader;
use super::{InterpreterError, Interpreter, Item};

/// Wrapper for InterpreterError with a built-in callback function for every iteration of the interpreter.
pub struct Debugger<'a, 'b, T>
where
    T: TokenReader,
{
    interpreter: Interpreter<'a, T>,
    transmuter: &'b dyn Fn(
        &mut Interpreter<'a, T>,
        Option<Result<Item, InterpreterError>>,
    ) -> Option<Result<Item, InterpreterError>>,
}

impl<'a, 'b, T> Debugger<'a, 'b, T>
where
    T: TokenReader,
{
    /// Create a new instance of Debugger using the provided interpreter and callback.
    pub fn new(
        faye: Interpreter<'a, T>,
        item_handler: &'b dyn Fn(
            &mut Interpreter<'a, T>,
            Option<Result<Item, InterpreterError>>,
        ) -> Option<Result<Item, InterpreterError>>,
    ) -> Self {
        Self {
            interpreter: faye,
            transmuter: item_handler,
        }
    }
}

impl<'a, 'b, T> Iterator for Debugger<'a, 'b, T>
where
    T: TokenReader,
{
    type Item = Result<Item, InterpreterError>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.interpreter.next();
        let transmuted_next = (self.transmuter)(&mut self.interpreter, next_item);
        transmuted_next
    }
}
