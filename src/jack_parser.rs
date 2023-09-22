use std::collections::HashSet;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;

use once_cell::sync::OnceCell;

use crate::jack_tokenizer::{Token, TokenType};
use crate::symbol_table::VarKind;
use crate::utils::save_file;

pub static CLASS_DEC: OnceCell<HashSet<&str>> = OnceCell::new();
pub static FUNC_DEC: OnceCell<HashSet<&str>> = OnceCell::new();
pub static STATEMENTS: OnceCell<HashSet<&str>> = OnceCell::new();
pub static OP: OnceCell<HashSet<char>> = OnceCell::new();
pub static UNARY_OP: OnceCell<HashSet<char>> = OnceCell::new();

pub struct JackParser {
    tokens:           TokenStream,
    completed_tokens: Vec<Token>,
    class:            Class,
}

impl JackParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        CLASS_DEC.set(HashSet::from(["static", "field"])).unwrap();

        FUNC_DEC
            .set(HashSet::from(["constructor", "function", "method"]))
            .unwrap();

        STATEMENTS
            .set(HashSet::from(["let", "if", "while", "do", "return"]))
            .unwrap();

        OP.set(HashSet::from(['+', '-', '*', '/', '&', '|', '<', '>', '=']))
            .unwrap();

        UNARY_OP.set(HashSet::from(['-', '~'])).unwrap();

        Self {
            tokens:           TokenStream::new(tokens),
            completed_tokens: vec![],
            class:            Class::default(),
        }
    }

    pub fn ast(self) -> Class {
        self.class
    }

    pub fn run(&mut self) {
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
        self.completed_tokens.push(token);
    }

    fn step_identifier(&mut self) -> String {
        let token = self.tokens.next();
        if token.category != TokenType::Identifier {
            panic!("invalid: {}, line: {}", token.value, token.line);
        }
        assert_eq!(token.category, TokenType::Identifier);
        let value = token.value.clone();
        self.completed_tokens.push(token);
        value
    }

    fn step_type(&mut self) -> VarType {
        let token = self.tokens.next();
        if token.category != TokenType::Keyword && token.category != TokenType::Identifier {
            panic!("invalid: {}, line: {}", token.value, token.line);
        }
        let value = token.value.clone();
        self.completed_tokens.push(token);

        match value.as_str() {
            "int" => VarType::Int,
            "char" => VarType::Char,
            "boolean" => VarType::Boolean,
            _ => VarType::Class(value),
        }
    }

    fn compile_class(&mut self) {
        self.completed_tokens.push(Token::unterminal("class", true));

        self.step("class"); // class
        self.class.name = self.step_identifier(); // className
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

        self.completed_tokens
            .push(Token::unterminal("class", false));
    }

    fn compile_class_var_dec(&mut self) {
        self.completed_tokens
            .push(Token::unterminal("classVarDec", true));

        // static | field
        let token = self.tokens.next();
        let scope = if token.value == "static" {
            ClassScope::Static
        } else if token.value == "field" {
            ClassScope::Field
        } else {
            panic!("invalid: {}, should be `static | field`", token.value);
        };
        self.completed_tokens.push(token);

        // type
        let type_ = self.step_type();

        // varName*
        let mut names = vec![];
        names.push(self.step_identifier()); // varName
        while self.tokens.peek().value.as_str() == "," {
            self.step(","); // ,
            names.push(self.step_identifier()); // varName
        }

        // ;
        self.step(";");

        self.class.vars.push(ClassVarDec {
            kind: scope,
            type_,
            names,
        });

        self.completed_tokens
            .push(Token::unterminal("classVarDec", false));
    }

    fn compile_subroutine(&mut self) {
        self.completed_tokens
            .push(Token::unterminal("subroutineDec", true));

        // constructor | function | method
        let token = self.tokens.next();
        let kind = match token.value.as_str() {
            "constructor" => SubroutineKind::Constructor,
            "function" => SubroutineKind::Function,
            "method" => SubroutineKind::Method,
            _ => panic!(
                "invalid: {}, should be `constructor | function | method`",
                token.value
            ),
        };
        self.completed_tokens.push(token);

        // void | type
        let token = self.tokens.next();
        let type_ = match token.value.as_str() {
            "void" => SubroutineType::Void,
            "int" => SubroutineType::Type(VarType::Int),
            "char" => SubroutineType::Type(VarType::Char),
            "boolean" => SubroutineType::Type(VarType::Boolean),
            _ => SubroutineType::Type(VarType::Class(token.value.clone())),
        };
        self.completed_tokens.push(token);

        let name = self.step_identifier(); // subroutineName
        self.step("("); // (
        let params = self.compile_param_list();
        self.step(")"); // )

        self.completed_tokens
            .push(Token::unterminal("subroutineBody", true));

        self.step("{"); // {
        let mut vars = vec![];
        while self.tokens.peek().value.as_str() == "var" {
            vars.push(self.compile_var_dec());
        }
        let statements = self.compile_statements();
        self.step("}"); // }

        self.class.subroutines.push(SubroutineDec {
            kind,
            type_,
            name,
            params,
            body: SubroutineBody {
                local_vars: vars,
                body:       statements,
            },
        });

        self.completed_tokens
            .push(Token::unterminal("subroutineBody", false));
        self.completed_tokens
            .push(Token::unterminal("subroutineDec", false));
    }

    fn compile_param_list(&mut self) -> Vec<Param> {
        self.completed_tokens
            .push(Token::unterminal("parameterList", true));

        let mut params = vec![];

        if self.tokens.peek().value.as_str() != ")" {
            let type_ = self.step_type(); // type
            let name = self.step_identifier(); // varName
            params.push(Param {
                name,
                var_type: type_,
            });
        }
        while self.tokens.peek().value.as_str() == "," {
            self.step(","); // ,
            let type_ = self.step_type(); // type
            let name = self.step_identifier(); // varName
            params.push(Param {
                name,
                var_type: type_,
            });
        }

        self.completed_tokens
            .push(Token::unterminal("parameterList", false));

        params
    }

    fn compile_var_dec(&mut self) -> VarDec {
        self.completed_tokens
            .push(Token::unterminal("varDec", true));

        let mut names = vec![];

        self.step("var"); // var
        let type_ = self.step_type(); // type
        names.push(self.step_identifier()); // varName
        while self.tokens.peek().value.as_str() == "," {
            self.step(","); // ,
            names.push(self.step_identifier()); // varName
        }
        self.step(";"); // ;

        self.completed_tokens
            .push(Token::unterminal("varDec", false));

        VarDec { type_, names }
    }

    fn compile_statements(&mut self) -> Vec<Statement> {
        self.completed_tokens
            .push(Token::unterminal("statements", true));

        let mut statements = vec![];

        while STATEMENTS
            .get()
            .unwrap()
            .contains(self.tokens.peek().value.as_str())
        {
            let v = self.tokens.peek().value;
            match v.as_str() {
                "if" => statements.push(self.compile_if()),
                "let" => statements.push(self.compile_let()),
                "while" => statements.push(self.compile_while()),
                "do" => statements.push(self.compile_do()),
                "return" => statements.push(self.compile_return()),
                _ => unreachable!(),
            }
        }

        self.completed_tokens
            .push(Token::unterminal("statements", false));

        statements
    }

    fn compile_if(&mut self) -> Statement {
        self.completed_tokens
            .push(Token::unterminal("ifStatement", true));

        self.step("if"); // if
        self.step("("); // (
        let cond = self.compile_expression();
        self.step(")"); // )

        self.step("{"); // {
        let if_body = self.compile_statements();
        self.step("}"); // }

        let else_body = if self.tokens.peek().value.as_str() == "else" {
            self.step("else"); // else
            self.step("{"); // {
            let else_body = self.compile_statements();
            self.step("}"); // }
            Some(else_body)
        } else {
            None
        };

        self.completed_tokens
            .push(Token::unterminal("ifStatement", false));

        Statement::If(IfStatement {
            cond,
            if_body,
            else_body,
        })
    }

    fn compile_let(&mut self) -> Statement {
        self.completed_tokens
            .push(Token::unterminal("letStatement", true));

        self.step("let"); // let
        let var_name = self.step_identifier(); // varName

        let array_index = if self.tokens.peek().value.as_str() == "[" {
            self.step("["); // [
            let expression = self.compile_expression();
            self.step("]"); // ]
            Some(expression)
        } else {
            None
        };

        self.step("="); // =
        let right_expr = self.compile_expression();
        self.step(";"); // ;

        self.completed_tokens
            .push(Token::unterminal("letStatement", false));

        Statement::Let(LetStatement {
            var_name,
            array_index,
            right_expr,
        })
    }

    fn compile_while(&mut self) -> Statement {
        self.completed_tokens
            .push(Token::unterminal("whileStatement", true));

        self.step("while"); // while
        self.step("("); // (
        let cond = self.compile_expression();
        self.step(")"); // )

        self.step("{"); // {
        let body = self.compile_statements();
        self.step("}"); // }

        self.completed_tokens
            .push(Token::unterminal("whileStatement", false));

        Statement::While(WhileStatement { cond, body })
    }

    fn compile_do(&mut self) -> Statement {
        self.completed_tokens
            .push(Token::unterminal("doStatement", true));

        self.step("do"); // do
        let name = self.step_identifier(); // subroutineCall
        let subroutine_call = if self.tokens.peek().value.as_str() == "(" {
            self.step("("); // (
            let args = self.compile_expression_list();
            self.step(")"); // )

            SubroutineCall::Internal(InternalCall {
                name,
                args: Args(args),
            })
        } else if self.tokens.peek().value.as_str() == "." {
            self.step("."); // .
            let subroutine_name = self.step_identifier(); // subroutineName
            self.step("("); // (
            let args = self.compile_expression_list();
            self.step(")"); // )

            SubroutineCall::External(ExternalCall {
                name,
                subroutine_name,
                args: Args(args),
            })
        } else {
            panic!(
                "invalid: {}, line: {}",
                self.tokens.peek().value,
                self.tokens.peek().line
            );
        };
        self.step(";"); // ;

        self.completed_tokens
            .push(Token::unterminal("doStatement", false));

        Statement::Do(DoStatement { subroutine_call })
    }

    fn compile_return(&mut self) -> Statement {
        self.completed_tokens
            .push(Token::unterminal("returnStatement", true));

        self.step("return"); // return

        let expr = if self.tokens.peek().value.as_str() != ";" {
            Some(self.compile_expression())
        } else {
            None
        };

        self.step(";"); // ;

        self.completed_tokens
            .push(Token::unterminal("returnStatement", false));

        Statement::Return(ReturnStatement { expr })
    }

    fn compile_expression(&mut self) -> Expression {
        self.completed_tokens
            .push(Token::unterminal("expression", true));

        let term = self.compile_term();

        let mut op_terms = vec![];
        while OP
            .get()
            .unwrap()
            .contains(&self.tokens.peek().value.chars().next().unwrap())
        {
            // op
            let token = self.tokens.next();
            let op = match token.value.as_str() {
                "+" => Op::Add,
                "-" => Op::Minus,
                "*" => Op::Multiply,
                "/" => Op::Divid,
                "&" => Op::And,
                "|" => Op::Or,
                "<" => Op::Less,
                ">" => Op::Greater,
                "=" => Op::Euqal,
                _ => panic!("invalid: {}, line: {}", token.value, token.line),
            };
            self.completed_tokens.push(token);

            let term = self.compile_term();

            op_terms.push(OpTerm { op, term });
        }

        self.completed_tokens
            .push(Token::unterminal("expression", false));

        Expression {
            term: Box::new(term),
            op_terms,
        }
    }

    fn compile_term(&mut self) -> Term {
        self.completed_tokens.push(Token::unterminal("term", true));

        let term = if self.tokens.peek().value.as_str() == "(" {
            self.step("("); // (
            let expression = self.compile_expression();
            self.step(")"); // )
            Term::Expression(expression)
        } else if UNARY_OP
            .get()
            .unwrap()
            .contains(&self.tokens.peek().value.chars().next().unwrap())
        {
            // - or ~
            let token = self.tokens.next();
            let unary_op = match token.value.as_str() {
                "-" => UnaryOp::Neg,
                "~" => UnaryOp::Not,
                _ => panic!("invalid: {}, line: {}", token.value, token.line),
            };
            self.completed_tokens.push(token);

            let term = self.compile_term();

            Term::UnaryExpression(UnaryExpression {
                unary_op,
                term: Box::new(term),
            })
        } else {
            let token = self.tokens.next();
            self.completed_tokens.push(token.clone());

            let name = token.value.clone();

            match self.tokens.peek().value.as_str() {
                "[" => {
                    self.step("["); // [
                    let expr = self.compile_expression();
                    self.step("]"); // ]

                    Term::Array(Array {
                        name,
                        index: Box::new(expr),
                    })
                }
                "(" => {
                    self.step("("); // (
                    let args = self.compile_expression_list();
                    self.step(")"); // )

                    Term::SubRoutineCall(SubroutineCall::Internal(InternalCall {
                        name,
                        args: Args(args),
                    }))
                }
                "." => {
                    self.step("."); // .
                    let subroutine_name = self.step_identifier(); // subroutineName
                    self.step("("); // (
                    let args = self.compile_expression_list();
                    self.step(")"); // )

                    Term::SubRoutineCall(SubroutineCall::External(ExternalCall {
                        name,
                        subroutine_name,
                        args: Args(args),
                    }))
                }
                _ => match token.category {
                    TokenType::IntegerConstant => {
                        Term::IntegerConst(token.value.parse::<u32>().unwrap())
                    }
                    TokenType::StringConstant => Term::StringConst(token.value),
                    TokenType::Keyword => match token.value.as_str() {
                        "false" => Term::KeywordConst(KeywordConstant::False),
                        "true" => Term::KeywordConst(KeywordConstant::True),
                        "null" => Term::KeywordConst(KeywordConstant::Null),
                        "this" => Term::KeywordConst(KeywordConstant::This),
                        _ => panic!(
                            "invalid: {}, line: {}",
                            self.tokens.peek().value,
                            self.tokens.peek().line
                        ),
                    },
                    TokenType::Identifier => Term::VarName(token.value),
                    _ => panic!(
                        "invalid: {}, line: {}",
                        self.tokens.peek().value,
                        self.tokens.peek().line
                    ),
                },
            }
        };

        self.completed_tokens.push(Token::unterminal("term", false));

        term
    }

    fn compile_expression_list(&mut self) -> Vec<Expression> {
        self.completed_tokens
            .push(Token::unterminal("expressionList", true));

        let mut expressions = vec![];

        if self.tokens.peek().value.as_str() != ")" {
            expressions.push(self.compile_expression());
            while self.tokens.peek().value.as_str() == "," {
                self.step(","); // ,
                expressions.push(self.compile_expression());
            }
        }

        self.completed_tokens
            .push(Token::unterminal("expressionList", false));

        expressions
    }

    pub fn save_file(&self, dst_path: &PathBuf) {
        assert_eq!(dst_path.extension().unwrap(), "xml");

        let mut output: Vec<u8> = vec![];

        for token in self.completed_tokens.iter() {
            writeln!(&mut output, "{}", token.form).unwrap();
        }

        save_file(&output, dst_path).unwrap();
    }
}

pub struct TokenStream {
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

#[derive(Default)]
pub struct Class {
    pub name:        String,
    pub vars:        Vec<ClassVarDec>,
    pub subroutines: Vec<SubroutineDec>,
}

pub struct ClassVarDec {
    pub kind:  ClassScope,
    pub type_: VarType,
    pub names: Vec<String>,
}

#[derive(Clone)]
pub enum ClassScope {
    Static,
    Field,
}

impl From<ClassScope> for VarKind {
    fn from(val: ClassScope) -> Self {
        match val {
            ClassScope::Field => VarKind::Field,
            ClassScope::Static => VarKind::Static,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VarType {
    Int,
    Char,
    Boolean,
    Class(String),
}

pub struct SubroutineDec {
    pub kind:   SubroutineKind,
    pub type_:  SubroutineType,
    pub name:   String,
    pub params: Vec<Param>,
    pub body:   SubroutineBody,
}

#[derive(Clone, PartialEq, Eq)]
pub enum SubroutineKind {
    Constructor,
    Function,
    Method,
}

pub enum SubroutineType {
    Void,
    Type(VarType),
}

pub struct Param {
    pub var_type: VarType,
    pub name:     String,
}

pub struct SubroutineBody {
    pub local_vars: Vec<VarDec>,
    pub body:       Vec<Statement>,
}

pub struct VarDec {
    pub type_: VarType,
    pub names: Vec<String>,
}

pub enum Statement {
    Let(LetStatement),
    If(IfStatement),
    While(WhileStatement),
    Do(DoStatement),
    Return(ReturnStatement),
}

pub struct LetStatement {
    pub var_name:    String,
    pub array_index: Option<Expression>,
    pub right_expr:  Expression,
}

pub struct IfStatement {
    pub cond:      Expression,
    pub if_body:   Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
}

pub struct WhileStatement {
    pub cond: Expression,
    pub body: Vec<Statement>,
}

pub struct DoStatement {
    pub subroutine_call: SubroutineCall,
}

pub struct ReturnStatement {
    pub expr: Option<Expression>,
}

pub struct Expression {
    pub term:     Box<Term>,
    pub op_terms: Vec<OpTerm>,
}

pub struct OpTerm {
    pub op:   Op,
    pub term: Term,
}

pub enum Term {
    IntegerConst(u32),
    StringConst(String),
    KeywordConst(KeywordConstant),
    VarName(String),
    Array(Array),
    SubRoutineCall(SubroutineCall),
    Expression(Expression),
    UnaryExpression(UnaryExpression),
}

pub struct Array {
    pub name:  String,
    pub index: Box<Expression>,
}

pub enum SubroutineCall {
    Internal(InternalCall),
    External(ExternalCall),
}

pub struct InternalCall {
    pub name: String,
    pub args: Args,
}

pub struct ExternalCall {
    pub name:            String,
    pub subroutine_name: String,
    pub args:            Args,
}

pub struct Args(pub Vec<Expression>);

pub struct UnaryExpression {
    pub unary_op: UnaryOp,
    pub term:     Box<Term>,
}

pub enum Op {
    Add,
    Minus,
    Multiply,
    Divid,
    And,
    Or,
    Greater,
    Less,
    Euqal,
}

pub enum UnaryOp {
    Neg,
    Not,
}

pub enum KeywordConstant {
    True,
    False,
    Null,
    This,
}

impl fmt::Display for KeywordConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeywordConstant::True => write!(f, "true"),
            KeywordConstant::False => write!(f, "false"),
            KeywordConstant::Null => write!(f, "null"),
            KeywordConstant::This => write!(f, "this"),
        }
    }
}
