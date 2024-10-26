use crate::lexer::{Lexer, Token};

#[allow(unused_macros)]
macro_rules! log_current_token {
    ($parser:expr) => {
        println!("Current token: {:?}", $parser.current_token);
    };
}

#[derive(Debug)]
pub enum ASTNode {
    Root(Vec<ASTNode>), // Represents the whole root
    // General nodes
    Number(String),
    StringLiteral(String),
    Comment(String),
    Array(Vec<ASTNode>),

    // Define node stuff
    Define {
        version: Option<String>,
        schemes: Vec<ASTNode>,
    },
    Schemes(Vec<ASTNode>),
    Scheme {
        preset: String,
        params: Vec<(String, ASTNode)>,
    },
    // Function node stuff
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::Eof, // Initialize to end of file
        };
        parser.next_token(); // Load the first token
        parser
    }

    /// Advances the current token to the next token in the lexer.
    fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    /// Parses the entire input into a root represented as an AST.
    pub fn parse(&mut self) -> ASTNode {
        let mut root = Vec::new();

        while self.current_token != Token::Eof {
            match self.current_token {
                Token::Define => {
                    // Parse a define statement and push it to the root.
                    root.push(self.parse_define());
                }
                _ => {
                    // Handle unexpected tokens by advancing to the next token.
                    self.next_token();
                }
            }
        }

        // Return the root as an ASTNode.
        ASTNode::Root(root)
    }

    /// Parses a define statement and returns it as an ASTNode.
    fn parse_define(&mut self) -> ASTNode {
        self.next_token(); // Move past 'define'
        if self.current_token != Token::LeftBrace {
            panic!("Expected '{{' to start define block");
        }

        self.next_token(); // Move past '{'
        let mut version = None;
        let mut schemes = Vec::new();

        while self.current_token != Token::RightBrace && self.current_token != Token::Eof {
            match &self.current_token {
                Token::Version => {
                    version = Some(self.parse_version().1); // Store only the version string
                }
                Token::Schemes => {
                    schemes = self.parse_schemes(); // Store the schemes as ASTNodes
                }
                _ => {
                    // Handle unexpected tokens within the define block.
                    self.next_token();
                }
            }
        }

        // Ensure the define block is closed with '}'
        if self.current_token != Token::RightBrace {
            panic!("Expected '}}' to end define block");
        }

        self.next_token(); // Move past '}'

        // Construct and return the Define ASTNode with collected information
        ASTNode::Define { version, schemes }
    }

    /// Parses a version statement and returns it as an ASTNode.
    fn parse_version(&mut self) -> (String, String) {
        self.next_token(); // Move past 'version'
        if let Token::Operator(ref op) = self.current_token {
            if op != "=" {
                panic!("Expected '=' after version");
            }
        } else {
            panic!("Expected '=' after version");
        }
        self.next_token(); // Move past '='

        if let Token::String(ref value) = self.current_token {
            let version_value = value.clone();
            self.next_token(); // Move past the string
            return ("version".to_string(), version_value);
        }
        panic!("Expected string value for version");
    }

    /// Parses schemes from the define statement and returns them as a Vec of ASTNodes.
    fn parse_schemes(&mut self) -> Vec<ASTNode> {
        self.next_token(); // Move past '='
        if let Token::Operator(ref op) = self.current_token {
            if op != "=" {
                panic!("Expected '=' after schemes");
            }
        } else {
            panic!("Expected '=' after schemes");
        }
        self.next_token(); // move past '='

        if self.current_token != Token::LeftBracket {
            panic!("Expected '[' to start schemes");
        }

        self.next_token(); // Move past '['
        let mut schemes = Vec::new();

        while self.current_token != Token::RightBracket && self.current_token != Token::Eof {
            if self.current_token == Token::LeftBrace {
                self.next_token(); // Move past '{'
                schemes.push(self.parse_scheme()); // Parse each scheme

                // should end with '}'
                if self.current_token != Token::RightBrace {
                    panic!("Expected '}}' to end scheme");
                }
            }

            self.next_token(); // Move to the next token (away from '}')
                               // TODO: Check for ,
        }

        if self.current_token != Token::RightBracket {
            panic!("Expected ']' to end schemes");
        }
        self.next_token(); // Move past ']'

        schemes
    }

    /// Parses an individual scheme and returns it as an ASTNode.
    fn parse_scheme(&mut self) -> ASTNode {
        // A scheme consists of a preset and parameters
        let preset = self.parse_preset();
        let params = self.parse_params();

        let scheme: ASTNode = ASTNode::Scheme { preset, params };
        ASTNode::Schemes(vec![scheme]) // Return a new SchemeNode (update as needed)
    }

    /// Parses a preset value from a scheme and returns it as an ASTNode.
    fn parse_preset(&mut self) -> String {
        // move past 'preset'
        if self.current_token != Token::Identifier("preset".to_string()) {
            panic!("Expected 'preset' to start scheme");
        }
        self.next_token();

        // move past '='
        if let Token::Operator(ref op) = self.current_token {
            if op != "=" {
                panic!("Expected '=' after preset");
            }
        } else {
            panic!("Expected '=' after preset");
        }
        self.next_token();

        if let Token::String(ref value) = self.current_token {
            let value = value.clone();
            self.next_token(); // Move past the string
            return value; // Return preset as StringLiteral
        }
        panic!("Expected string value for preset");
    }

    /// Parses parameters from a scheme and returns them as an ASTNode.
    fn parse_params(&mut self) -> Vec<(String, ASTNode)> {
        // move past 'params'
        if self.current_token != Token::Identifier("params".to_string()) {
            panic!("Expected 'params' to start scheme");
        }
        self.next_token();

        // move past '='
        if let Token::Operator(ref op) = self.current_token {
            if op != "=" {
                panic!("Expected '=' after params");
            }
        } else {
            panic!("Expected '=' after params");
        }
        self.next_token();

        // move past '{'
        if self.current_token != Token::LeftBrace {
            panic!("Expected '{{' to start params");
        }
        self.next_token();

        let mut params = Vec::new();
        while self.current_token != Token::RightBrace && self.current_token != Token::Eof {
            if let Token::Identifier(ref id) = self.current_token {
                let id = id.clone(); // the key
                                     // Move past the identifier to the value
                self.next_token();

                // move past '='
                if let Token::Operator(ref op) = self.current_token {
                    if op != "=" {
                        panic!("Expected '=' after identifier");
                    }
                } else {
                    panic!("Expected '=' after identifier");
                }
                self.next_token();

                // It could be an array so we need to check for '['
                if self.current_token == Token::LeftBracket {
                    // Now we must parse this array
                    self.next_token(); // Move past '['

                    let mut array = Vec::new();
                    while self.current_token != Token::RightBracket {
                        if let Token::String(ref value) = self.current_token {
                            array.push(ASTNode::StringLiteral(value.clone()));
                        }
                        self.next_token(); // Move to the next token
                    }
                    self.next_token(); // Move past ']'
                    params.push((id, ASTNode::Array(array)));
                    self.next_token();
                } else if let Token::Number(ref value) = self.current_token {
                    let mut value = value.clone();
                    self.next_token();

                    // Handle plus operator
                    while self.current_token == Token::Operator("+".to_string()) {
                        self.next_token();
                        if let Token::Number(ref value2) = self.current_token {
                            let original_value = value.parse::<u128>().unwrap(); // TODO: Handle errors
                            let next_value = value2.parse::<u128>().unwrap(); // TODO: Handle errors

                            value = (original_value + next_value).to_string();
                            self.next_token();
                        } else {
                            panic!("Expected number after operator");
                        }
                    }
                    // Handle minus operator
                    while self.current_token == Token::Operator("-".to_string()) {
                        self.next_token();
                        if let Token::Number(ref value2) = self.current_token {
                            let original_value = value.parse::<u128>().unwrap(); // TODO: Handle errors
                            let next_value = value2.parse::<u128>().unwrap(); // TODO: Handle errors

                            value = (original_value - next_value).to_string();
                            self.next_token();
                        } else {
                            panic!("Expected number after operator");
                        }
                    }
                    // Handle multiplication operator
                    while self.current_token == Token::Operator("*".to_string()) {
                        self.next_token();
                        if let Token::Number(ref value2) = self.current_token {
                            let original_value = value.parse::<u128>().unwrap(); // TODO: Handle errors
                            let next_value = value2.parse::<u128>().unwrap(); // TODO: Handle errors

                            value = (original_value * next_value).to_string();
                            self.next_token();
                        } else {
                            panic!("Expected number after operator");
                        }
                    }
                    // Handle division operator
                    while self.current_token == Token::Operator("/".to_string()) {
                        self.next_token();
                        if let Token::Number(ref value2) = self.current_token {
                            let original_value = value.parse::<u128>().unwrap(); // TODO: Handle errors
                            let next_value = value2.parse::<u128>().unwrap(); // TODO: Handle errors

                            value = (original_value / next_value).to_string();
                            self.next_token();
                        } else {
                            panic!("Expected number after operator");
                        }
                    }
                    // Handle modulo operator
                    while self.current_token == Token::Operator("%".to_string()) {
                        self.next_token();
                        if let Token::Number(ref value2) = self.current_token {
                            let original_value = value.parse::<u128>().unwrap(); // TODO: Handle errors
                            let next_value = value2.parse::<u128>().unwrap(); // TODO: Handle errors

                            value = (original_value % next_value).to_string();
                            self.next_token();
                        } else {
                            panic!("Expected number after operator");
                        }
                    }
                    // Handle exponentiation operator
                    while self.current_token == Token::Operator("^".to_string()) {
                        self.next_token();
                        if let Token::Number(ref value2) = self.current_token {
                            let original_value = value.parse::<u128>().unwrap(); // TODO: Handle errors
                            let next_value = value2.parse::<u128>().unwrap(); // TODO: Handle errors

                            value = (original_value.pow(next_value as u32)).to_string();
                            self.next_token();
                        } else {
                            panic!("Expected number after operator");
                        }
                    }

                    params.push((id, ASTNode::Number(value)));
                } else if let Token::String(ref value) = self.current_token {
                    params.push((id, ASTNode::StringLiteral(value.clone())));
                    self.next_token();
                } else if let Token::RightBrace = self.current_token {
                    self.next_token();
                    break;
                } else {
                    panic!("Unexpected token in params");
                }
            } else {
                panic!("Expected identifier in params");
            }

            // this will also get rid of params }
            // self.next_token(); // Move to the next token
        }

        params // Return params as Consts node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_parsing() {
        let input = r#"
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
        "#;

        let lexer = Lexer::new(input, "");
        let mut parser = Parser::new(lexer);
        let ast = parser.parse();
        // Further assertions can be made here to validate the resulting AST
        println!("{:#?}", ast);

        // assert!(false); // for debug purposes
    }
}
