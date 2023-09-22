use std::fmt;

use crate::symbol_table::VarKind;

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
