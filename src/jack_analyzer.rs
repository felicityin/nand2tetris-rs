use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;

use once_cell::sync::OnceCell;

use crate::jack_tokenizer::{Token, TokenType};
use crate::utils::save_file;

pub static CLASS_DEC: OnceCell<HashSet<&str>> = OnceCell::new();
pub static FUNC_DEC: OnceCell<HashSet<&str>> = OnceCell::new();
pub static STATEMENTS: OnceCell<HashSet<&str>> = OnceCell::new();
pub static BINARY_OP: OnceCell<HashSet<char>> = OnceCell::new();
pub static UNARY_OP: OnceCell<HashSet<char>> = OnceCell::new();

pub struct JackAnalyzer {
    dest_path: PathBuf,
    tokens:    TokenStream,
    tree:      Vec<Token>,
}

impl JackAnalyzer {
    pub fn new(mut path: PathBuf, tokens: Vec<Token>) -> Self {
        CLASS_DEC.set(HashSet::from(["static", "field"])).unwrap();

        FUNC_DEC
            .set(HashSet::from(["constructor", "function", "method"]))
            .unwrap();

        STATEMENTS
            .set(HashSet::from(["let", "if", "while", "do", "return"]))
            .unwrap();

        BINARY_OP
            .set(HashSet::from(['+', '-', '*', '/', '&', '|', '<', '>', '=']))
            .unwrap();

        UNARY_OP.set(HashSet::from(['-', '~'])).unwrap();

        assert_eq!(path.extension().unwrap(), "jack");
        path.set_extension("tree.xml");

        Self {
            dest_path: path,
            tokens:    TokenStream::new(tokens),
            tree:      vec![],
        }
    }

    pub fn dest_path(&self) -> &PathBuf {
        &self.dest_path
    }

    pub fn analyze(&mut self) {
        self.compile_class();
    }

    fn step(&mut self, expected: &str) {
        let token = self.tokens.next();
        match token.category {
            TokenType::Keyword | TokenType::Symbol => {
                if token.value != expected {
                    panic!("invalid: {}, line: {}", token.value, token.line);
                }
            }
            _ => {}
        }
        self.tree.push(token);
    }

    fn step_identifier(&mut self) {
        let token = self.tokens.next();
        if token.category != TokenType::Identifier {
            panic!("invalid: {}, line: {}", token.value, token.line);
        }
        assert_eq!(token.category, TokenType::Identifier);
        self.tree.push(token);
    }

    fn step_type(&mut self) {
        let token = self.tokens.next();
        if token.category != TokenType::Keyword && token.category != TokenType::Identifier {
            panic!("invalid: {}, line: {}", token.value, token.line);
        }
        self.tree.push(token);
    }

    fn compile_class(&mut self) {
        self.tree.push(Token::unterminal("class", true));

        self.step("class"); // class
        self.step_identifier(); // className
        self.step("{"); // {
        while CLASS_DEC
            .get()
            .unwrap()
            .contains(self.tokens.peek().value.as_str())
        {
            self.compile_class_var_dec();
        }
        while FUNC_DEC
            .get()
            .unwrap()
            .contains(self.tokens.peek().value.as_str())
        {
            self.compile_subroutine();
        }
        self.step("}"); // }

        self.tree.push(Token::unterminal("class", false));
    }

    fn compile_class_var_dec(&mut self) {
        self.tree.push(Token::unterminal("classVarDec", true));

        self.tree.push(self.tokens.next()); // static | field
        self.step_type(); // type
        self.step_identifier(); // varName
        while self.tokens.peek().value.as_str() == "," {
            self.step(","); // ,
            self.step_identifier(); // varName
        }
        self.step(";"); // ;

        self.tree.push(Token::unterminal("classVarDec", false));
    }

    fn compile_subroutine(&mut self) {
        self.tree.push(Token::unterminal("subroutineDec", true));

        self.tree.push(self.tokens.next()); // constructor | function | method
        self.tree.push(self.tokens.next()); // void | type
        self.step_identifier(); // subroutineName
        self.step("("); // (
        self.compile_param_list();
        self.step(")"); // )

        self.tree.push(Token::unterminal("subroutineBody", true));

        self.step("{"); // {
        while self.tokens.peek().value.as_str() == "var" {
            self.compile_var_dec();
        }
        self.compile_statements();
        self.step("}"); // }

        self.tree.push(Token::unterminal("subroutineBody", false));
        self.tree.push(Token::unterminal("subroutineDec", false));
    }

    fn compile_param_list(&mut self) {
        self.tree.push(Token::unterminal("parameterList", true));

        if self.tokens.peek().value.as_str() != ")" {
            self.step_type(); // type
            self.step_identifier(); // varName
        }
        while self.tokens.peek().value.as_str() == "," {
            self.step(","); // ,
            self.step_type(); // type
            self.step_identifier(); // varName
        }

        self.tree.push(Token::unterminal("parameterList", false));
    }

    fn compile_var_dec(&mut self) {
        self.tree.push(Token::unterminal("varDec", true));

        self.step("var"); // var
        self.step_type(); // type
        self.step_identifier(); // varName
        while self.tokens.peek().value.as_str() == "," {
            self.step(","); // ,
            self.step_identifier(); // varName
        }
        self.step(";"); // ;

        self.tree.push(Token::unterminal("varDec", false));
    }

    fn compile_statements(&mut self) {
        self.tree.push(Token::unterminal("statements", true));

        while STATEMENTS
            .get()
            .unwrap()
            .contains(self.tokens.peek().value.as_str())
        {
            let v = self.tokens.peek().value;
            match v.as_str() {
                "if" => self.compile_if(),
                "let" => self.compile_let(),
                "while" => self.compile_while(),
                "do" => self.compile_do(),
                "return" => self.compile_return(),
                _ => unreachable!(),
            }
        }

        self.tree.push(Token::unterminal("statements", false));
    }

    fn compile_if(&mut self) {
        self.tree.push(Token::unterminal("ifStatement", true));

        self.step("if"); // if
        self.step("("); // (
        self.compile_expression();
        self.step(")"); // )
        self.step("{"); // {
        self.compile_statements();
        self.step("}"); // }
        if self.tokens.peek().value.as_str() == "else" {
            self.step("else"); // else
            self.step("{"); // {
            self.compile_statements();
            self.step("}"); // }
        }

        self.tree.push(Token::unterminal("ifStatement", false));
    }

    fn compile_let(&mut self) {
        self.tree.push(Token::unterminal("letStatement", true));

        self.step("let"); // let
        self.step_identifier(); // varName
        if self.tokens.peek().value.as_str() == "[" {
            self.step("["); // [
            self.compile_expression();
            self.step("]"); // ]
        }
        self.step("="); // =
        self.compile_expression();
        self.step(";"); // ;

        self.tree.push(Token::unterminal("letStatement", false));
    }

    fn compile_while(&mut self) {
        self.tree.push(Token::unterminal("whileStatement", true));

        self.step("while"); // while
        self.step("("); // (
        self.compile_expression();
        self.step(")"); // )
        self.step("{"); // {
        self.compile_statements();
        self.step("}"); // }

        self.tree.push(Token::unterminal("whileStatement", false));
    }

    fn compile_do(&mut self) {
        self.tree.push(Token::unterminal("doStatement", true));

        self.step("do"); // do
        self.tree.push(self.tokens.next()); // subroutineCall
        if self.tokens.peek().value.as_str() == "(" {
            self.step("("); // (
            self.compile_expression_list();
            self.step(")"); // )
        } else if self.tokens.peek().value.as_str() == "." {
            self.step("."); // .
            self.step_identifier(); // subroutineName
            self.step("("); // (
            self.compile_expression_list();
            self.step(")"); // )
        }
        self.step(";"); // ;

        self.tree.push(Token::unterminal("doStatement", false));
    }

    fn compile_return(&mut self) {
        self.tree.push(Token::unterminal("returnStatement", true));

        self.step("return"); // return
        if self.tokens.peek().value.as_str() != ";" {
            self.compile_expression();
        }
        self.step(";"); // ;

        self.tree.push(Token::unterminal("returnStatement", false));
    }

    fn compile_expression(&mut self) {
        self.tree.push(Token::unterminal("expression", true));

        self.compile_term();
        while BINARY_OP
            .get()
            .unwrap()
            .contains(&self.tokens.peek().value.chars().next().unwrap())
        {
            self.tree.push(self.tokens.next()); // op
            self.compile_term();
        }

        self.tree.push(Token::unterminal("expression", false));
    }

    fn compile_term(&mut self) {
        self.tree.push(Token::unterminal("term", true));

        if self.tokens.peek().value.as_str() == "(" {
            self.step("("); // (
            self.compile_expression();
            self.step(")"); // )
        } else if UNARY_OP
            .get()
            .unwrap()
            .contains(&self.tokens.peek().value.chars().next().unwrap())
        {
            self.tree.push(self.tokens.next()); // - or ~
            self.compile_term();
        } else {
            self.tree.push(self.tokens.next());
            match self.tokens.peek().value.as_str() {
                "[" => {
                    self.step("["); // [
                    self.compile_expression();
                    self.step("]"); // ]
                }
                "(" => {
                    self.step("("); // (
                    self.compile_expression_list();
                    self.step(")"); // )
                }
                "." => {
                    self.step("."); // .
                    self.step_identifier(); // subroutineName
                    self.step("("); // (
                    self.compile_expression_list();
                    self.step(")"); // )
                }
                _ => {}
            }
        }

        self.tree.push(Token::unterminal("term", false));
    }

    fn compile_expression_list(&mut self) {
        self.tree.push(Token::unterminal("expressionList", true));

        if self.tokens.peek().value.as_str() != ")" {
            self.compile_expression();
            while self.tokens.peek().value.as_str() == "," {
                self.step(","); // ,
                self.compile_expression();
            }
        }

        self.tree.push(Token::unterminal("expressionList", false));
    }

    pub fn save_file(&self) {
        let mut output: Vec<u8> = vec![];

        for token in self.tree.iter() {
            writeln!(&mut output, "{}", token.form).unwrap();
        }

        save_file(&output, &self.dest_path).unwrap();
    }
}

struct TokenStream {
    tokens: Vec<Token>,
    i:      usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, i: 0 }
    }

    pub fn next(&mut self) -> Token {
        let token = self.tokens[self.i].clone();
        self.i += 1;
        token
    }

    pub fn peek(&self) -> Token {
        self.tokens[self.i].clone()
    }

    #[allow(dead_code)]
    pub fn is_end(&self) -> bool {
        self.i >= self.tokens.len()
    }
}
