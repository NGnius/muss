use std::iter::Iterator;

use super::tokens::MpsTokenReader;
use super::{MpsError, MpsFaye, MpsItem};

/// Wrapper for MpsFaye with a built-in callback function for every iteration of the interpreter.
pub struct MpsDebugger<'a, 'b, T>
where
    T: MpsTokenReader,
{
    interpreter: MpsFaye<'a, T>,
    transmuter: &'b dyn Fn(
        &mut MpsFaye<'a, T>,
        Option<Result<MpsItem, MpsError>>,
    ) -> Option<Result<MpsItem, MpsError>>,
}

impl<'a, 'b, T> MpsDebugger<'a, 'b, T>
where
    T: MpsTokenReader,
{
    /// Create a new instance of MpsDebugger using the provided interpreter and callback.
    pub fn new(
        faye: MpsFaye<'a, T>,
        item_handler: &'b dyn Fn(
            &mut MpsFaye<'a, T>,
            Option<Result<MpsItem, MpsError>>,
        ) -> Option<Result<MpsItem, MpsError>>,
    ) -> Self {
        Self {
            interpreter: faye,
            transmuter: item_handler,
        }
    }
}

impl<'a, 'b, T> Iterator for MpsDebugger<'a, 'b, T>
where
    T: MpsTokenReader,
{
    type Item = Result<MpsItem, MpsError>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.interpreter.next();
        let transmuted_next = (self.transmuter)(&mut self.interpreter, next_item);
        transmuted_next
    }
}
