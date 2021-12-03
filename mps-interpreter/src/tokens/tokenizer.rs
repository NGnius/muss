use std::collections::VecDeque;

use super::ParseError;
use super::MpsToken;

pub trait MpsTokenReader {
    fn current_line(&self) -> usize;

    fn current_column(&self) -> usize;

    fn next_statements(&mut self, count: usize, token_buffer: &mut VecDeque<MpsToken>) -> Result<(), ParseError>;

    fn end_of_file(&self) -> bool;
}

pub struct MpsTokenizer<R> where R: std::io::Read {
    reader: R,
    fsm: ReaderStateMachine,
    line: usize,
    column: usize,
}

impl<R> MpsTokenizer<R> where R: std::io::Read {
    pub fn new(reader: R) -> Self {
        Self {
            reader: reader,
            fsm: ReaderStateMachine::Start{},
            line: 0,
            column: 0,
        }
    }

    pub fn read_line(&mut self, buf: &mut VecDeque<MpsToken>) -> Result<(), ParseError> {
        let mut byte_buf = [0_u8];
        // first read special case
        // always read before checking if end of statement
        // since FSM could be from previous (already ended) statement
        if self.reader.read(&mut byte_buf).map_err(|e| self.error(format!("IO read error: {}", e)))? == 0 {
            byte_buf[0] = 0; // clear to null char (nothing read is assumed to mean end of file)
        }
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
                ReaderStateMachine::EndLiteral{} => {
                    let literal = String::from_utf8(bigger_buf.clone())
                        .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
                    buf.push_back(MpsToken::Literal(literal));
                    bigger_buf.clear();
                },
                ReaderStateMachine::EndToken{} => {
                    let token = String::from_utf8(bigger_buf.clone())
                        .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
                    buf.push_back(
                        MpsToken::parse_from_string(token)
                            .map_err(|e| self.error(format!("Invalid token {}", e)))?
                    );
                    bigger_buf.clear();
                },
                ReaderStateMachine::Bracket{..} => {
                    let out = bigger_buf.pop().unwrap(); // bracket token
                    if bigger_buf.len() != 0 { // bracket tokens can be beside other tokens, without separator
                        let token = String::from_utf8(bigger_buf.clone())
                            .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
                        buf.push_back(
                            MpsToken::parse_from_string(token)
                                .map_err(|e| self.error(format!("Invalid token {}", e)))?
                        );
                        bigger_buf.clear();
                    }
                    // process bracket token
                    bigger_buf.push(out);
                    let token = String::from_utf8(bigger_buf.clone())
                        .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
                    buf.push_back(
                        MpsToken::parse_from_string(token)
                            .map_err(|e| self.error(format!("Invalid token {}", e)))?
                    );
                    bigger_buf.clear();
                },
                ReaderStateMachine::EndStatement{} => {
                    // unnecessary; loop will have already exited
                },
                ReaderStateMachine::EndOfFile{} => {
                    // unnecessary; loop will have already exited
                },
                _ => {},
            }
            if self.reader.read(&mut byte_buf).map_err(|e| self.error(format!("IO read error: {}", e)))? == 0 {
                byte_buf[0] = 0; // clear to null char (nothing read is assumed to mean end of file)
            }
            self.do_tracking(byte_buf[0]);
            self.fsm = self.fsm.next_state(byte_buf[0]);
        }
        // handle end statement
        if bigger_buf.len() != 0 { // also end of token
            // note: never also end of literal, since those have explicit closing characters
            let token = String::from_utf8(bigger_buf.clone())
                .map_err(|e| self.error(format!("UTF-8 encoding error: {}", e)))?;
            buf.push_back(
                MpsToken::parse_from_string(token)
                    .map_err(|e| self.error(format!("Invalid token {}", e)))?
            );
            bigger_buf.clear();
        }
        Ok(())
    }

    /// track line and column locations
    fn do_tracking(&mut self, input: u8) {
        if input as char == '\n' {
            self.line += 1;
        }
        self.column += 1; // TODO correctly track columns with utf-8 characters longer than one byte
    }

    /// error factory (for ergonomics/DRY)
    fn error(&self, item: String) -> ParseError {
        ParseError {
            line: self.current_line(),
            column: self.current_column(),
            item: item,
        }
    }
}

impl<R> MpsTokenReader for MpsTokenizer<R>
where
    R: std::io::Read
{
    fn current_line(&self) -> usize {
        self.line
    }

    fn current_column(&self) -> usize {
        self.column
    }

    fn next_statements(&mut self, count: usize, buf: &mut VecDeque<MpsToken>) -> Result<(), ParseError> {
        for _ in 0..count {
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
    Start{}, // beginning of machine, no parsing has occured
    Regular{
        out: u8,
    }, // standard
    Escaped{
        inside: char, // literal
    }, // escape character; applied to next character
    StartTickLiteral{},
    StartQuoteLiteral{},
    InsideTickLiteral{
        out: u8,
    },
    InsideQuoteLiteral{
        out: u8,
    },
    Bracket {
        out: u8,
    },
    EndLiteral{},
    EndToken{},
    EndStatement{},
    EndOfFile{},
}

impl ReaderStateMachine {
    pub fn next_state(self, input: u8) -> Self {
        let input_char = input as char;
        match self {
            Self::Start{}
            | Self::Regular{..}
            | Self::Bracket{..}
            | Self::EndLiteral{}
            | Self::EndToken{}
            | Self::EndStatement{} =>
                match input_char {
                    '\\' => Self::Escaped{inside: '_'},
                    '`' => Self::StartTickLiteral{},
                    '"' => Self::StartQuoteLiteral{},
                    ' ' => Self::EndToken{},
                    '\n' | '\r' | ';' => Self::EndStatement{},
                    '\0' => Self::EndOfFile{},
                    '(' | ')' => Self::Bracket{out: input},
                    _ => Self::Regular{out: input},
                },
            Self::Escaped{inside} => match inside {
                '`' => Self::InsideTickLiteral{out: input},
                '"' => Self::InsideQuoteLiteral{out: input},
                '_' | _ => Self::Regular{out: input}
            },
            Self::StartTickLiteral{}
            | Self::InsideTickLiteral{..} =>
                match input_char {
                    '\\' => Self::Escaped{inside: '`'},
                    '`' => Self::EndLiteral{},
                    _ => Self::InsideTickLiteral{out: input},
                },
            Self::StartQuoteLiteral{}
            | Self::InsideQuoteLiteral{..} =>
                match input_char {
                    '\\' => Self::Escaped{inside: '"'},
                    '"' => Self::EndLiteral{},
                    _ => Self::InsideQuoteLiteral{out: input},
                },
            Self::EndOfFile{} => Self::EndOfFile{},
        }
    }

    pub fn is_end_statement(&self) -> bool {
        match self {
            Self::EndStatement{} => true,
            _ => false
        }
    }

    pub fn is_end_of_file(&self) -> bool {
        match self {
            Self::EndOfFile{} => true,
            _ => false
        }
    }

    pub fn output(&self) -> Option<u8> {
        match self {
            Self::Regular{ out, ..}
            | Self::Bracket{ out, ..}
            | Self::InsideTickLiteral{ out, ..}
            | Self::InsideQuoteLiteral{ out, ..} => Some(*out),
            _ => None
        }
    }
}
