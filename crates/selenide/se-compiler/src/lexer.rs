use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Define,
    Version,
    Schemes,
    State,
    Consts,
    Include(&'a str),
    Procedures,
    Address,
    U128,
    U8,
    Bool,
    Table,
    PubFModifier,
    MutFModifier,
    Return,
    Number(String), // String so we don't need to box leak it
    Identifier(&'a str),
    Operator(&'a str),
    Comment(&'a str),
    String(&'a str),
    Whitespace,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Comma,
    SemiColon,
    Period,
    Eof,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    inner_lexer: Option<Box<Lexer<'a>>>,
    included_files: Vec<String>, // Store owned Strings
    working_dir: &'a str,
    keywords: &'static HashMap<&'static str, Token<'static>>,
}

impl<'a> Lexer<'a> {
    fn build_keyword_map() -> HashMap<&'static str, Token<'static>> {
        let mut keywords = HashMap::new();
        keywords.insert("$define", Token::Define);
        keywords.insert("version", Token::Version);
        keywords.insert("schemes", Token::Schemes);
        keywords.insert("$state", Token::State);
        keywords.insert("$consts", Token::Consts);
        keywords.insert("$procedures", Token::Procedures);
        keywords.insert("address", Token::Address);
        keywords.insert("table", Token::Table);
        keywords.insert("u128", Token::U128);
        keywords.insert("u8", Token::U8);
        keywords.insert("bool", Token::Bool);
        keywords.insert("pub", Token::PubFModifier);
        keywords.insert("mut", Token::MutFModifier);
        keywords.insert("return", Token::Return);
        keywords
    }

    pub fn new(input: &'a str, working_dir: &'a str) -> Self {
        static KEYWORDS: OnceLock<HashMap<&'static str, Token<'static>>> = OnceLock::new();
        let keywords = KEYWORDS.get_or_init(Self::build_keyword_map);

        Lexer {
            input,
            pos: 0,
            inner_lexer: None,
            included_files: Vec::with_capacity(10),
            working_dir,
            keywords,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            self.pos += c.len_utf8();
        }
    }

    fn skip_whitespace(&mut self) {
        while self.current_char().map_or(false, |c| c.is_whitespace()) {
            self.advance();
        }
    }

    pub fn next_token(&mut self) -> Token<'a> {
        if let Some(inner) = self.inner_lexer.as_mut() {
            let token = inner.next_token();
            if token == Token::Eof {
                self.inner_lexer = None;
                return self.next_token();
            }
            return token;
        }

        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Token::Eof;
        }

        let current_char = self.current_char().unwrap();

        if current_char == '/' && self.input[self.pos..].starts_with("//") {
            let start_pos = self.pos;
            while self.current_char().map_or(false, |c| c != '\n') {
                self.advance();
            }
            return Token::Comment(&self.input[start_pos..self.pos]);
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

            if identifier == "$include" {
                return self.tokenize_include();
            }

            if let Some(token) = self.keywords.get(identifier) {
                return token.clone();
            }

            return Token::Identifier(identifier);
        }

        if current_char == '"' {
            self.advance();
            let start_pos = self.pos;
            while self.current_char().map_or(false, |c| c != '"') {
                self.advance();
            }
            let end_pos = self.pos;
            self.advance();
            return Token::String(&self.input[start_pos..end_pos]);
        }

        if current_char.is_digit(10) || current_char == '.' {
            return self.tokenize_number();
        }

        self.advance();
        match current_char {
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            ',' => Token::Comma,
            ';' => Token::SemiColon,
            '.' => Token::Period,
            _ => Token::Operator(&self.input[self.pos - 1..self.pos]),
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
        self.advance();

        self.load_header(include);
        Token::Include(include)
    }

    fn load_header(&mut self, filename: &str) {
        let included_file_path = Path::new(self.working_dir).join(filename);
        let file_content =
            std::fs::read_to_string(included_file_path).expect("Failed to read included file");

        // Create a new lexer with a static reference
        let content = Box::leak(file_content.into_boxed_str());
        self.included_files.push(content.to_string()); // Store for potential cleanup

        self.inner_lexer = Some(Box::new(Lexer::new(content, self.working_dir)));
    }

    fn tokenize_number(&mut self) -> Token<'a> {
        let start_pos = self.pos;
        let mut has_exponent = false;

        while self.current_char().map_or(false, |c| {
            c.is_digit(10) || c == '.' || c == 'e' || c == 'E'
        }) {
            if matches!(self.current_char(), Some('e' | 'E')) {
                has_exponent = true;
            }
            self.advance();
        }

        let number = &self.input[start_pos..self.pos];

        if has_exponent {
            if let Some(exponent) = self.parse_exponent(number) {
                if let Some(base_end) = number.find(['e', 'E']) {
                    let base = &number[..base_end];
                    let expanded = self.expand_scientific_notation(base, exponent);
                    return Token::Number(expanded);
                }
            }
        }

        if number == "." {
            return Token::Period;
        }

        Token::Number(number.to_string())
    }

    fn parse_exponent(&self, number: &str) -> Option<i32> {
        number
            .find(['e', 'E'])
            .and_then(|index| number[(index + 1)..].parse::<i32>().ok())
    }

    fn expand_scientific_notation(&self, base: &str, exponent: i32) -> String {
        let mut expanded = base.to_string();
        for _ in 0..exponent {
            expanded.push('0');
        }
        expanded
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
        let mut token_count = 0;

        loop {
            let token = lexer.next_token();
            println!("{:?}", token);
            if token == Token::Eof {
                break;
            }
            token_count += 1;
        }

        assert_eq!(token_count, 114);
    }

    #[test]
    fn test_numbers_and_scientific_notation() {
        let input = "123 1e5";
        let mut lexer = Lexer::new(input, "");

        assert_eq!(lexer.next_token(), Token::Number("123".to_string()));
        assert_eq!(lexer.next_token(), Token::Number("100000".to_string()));
    }
}
