use std::iter::Iterator;

use super::tokens::TokenReader;
use super::{Interpreter, InterpreterItem};

/// Wrapper for InterpreterError with a built-in callback function for every iteration of the interpreter.
pub struct Debugger<'a, T, F>
where
    T: TokenReader,
    F: Fn(&mut Interpreter<'a, T>, Option<InterpreterItem>) -> Option<InterpreterItem>,
{
    interpreter: Interpreter<'a, T>,
    transmuter: F,
}

impl<'a, T, F> Debugger<'a, T, F>
where
    T: TokenReader,
    F: Fn(&mut Interpreter<'a, T>, Option<InterpreterItem>) -> Option<InterpreterItem>,
{
    /// Create a new instance of Debugger using the provided interpreter and callback.
    pub fn new(faye: Interpreter<'a, T>, item_handler: F) -> Self {
        Self {
            interpreter: faye,
            transmuter: item_handler,
        }
    }
}

impl<'a, T, F> Iterator for Debugger<'a, T, F>
where
    T: TokenReader,
    F: Fn(&mut Interpreter<'a, T>, Option<InterpreterItem>) -> Option<InterpreterItem>,
{
    type Item = InterpreterItem;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.interpreter.next();
        (self.transmuter)(&mut self.interpreter, next_item)
    }
}
