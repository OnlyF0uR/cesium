use crate::lexer::{Lexer, Token};

#[allow(unused_macros)]
macro_rules! log_current_token {
    ($parser:expr) => {
        println!("Current token: {:?}", $parser.current_token);
    };
}

#[derive(Debug)]
pub enum VariableType {
    U128,
    U8,
    Address,
    String,
    Bool,
    Array(Box<VariableType>),
}

#[derive(Debug)]
pub enum ASTNode {
    Number(String),
    StringLiteral(String),
    Comment(String),
    Array(Vec<ASTNode>),
    Address(String),

    Root(Vec<ASTNode>),
    Define {
        version: Option<String>,
        schemes: Vec<ASTNode>,
    },
    Schemes(Vec<ASTNode>),
    Scheme {
        preset: String,
        params: Vec<(String, ASTNode)>,
    },
    State(Vec<ASTNode>),
    StateVariableDeclaration {
        name: String,
        var_type: VariableType,
    },
    Consts(Vec<ASTNode>), // <-- New node type for const declarations
    ConstDeclaration {
        name: String,
        var_type: VariableType,
        value: Box<ASTNode>,
    },
    Procedures(Vec<ASTNode>),
    Function {
        name: String,
        public: bool,
        mutates: bool,
        params: Vec<(String, VariableType)>,
        body: Vec<ASTNode>,
    },
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
                Token::State => {
                    // Parse a state block and push it to the root.
                    root.push(self.parse_state_block());
                }
                Token::Consts => {
                    // Parse a consts block and push it to the root.
                    root.push(self.parse_consts_block());
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

        self.expect_operator("=");
        let value = self.expect_string("Expected string value for version");

        ("version".to_owned(), value)
    }

    /// Parses schemes from the define statement and returns them as a Vec of ASTNodes.
    fn parse_schemes(&mut self) -> Vec<ASTNode> {
        self.next_token();
        self.expect_operator("=");

        self.expect_token(Token::LeftBracket, "Expected '[' to start schemes");
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

        self.expect_token(Token::RightBracket, "Expected ']' to end schemes");
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
        self.expect_token(
            Token::Identifier("preset"),
            "Expected 'preset' to start scheme",
        );
        self.expect_operator("=");
        self.expect_string("Expected string value for preset")
    }

    /// Parses parameters from a scheme and returns them as an ASTNode.
    fn parse_params(&mut self) -> Vec<(String, ASTNode)> {
        self.expect_token(
            Token::Identifier("params"),
            "Expected 'params' to start scheme",
        );

        self.expect_operator("=");
        self.expect_token(Token::LeftBrace, "Expected '{' to start params");

        let mut params = Vec::new();
        // Loop for as long as the params are not closed with '}'
        while self.current_token != Token::RightBrace && self.current_token != Token::Eof {
            let id = self.get_identifier();
            self.expect_operator("=");

            let value = self.expect_value();
            params.push((id.to_string(), value));
        }

        params
    }

    fn parse_state_block(&mut self) -> ASTNode {
        self.expect_token(Token::State, "Expected '$state' keyword");
        self.expect_token(Token::LeftBrace, "Expected '{' after '$state'");

        let mut state_variables = Vec::new();
        // Loop for as long as the state is not closed with '}'
        while self.current_token != Token::RightBrace && self.current_token != Token::Eof {
            let var_type = self.expect_variable_type();
            let var_name = self.get_identifier();

            state_variables.push(ASTNode::StateVariableDeclaration {
                name: var_name,
                var_type,
            });

            self.expect_token(
                Token::SemiColon,
                "Expected ';' at the end of the state variable declaration",
            );
        }

        self.expect_token(
            Token::RightBrace,
            "Expected '}' at the end of the state block",
        );
        ASTNode::State(state_variables)
    }

    fn parse_consts_block(&mut self) -> ASTNode {
        self.expect_token(Token::Consts, "Expected '$consts' keyword");
        self.expect_token(Token::LeftBrace, "Expected '{' after '$consts'");

        let mut const_variables = Vec::new();
        // Loop for as long as the consts block is not closed with '}'
        while self.current_token != Token::RightBrace && self.current_token != Token::Eof {
            let var_type = self.expect_variable_type();
            let var_name = self.get_identifier();
            self.expect_operator("=");
            let value = self.expect_value();

            const_variables.push(ASTNode::ConstDeclaration {
                name: var_name,
                var_type,
                value: Box::new(value),
            });

            self.expect_token(
                Token::SemiColon,
                "Expected ';' at the end of the state variable declaration",
            );
        }

        self.expect_token(
            Token::RightBrace,
            "Expected '}' at the end of the consts block",
        );

        ASTNode::Consts(const_variables)
    }

    fn expect_value(&mut self) -> ASTNode {
        // It could be an array so we need to check for '['
        if self.current_token == Token::LeftBracket {
            // Now we must parse this array
            self.next_token(); // Move past '['

            let mut array = Vec::new();
            while self.current_token != Token::RightBracket {
                if let Token::String(value) = self.current_token {
                    array.push(ASTNode::StringLiteral(value.to_owned()));
                }
                self.next_token(); // Move to the next token
            }
            self.next_token(); // Move past ']'
            return ASTNode::Array(array);
        } else if let Token::Number(ref value) = self.current_token {
            let mut value = value.clone();
            self.next_token();

            // Handle plus operator
            while self.current_token == Token::Operator("+") {
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
            while self.current_token == Token::Operator("-") {
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
            while self.current_token == Token::Operator("*") {
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
            while self.current_token == Token::Operator("/") {
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
            while self.current_token == Token::Operator("%") {
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
            while self.current_token == Token::Operator("^") {
                self.next_token();
                if let Token::Number(ref value2) = self.current_token {
                    let original_value = value.parse::<u128>().unwrap(); // TODO: Handle errors
                    let next_value = value2.parse::<u32>().unwrap(); // TODO: Handle errors

                    value = (original_value.pow(next_value)).to_string();
                    self.next_token();
                } else {
                    panic!("Expected number after operator");
                }
            }

            return ASTNode::Number(value);
        } else if let Token::String(value) = self.current_token {
            return ASTNode::StringLiteral(value.to_owned());
        } else {
            panic!("Unexpected token in params");
        }
    }

    fn expect_token(&mut self, expected: Token<'a>, message: &str) {
        if self.current_token != expected {
            panic!("{}", message);
        }
        self.next_token();
    }

    fn get_identifier(&mut self) -> String {
        if let Token::Identifier(id) = self.current_token {
            self.next_token();
            id.to_owned()
        } else {
            panic!("Expected an identifier, found {:?}", self.current_token);
        }
    }

    fn expect_operator(&mut self, expected_op: &str) {
        if let Token::Operator(op) = &self.current_token {
            if *op == expected_op {
                self.next_token();
            } else {
                panic!("Expected '{}' operator", expected_op);
            }
        } else {
            panic!("Expected '{}' operator", expected_op);
        }
    }

    fn expect_string(&mut self, message: &str) -> String {
        if let Token::String(value) = self.current_token {
            self.next_token();
            value.to_owned()
        } else {
            panic!("{}", message);
        }
    }

    fn expect_variable_type(&mut self) -> VariableType {
        let t = match self.current_token {
            Token::Address => VariableType::Address,
            Token::U128 => VariableType::U128,
            Token::U8 => VariableType::U8,
            Token::Bool => VariableType::Bool,
            _ => panic!("Expected a type identifier"),
        };

        self.next_token();
        t
    }

    // fn expect_number(&mut self) -> String {
    //     if let Token::Number(value) = &self.current_token {
    //         let num = value.clone();
    //         self.next_token();
    //         num
    //     } else {
    //         panic!("Expected a numeric value");
    //     }
    // }
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

        $state {
            address creator;
        }

        $consts {
            u128 constant = 10;
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
