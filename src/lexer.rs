#[derive(Debug, PartialEq)]
pub enum Token {
    Instruction(String),
    Register(String),
    Number(f32),
    Label(String),
    Comma,
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        // check if we reached end of file
        if self.pos >= self.input.len() {
            return Token::EOF;
        }

        let ch = self.input[self.pos];

        // handle comments starting with semicolon
        if ch == ';' {
            self.skip_comment();
            return self.next_token();
        }

        // handle explicit separators
        if ch == ',' {
            self.pos += 1;
            return Token::Comma;
        }

        // parse alphabetic identifiers or registers
        if ch.is_alphabetic() {
            return self.read_identifier();
        }

        // parse numbers including negative and floating point
        if ch.is_numeric() || ch == '-' || ch == '.' {
            return self.read_number();
        }

        // skip unknown characters and move forward
        self.pos += 1;
        self.next_token()
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        // consume all alphanumeric characters in a row
        while self.pos < self.input.len() && (self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_') {
            self.pos += 1;
        }

        let text: String = self.input[start..self.pos].iter().collect();

        // check if identifier is a label
        if self.pos < self.input.len() && self.input[self.pos] == ':' {
            self.pos += 1;
            return Token::Label(text);
        }

        // check if identifier matches register pattern
        if text.starts_with('r') && text.len() > 1 && text[1..].chars().all(|c| c.is_numeric()) {
            return Token::Register(text);
        }

        Token::Instruction(text)
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;
        let mut has_digit = false;

        // handle signs and digits
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_numeric() {
                has_digit = true;
                self.pos += 1;
            } else if ch == '.' || ch == '-' {
                self.pos += 1;
            } else {
                break;
            }
        }

        let text: String = self.input[start..self.pos].iter().collect();

        // ensure we actually consumed something valid
        if !has_digit && text == "-" {
            return self.next_token();
        }

        let val = text.parse::<f32>().unwrap_or(0.0);
        Token::Number(val)
    }

    fn skip_whitespace(&mut self) {
        // move past spaces tabs and newlines
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn skip_comment(&mut self) {
        // move to the end of current line
        while self.pos < self.input.len() && self.input[self.pos] != '\n' {
            self.pos += 1;
        }
    }
}