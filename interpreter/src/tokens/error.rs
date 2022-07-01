use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::LanguageError;

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub item: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "ParseError (line {}, column {}): Unexpected {}",
            &self.line, &self.column, &self.item
        )
    }
}

impl TokenError for ParseError {
    fn set_line(&mut self, line: usize) {
        self.line = line
    }

    fn set_column(&mut self, column: usize) {
        self.column = column
    }
}

pub trait TokenError: Display + Debug {
    fn set_line(&mut self, line: usize);

    fn set_column(&mut self, column: usize);

    fn set_location(&mut self, line: usize, column: usize) {
        self.set_line(line);
        self.set_column(column);
    }
}

impl<T: TokenError> LanguageError for T {
    fn set_line(&mut self, line: usize) {
        (self as &mut dyn TokenError).set_line(line);
    }
}
