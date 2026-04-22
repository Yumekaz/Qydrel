//! Abstract Syntax Tree (AST) definitions for MiniLang.
//!
//! All AST nodes are immutable and use arena allocation in the parser
//! for cache-friendly traversal.

use crate::token::Span;

/// Type enumeration for the MiniLang type system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Void,
    Error,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Expression nodes
#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral {
        value: i32,
        span: Span,
    },
    BoolLiteral {
        value: bool,
        span: Span,
    },
    Identifier {
        name: String,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    Call {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    ArrayIndex {
        array_name: String,
        index: Box<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::IntLiteral { span, .. } => *span,
            Expr::BoolLiteral { span, .. } => *span,
            Expr::Identifier { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::ArrayIndex { span, .. } => *span,
        }
    }
}

/// Statement nodes
#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl {
        var_type: Type,
        name: String,
        init_expr: Option<Expr>,
        array_size: Option<i32>,
        span: Span,
    },
    Assign {
        target: String,
        index_expr: Option<Expr>,
        value: Expr,
        span: Span,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Return {
        value: Expr,
        span: Span,
    },
    Print {
        value: Expr,
        span: Span,
    },
    ExprStmt {
        expr: Expr,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDecl { span, .. } => *span,
            Stmt::Assign { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::While { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
            Stmt::Print { span, .. } => *span,
            Stmt::ExprStmt { span, .. } => *span,
        }
    }
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub param_type: Type,
    pub name: String,
    pub span: Span,
}

/// Function declaration
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Global variable declaration
#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub var_type: Type,
    pub name: String,
    pub init_expr: Option<Expr>,
    pub array_size: Option<i32>,
    pub span: Span,
}

/// Program (root AST node)
#[derive(Debug, Clone)]
pub struct Program {
    pub globals: Vec<GlobalVar>,
    pub functions: Vec<Function>,
}
