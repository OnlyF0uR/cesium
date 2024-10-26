#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Define,
    Version,
    Schemes,
    State,
    Consts,
    Include(&'a str),
    Address,
    U128,
    Table,
    Function,
    Return,
    Number(String),
    Identifier(String),
    Operator(String),
    Comment(String),
    String(String),
    Whitespace,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Comma,
    SemiColon,
    Eof,
}

pub struct Lexer<'a> {
    input: &'a str, // Reference to the input string
    pos: usize,
    inner_lexer: Option<Box<Lexer<'a>>>, // Inner lexer for included files with the same lifetime as the outer lexer
    included_files: Vec<&'a str>,        // Store included file contents
    working_dir: &'a str,                // Working directory for the lexer
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, working_dir: &'a str) -> Lexer<'a> {
        Lexer {
            input,
            pos: 0,
            inner_lexer: None,
            included_files: Vec::new(), // Initialize the included files vector
            working_dir,
        }
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

    pub fn next_token(&mut self) -> Token<'a> {
        // Check if we have an inner lexer to process first
        if let Some(inner) = self.inner_lexer.as_mut() {
            let token = inner.next_token();
            if token == Token::Eof {
                self.inner_lexer = None; // Clear inner lexer when done
            }
            return token;
        }

        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Token::Eof;
        }

        let current_char = self.current_char().unwrap();

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

        if current_char == '$' || current_char.is_alphabetic() {
            let start_pos = self.pos;
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
                "$state" => return Token::State,
                "$consts" => return Token::Consts,
                "$include" => return self.tokenize_include(),
                "address" => return Token::Address,
                "table" => return Token::Table,
                "u128" => return Token::U128,
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
            if self.current_char() == Some('"') {
                self.advance();
            }
            let string_literal = &self.input[start_pos..self.pos - 1];
            return Token::String(string_literal.to_string());
        }

        // Handle numbers
        if current_char.is_digit(10) || current_char == '.' {
            return self.tokenize_number();
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

    fn tokenize_include(&mut self) -> Token<'a> {
        self.skip_whitespace();

        if self.current_char() == Some('"') {
            self.advance();
        }

        let start_pos = self.pos;
        while self.current_char().map_or(false, |c| c != '"') {
            self.advance();
        }

        let include = &self.input[start_pos..self.pos];
        self.advance(); // close the quote

        // Load the content of the included file
        self.load_header(include);
        Token::Include(include) // Return the include token
    }

    fn load_header(&mut self, filename: &str) {
        let included_file_path = std::path::Path::new(self.working_dir).join(filename);
        let file_content = std::fs::read_to_string(included_file_path).unwrap();

        // Store the included content and create a new lexer for it
        let included_content: &'a str = Box::leak(file_content.into_boxed_str());
        self.included_files.push(included_content);

        // Create a new lexer for the included content with the same lifetime as the outer lexer
        self.inner_lexer = Some(Box::new(Lexer::new(included_content, self.working_dir)));
    }

    fn tokenize_number(&mut self) -> Token<'a> {
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
            if let Some(exponent) = self.parse_exponent(number) {
                let base = &number[..number.find('e').unwrap()];
                let expanded_number = self.expand_scientific_notation(base, exponent);
                return Token::Number(expanded_number);
            }
        }

        Token::Number(number.to_string())
    }

    fn parse_exponent(&self, number: &str) -> Option<i32> {
        if let Some(index) = number.find('e').or_else(|| number.find('E')) {
            let exponent = &number[(index + 1)..];
            if let Ok(exponent_value) = exponent.parse::<i32>() {
                return Some(exponent_value);
            }
        }
        None
    }

    fn expand_scientific_notation(&self, base: &str, exponent: i32) -> String {
        let base_number: String = base.chars().collect();
        let mut expanded_number = base_number.to_string();
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
        let w_path = "../../../examples/selenide/create_token";
        let main_path = format!("{}/main.se", w_path);

        let input = std::fs::read_to_string(main_path).unwrap();

        let mut lexer = Lexer::new(&input, w_path);
        let mut i = 0;
        loop {
            let token = lexer.next_token();
            println!("{:?}", token);
            if token == Token::Eof {
                break;
            }
            i += 1;
        }

        assert_eq!(i, 61);
    }
}
