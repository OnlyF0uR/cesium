#[cfg(debug_assertions)]
macro_rules! log_current_char {
    ($lexer:expr) => {
        println!("{:?}", $lexer.current_char());
    };
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Define,             // Define keyword
    Version,            // Version keyword
    Schemes,            // Schemes keyword
    Include(String),    // Include keyword
    Address,            // Address keyword
    Consts,             // Consts keyword
    Function,           // Function keyword
    Return,             // Return keyword
    Number(String),     // Number
    Identifier(String), // Identifier
    Operator(String),   // Operator
    Comment(String),    // Comment
    String(String),     // String literal
    Whitespace,         // Whitespace
    LeftBrace,          // {
    RightBrace,         // }
    LeftBracket,        // [
    RightBracket,       // ]
    LeftParen,          // (
    RightParen,         // )
    Comma,              // ,
    SemiColon,          // ;
    Eof,                // End of file
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer { input, pos: 0 }
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) {
        self.pos += self.current_char().map_or(0, |c| c.len_utf8());
    }

    fn skip_whitespace(&mut self) {
        while self.current_char().map_or(false, |c| c.is_whitespace()) {
            self.advance();
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Token::Eof;
        }

        let current_char = self.current_char().unwrap();

        // Handle comments
        if current_char == '/' {
            if self.input[self.pos..].starts_with("//") {
                let start_pos = self.pos;
                while self.current_char().map_or(false, |c| c != '\n') {
                    self.advance();
                }
                let comment = &self.input[start_pos..self.pos];
                return Token::Comment(comment.to_string());
            }
        }

        // Handle identifiers and keywords
        if current_char == '$' || current_char.is_alphabetic() {
            let start_pos = self.pos;
            // Allow identifiers to start with $ and then be followed by letters or underscores
            while self
                .current_char()
                .map_or(false, |c| c.is_alphanumeric() || c == '_' || c == '$')
            {
                self.advance();
            }
            let identifier = &self.input[start_pos..self.pos];
            match identifier {
                "$define" => return Token::Define,
                "version" => return Token::Version,
                "schemes" => return Token::Schemes,
                "$include" => return self.tokenize_include(),
                "address" => return Token::Address,
                "consts" => return Token::Consts,
                "pub" => return Token::Function,
                "return" => return Token::Return,
                _ => return Token::Identifier(identifier.to_string()),
            }
        }

        // Handle string literals
        if current_char == '"' {
            self.advance(); // skip the opening quote
            let start_pos = self.pos;
            while self.current_char().map_or(false, |c| c != '"') {
                self.advance();
            }
            // Skip the closing quote
            if self.current_char() == Some('"') {
                self.advance();
            }
            let string_literal = &self.input[start_pos..self.pos - 1]; // -1 to exclude closing quote
            return Token::String(string_literal.to_string());
        }

        // Handle numbers
        if current_char.is_digit(10) || current_char == '.' {
            return self.tokenize_number(); // Handle number tokenization
        }

        // Handle operators and punctuation
        match current_char {
            '{' => {
                self.advance();
                return Token::LeftBrace;
            }
            '}' => {
                self.advance();
                return Token::RightBrace;
            }
            '[' => {
                self.advance();
                return Token::LeftBracket;
            }
            ']' => {
                self.advance();
                return Token::RightBracket;
            }
            '(' => {
                self.advance();
                return Token::LeftParen;
            }
            ')' => {
                self.advance();
                return Token::RightParen;
            }
            ',' => {
                self.advance();
                return Token::Comma;
            }
            ';' => {
                self.advance();
                return Token::SemiColon;
            }
            _ => {
                self.advance();
                return Token::Operator(current_char.to_string());
            }
        }
    }

    fn tokenize_include(&mut self) -> Token {
        // if there is a whitespace after $include, skip it
        if self.current_char().map_or(false, |c| c.is_whitespace()) {
            self.skip_whitespace();
        }

        // Open the quote
        if self.current_char() == Some('"') {
            self.advance();
        }

        // read till next quote
        let start_pos = self.pos;
        // while alphbetical or numeric or underscore
        while self.current_char().map_or(false, |c| c != '"') {
            self.advance();
        }

        let include = &self.input[start_pos..self.pos];
        self.advance(); // close the quote
        Token::Include(include.to_string())
    }

    fn tokenize_number(&mut self) -> Token {
        let start_pos = self.pos;
        let mut has_exponent = false;

        while self.current_char().map_or(false, |c| {
            c.is_digit(10) || c == '.' || c == 'e' || c == 'E'
        }) {
            if self.current_char() == Some('e') || self.current_char() == Some('E') {
                has_exponent = true;
            }

            self.advance();
        }

        let number = &self.input[start_pos..self.pos];

        if has_exponent {
            // Handle scientific notation like 10e12
            if let Some(exponent) = self.parse_exponent(number) {
                // Expand the number into a string format
                let base = &number[..number.find('e').unwrap()];
                let expanded_number = self.expand_scientific_notation(base, exponent);
                return Token::Number(expanded_number);
            }
        }

        // Return the number as a string if there's no exponent
        Token::Number(number.to_string())
    }

    fn parse_exponent(&self, number: &str) -> Option<i32> {
        if let Some(index) = number.find('e').or_else(|| number.find('E')) {
            let exponent = &number[(index + 1)..]; // Extract exponent (after 'e')

            // Parse the exponent as i32, and ignore anything that doesn't parse correctly
            if let Ok(exponent_value) = exponent.parse::<i32>() {
                return Some(exponent_value);
            }
        }

        None // Return None if it's not a valid exponent
    }

    fn expand_scientific_notation(&self, base: &str, exponent: i32) -> String {
        // Generate a string representation of the expanded number
        let base_number: String = base.chars().collect(); // Keep the base as a string
        let mut expanded_number = base_number.to_string();

        // Append zeros based on the exponent value
        for _ in 0..exponent {
            expanded_number.push('0');
        }

        expanded_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define() {
        let input = r#"
    // This is a comment
    $define {
      version = "^0.1.0"
      schemes = [
        {
          preset = "token@0.1.0"
          params = {
            decimals = 12
            total_supply = 10e12 * 5
            name = ["coolium", "COOL"]
          }
        }
      ]
    }

    $include "file.se"
    "#;

        let mut lexer = Lexer::new(input);
        let mut i = 0;
        loop {
            let token = lexer.next_token();
            println!("{:?}", token);
            if token == Token::Eof {
                break;
            }
            i += 1;
        }

        assert_eq!(i, 36);
    }
}
