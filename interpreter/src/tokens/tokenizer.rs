use std::collections::VecDeque;

use super::Token;
use super::ParseError;

pub trait TokenReader {
    fn current_line(&self) -> usize;

    fn current_column(&self) -> usize;

    fn next_statement(&mut self, token_buffer: &mut VecDeque<Token>) -> Result<(), ParseError>;

    fn end_of_file(&self) -> bool;
}

pub struct Tokenizer<R>
where
    R: std::io::Read,
{
    reader: R,
    fsm: ReaderStateMachine,
    line: usize,
    column: usize,
}

impl<R> Tokenizer<R>
where
    R: std::io::Read,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            fsm: ReaderStateMachine::Start {},
            line: 0,
            column: 0,
        }
    }

    pub fn read_line(&mut self, buf: &mut VecDeque<Token>) -> Result<(), ParseError> {
        let mut byte_buf = [0_u8];
        // first read special case
        // always read before checking if end of statement
        // since FSM could be from previous (already ended) statement
        if self
            .reader
            .read(&mut byte_buf)
            .map_err(|e| self.error(format!("IO read error: {}", e)))?
            == 0
        {
            byte_buf[0] = 0; // clear to null char (nothing read is assumed to mean end of file)
        }
        //println!("tokenizer read char: {}", byte_buf[0]);
        self.do_tracking(byte_buf[0]);
        self.fsm = self.fsm.next_state(byte_buf[0]);
        let mut bigger_buf: Vec<u8> = Vec::new();
        while !(self.fsm.is_end_statement() || self.fsm.is_end_of_file()) {
            // keep token's bytes
            if let Some(out) = self.fsm.output() {
                bigger_buf.push(out);
            }
            // handle parse endings
            match self.fsm {
                ReaderStateMachine::EndLiteral {} => {
                    let literal = String::from_utf8(bigger_buf.clone())
                        .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
                    buf.push_back(Token::Literal(literal));
                    bigger_buf.clear();
                }
                ReaderStateMachine::EndComment {} => {
                    //let _comment = String::from_utf8(bigger_buf.clone())
                    //    .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
                    // ignore comments
                    //buf.push_back(Token::Comment(comment));
                    bigger_buf.clear();
                }
                ReaderStateMachine::EndToken {} => {
                    if !bigger_buf.is_empty() {
                        // ignore consecutive end tokens
                        let token = String::from_utf8(bigger_buf.clone())
                            .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
                        buf.push_back(
                            Token::parse_from_string(token)
                                .map_err(|e| self.error(format!("invalid token `{}`", e)))?,
                        );
                        bigger_buf.clear();
                    }
                }
                ReaderStateMachine::SingleCharToken { .. } => {
                    let out = bigger_buf.pop().unwrap(); // bracket or comma token
                    if !bigger_buf.is_empty() {
                        // bracket tokens can be beside other tokens, without separator
                        let token = String::from_utf8(bigger_buf.clone())
                            .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
                        buf.push_back(
                            Token::parse_from_string(token)
                                .map_err(|e| self.error(format!("invalid token `{}`", e)))?,
                        );
                        bigger_buf.clear();
                    }
                    // process bracket token
                    bigger_buf.push(out);
                    let token = String::from_utf8(bigger_buf.clone())
                        .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
                    buf.push_back(
                        Token::parse_from_string(token)
                            .map_err(|e| self.error(format!("invalid token `{}`", e)))?,
                    );
                    bigger_buf.clear();
                }
                ReaderStateMachine::EndStatement {} => {
                    // unnecessary; loop will have already exited
                }
                ReaderStateMachine::EndOfFile {} => {
                    // unnecessary; loop will have already exited
                }
                ReaderStateMachine::Invalid { .. } => {
                    let invalid_char = bigger_buf.pop().unwrap(); // invalid single char
                                                                  // clear everything, to avoid further errors
                    bigger_buf.clear();
                    buf.clear();
                    return match invalid_char {
                        0 => Err(self.error("EOF".to_string())),
                        _ => Err(self.error(format!(
                            "character {:?} ({})",
                            invalid_char as char, invalid_char
                        ))),
                    };
                }
                _ => {}
            }
            if self
                .reader
                .read(&mut byte_buf)
                .map_err(|e| self.error(format!("IO read error: {}", e)))?
                == 0
            {
                byte_buf[0] = 0; // clear to null char (nothing read is assumed to mean end of file)
            }
            self.do_tracking(byte_buf[0]);
            self.fsm = self.fsm.next_state(byte_buf[0]);
        }
        // handle end statement
        if !bigger_buf.is_empty() {
            // also end of token
            // note: never also end of literal, since those have explicit closing characters
            let token = String::from_utf8(bigger_buf.clone())
                .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
            buf.push_back(
                Token::parse_from_string(token)
                    .map_err(|e| self.error(format!("invalid token `{}`", e)))?,
            );
            bigger_buf.clear();
        }
        Ok(())
    }

    /// track line and column locations
    fn do_tracking(&mut self, input: u8) {
        if input as char == '\n' {
            self.line += 1;
            self.column = 0;
        } else if input != 0 {
            self.column += 1; // TODO correctly track columns with utf-8 characters longer than one byte
        }
    }

    /// error factory (for ergonomics/DRY)
    fn error(&self, item: String) -> ParseError {
        ParseError {
            line: self.current_line(),
            column: self.current_column(),
            item,
        }
    }
}

impl<R> TokenReader for Tokenizer<R>
where
    R: std::io::Read,
{
    fn current_line(&self) -> usize {
        self.line
    }

    fn current_column(&self) -> usize {
        self.column
    }

    fn next_statement(&mut self, buf: &mut VecDeque<Token>) -> Result<(), ParseError> {
        // read until buffer gets some tokens, in case multiple end of line tokens are at start of stream
        let original_size = buf.len();
        self.read_line(buf)?; // always try once, even if at end of file
        while original_size == buf.len() && !self.end_of_file() {
            self.read_line(buf)?;
        }
        Ok(())
    }

    fn end_of_file(&self) -> bool {
        self.fsm.is_end_of_file()
    }
}

#[derive(Copy, Clone)]
enum ReaderStateMachine {
    Start {}, // beginning of machine, no parsing has occured
    Regular {
        out: u8,
    }, // standard
    Escaped {
        inside: char, // literal
    }, // escape character; applied to next character
    StartTickLiteral {},
    StartQuoteLiteral {},
    InsideTickLiteral {
        out: u8,
    },
    InsideQuoteLiteral {
        out: u8,
    },
    SingleCharToken {
        out: u8,
    },
    Slash {
        out: u8,
    },
    Octothorpe {
        out: u8,
    },
    Comment {
        out: u8,
    },
    EndLiteral {},
    EndToken {},
    EndComment {},
    EndStatement {},
    EndOfFile {},
    Invalid {
        out: u8,
    },
}

impl ReaderStateMachine {
    pub fn next_state(self, input: u8) -> Self {
        let input_char = input as char;
        match self {
            Self::Start {}
            | Self::Regular { .. }
            | Self::SingleCharToken { .. }
            | Self::EndLiteral {}
            | Self::EndToken {}
            | Self::EndComment {}
            | Self::EndStatement {}
            | Self::EndOfFile {}
            | Self::Invalid { .. } => match input_char {
                '\\' => Self::Escaped { inside: '_' },
                '/' => Self::Slash { out: input },
                '#' => Self::Octothorpe { out: input },
                '`' => Self::StartTickLiteral {},
                '"' => Self::StartQuoteLiteral {},
                '\n' | '\r' | '\t' | ' ' => Self::EndToken {},
                ';' => Self::EndStatement {},
                '\0' => Self::EndOfFile {},
                '(' | ')' | ',' | '=' | '<' | '>' | '.' | '!' | '?' | '|' | '&' | ':' | '{'
                | '}' | '+' | '-' | '~' => Self::SingleCharToken { out: input },
                _ => Self::Regular { out: input },
            },
            Self::Escaped { inside } => match inside {
                '`' => Self::InsideTickLiteral { out: input },
                '"' => Self::InsideQuoteLiteral { out: input },
                '_' | _ => Self::Regular { out: input },
            },
            Self::StartTickLiteral {} | Self::InsideTickLiteral { .. } => match input_char {
                '\\' => Self::Escaped { inside: '`' },
                '`' => Self::EndLiteral {},
                '\0' => Self::Invalid { out: input },
                _ => Self::InsideTickLiteral { out: input },
            },
            Self::StartQuoteLiteral {} | Self::InsideQuoteLiteral { .. } => match input_char {
                '\\' => Self::Escaped { inside: '"' },
                '"' => Self::EndLiteral {},
                '\0' => Self::Invalid { out: input },
                _ => Self::InsideQuoteLiteral { out: input },
            },
            Self::Slash { .. } => match input_char {
                '/' => Self::Comment { out: input },
                ' ' => Self::EndToken {},
                '\0' => Self::EndOfFile {},
                ';' => Self::EndStatement {},
                _ => Self::Regular { out: input },
            },
            Self::Octothorpe { .. } => match input_char {
                '\n' | '\r' | '\0' => Self::EndComment {},
                _ => Self::Comment { out: input },
            },
            Self::Comment { .. } => match input_char {
                '\n' | '\r' | '\0' => Self::EndComment {},
                _ => Self::Comment { out: input },
            },
            //Self::EndOfFile {} => Self::EndOfFile {}, // For REPL, the end of the file is not necessarily the end forever
        }
    }

    pub fn is_end_statement(&self) -> bool {
        match self {
            Self::EndStatement {} => true,
            _ => false,
        }
    }

    pub fn is_end_of_file(&self) -> bool {
        match self {
            Self::EndOfFile {} => true,
            _ => false,
        }
    }

    pub fn output(&self) -> Option<u8> {
        match self {
            Self::Regular { out, .. }
            | Self::SingleCharToken { out, .. }
            | Self::InsideTickLiteral { out, .. }
            | Self::InsideQuoteLiteral { out, .. }
            | Self::Slash { out, .. }
            | Self::Octothorpe { out, .. }
            | Self::Comment { out, .. }
            | Self::Invalid { out, .. } => Some(*out),
            _ => None,
        }
    }
}
