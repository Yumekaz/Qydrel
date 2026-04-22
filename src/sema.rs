//! Semantic analyzer for MiniLang.
//!
//! Performs type checking, scope analysis, and validation.

use crate::ast::*;
use crate::limits;
use crate::token::Span;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Semantic error at {span}: {message}")]
    Error { message: String, span: Span },
}

pub type SemanticResult<T> = Result<T, SemanticError>;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub sym_type: Type,
    pub is_global: bool,
    pub is_array: bool,
    pub array_size: i32,
    pub is_function: bool,
    pub param_types: Vec<Type>,
}

#[derive(Debug)]
struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    fn new(parent: Option<Box<Scope>>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
        }
    }

    fn define(&mut self, symbol: Symbol) -> bool {
        if self.symbols.contains_key(&symbol.name) {
            return false;
        }
        self.symbols.insert(symbol.name.clone(), symbol);
        true
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        if let Some(sym) = self.symbols.get(name) {
            return Some(sym);
        }
        if let Some(ref parent) = self.parent {
            return parent.lookup(name);
        }
        None
    }
}

pub struct SemanticAnalyzer {
    scope: Scope,
    functions: HashMap<String, Symbol>,
    errors: Vec<SemanticError>,
    current_function: Option<String>,
    current_local_names: HashSet<String>,
    current_local_slots: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(None),
            functions: HashMap::new(),
            errors: Vec::new(),
            current_function: None,
            current_local_names: HashSet::new(),
            current_local_slots: 0,
        }
    }

    fn error(&mut self, message: &str, span: Span) {
        self.errors.push(SemanticError::Error {
            message: message.to_string(),
            span,
        });
    }

    fn push_scope(&mut self) {
        let old_scope = std::mem::replace(&mut self.scope, Scope::new(None));
        self.scope = Scope::new(Some(Box::new(old_scope)));
    }

    fn pop_scope(&mut self) {
        if let Some(parent) = self.scope.parent.take() {
            self.scope = *parent;
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        // First pass: collect all function signatures and globals
        self.collect_declarations(program);

        // Check main exists
        if !self.functions.contains_key("main") {
            self.error("Program must define a 'main' function", Span::default());
        } else {
            let main_func = &self.functions["main"];
            if !main_func.param_types.is_empty() {
                self.error("'main' function must take no parameters", Span::default());
            }
        }

        // Second pass: analyze function bodies
        for func in &program.functions {
            self.analyze_function(func);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn collect_declarations(&mut self, program: &Program) {
        let mut global_slots = 0usize;

        // Collect globals
        for glob in &program.globals {
            let checked_array_size =
                self.validate_array_size(&glob.name, glob.array_size, glob.span);
            let slot_count = if glob.array_size.is_some() {
                checked_array_size.unwrap_or(0)
            } else {
                1
            };
            self.add_global_slots(&glob.name, slot_count, glob.span, &mut global_slots);

            let sym = Symbol {
                name: glob.name.clone(),
                sym_type: glob.var_type,
                is_global: true,
                is_array: glob.array_size.is_some(),
                array_size: checked_array_size.map(|size| size as i32).unwrap_or(0),
                is_function: false,
                param_types: vec![],
            };
            if !self.scope.define(sym) {
                self.error(
                    &format!("Duplicate global variable: {}", glob.name),
                    glob.span,
                );
            }
        }

        // Collect functions
        for func in &program.functions {
            let param_types: Vec<Type> = func.params.iter().map(|p| p.param_type).collect();
            let sym = Symbol {
                name: func.name.clone(),
                sym_type: Type::Int, // All functions return int
                is_global: false,
                is_array: false,
                array_size: 0,
                is_function: true,
                param_types,
            };
            if self.functions.contains_key(&func.name) {
                self.error(&format!("Duplicate function: {}", func.name), func.span);
            } else {
                self.functions.insert(func.name.clone(), sym.clone());
                if !self.scope.define(sym) {
                    self.error(
                        &format!("Duplicate top-level symbol: {}", func.name),
                        func.span,
                    );
                }
            }
        }

        // Type check global initializers after collecting all top-level names.
        for glob in &program.globals {
            if let Some(ref init_expr) = glob.init_expr {
                let init_type = self.analyze_expr(init_expr);
                if init_type != glob.var_type && init_type != Type::Error {
                    self.error(
                        &format!(
                            "Type mismatch: cannot assign {:?} to {:?}",
                            init_type, glob.var_type
                        ),
                        glob.span,
                    );
                }
            }
        }
    }

    fn analyze_function(&mut self, func: &Function) {
        self.current_function = Some(func.name.clone());
        self.current_local_names.clear();
        self.current_local_names
            .extend(func.params.iter().map(|param| param.name.clone()));
        self.current_local_slots = func.params.len();
        self.check_local_slot_limit(func.span);

        self.push_scope();

        // Add parameters to scope
        for param in &func.params {
            if self.scope.lookup(&param.name).is_some() {
                self.error(
                    &format!("Parameter shadows existing symbol: {}", param.name),
                    param.span,
                );
                continue;
            }

            let sym = Symbol {
                name: param.name.clone(),
                sym_type: param.param_type,
                is_global: false,
                is_array: false,
                array_size: 0,
                is_function: false,
                param_types: vec![],
            };
            if !self.scope.define(sym) {
                self.error(&format!("Duplicate parameter: {}", param.name), param.span);
            }
        }

        // Analyze body
        for stmt in &func.body {
            self.analyze_stmt(stmt);
        }

        self.pop_scope();
        self.current_function = None;
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                var_type,
                name,
                init_expr,
                array_size,
                span,
            } => {
                if self.scope.lookup(name).is_some() {
                    self.error(
                        &format!("Variable shadows existing symbol: {}", name),
                        *span,
                    );
                    return;
                }

                let checked_array_size = self.validate_array_size(name, *array_size, *span);
                self.add_local_slot(name, *span);

                let sym = Symbol {
                    name: name.clone(),
                    sym_type: *var_type,
                    is_global: false,
                    is_array: array_size.is_some(),
                    array_size: checked_array_size.map(|size| size as i32).unwrap_or(0),
                    is_function: false,
                    param_types: vec![],
                };
                if !self.scope.define(sym) {
                    self.error(&format!("Duplicate variable: {}", name), *span);
                }

                if let Some(init) = init_expr {
                    let init_type = self.analyze_expr(init);
                    if init_type != *var_type && init_type != Type::Error {
                        self.error(
                            &format!(
                                "Type mismatch: cannot assign {:?} to {:?}",
                                init_type, var_type
                            ),
                            *span,
                        );
                    }
                }
            }

            Stmt::Assign {
                target,
                index_expr,
                value,
                span,
            } => {
                let Some(sym) = self.scope.lookup(target).cloned() else {
                    self.error(&format!("Undefined variable: {}", target), *span);
                    return;
                };

                if sym.is_function {
                    self.error(&format!("Cannot assign to function: {}", target), *span);
                    return;
                }

                if let Some(idx) = index_expr {
                    let index_type = self.analyze_expr(idx);
                    if index_type != Type::Int && index_type != Type::Error {
                        self.error("Array index must be an integer", *span);
                    }
                    if !sym.is_array {
                        self.error(&format!("Cannot index non-array: {}", target), *span);
                    }
                } else if sym.is_array {
                    self.error(
                        &format!("Cannot assign to array without index: {}", target),
                        *span,
                    );
                    return;
                }

                let value_type = self.analyze_expr(value);
                if value_type != sym.sym_type && value_type != Type::Error {
                    self.error(
                        &format!(
                            "Type mismatch: cannot assign {:?} to {:?}",
                            value_type, sym.sym_type
                        ),
                        *span,
                    );
                }
            }

            Stmt::If {
                condition,
                then_body,
                else_body,
                span,
            } => {
                let cond_type = self.analyze_expr(condition);
                if !matches!(cond_type, Type::Int | Type::Bool | Type::Error) {
                    self.error("Condition must be int or bool", *span);
                }

                self.push_scope();
                for s in then_body {
                    self.analyze_stmt(s);
                }
                self.pop_scope();

                if let Some(else_stmts) = else_body {
                    self.push_scope();
                    for s in else_stmts {
                        self.analyze_stmt(s);
                    }
                    self.pop_scope();
                }
            }

            Stmt::While {
                condition,
                body,
                span,
            } => {
                let cond_type = self.analyze_expr(condition);
                if !matches!(cond_type, Type::Int | Type::Bool | Type::Error) {
                    self.error("Condition must be int or bool", *span);
                }

                self.push_scope();
                for s in body {
                    self.analyze_stmt(s);
                }
                self.pop_scope();
            }

            Stmt::Return { value, span } => {
                let value_type = self.analyze_expr(value);
                if value_type != Type::Int && value_type != Type::Error {
                    self.error("Return value must be int", *span);
                }
            }

            Stmt::Print { value, .. } => {
                self.analyze_expr(value);
            }

            Stmt::ExprStmt { expr, .. } => {
                self.analyze_expr(expr);
            }
        }
    }

    fn validate_array_size(
        &mut self,
        name: &str,
        array_size: Option<i32>,
        span: Span,
    ) -> Option<usize> {
        match array_size {
            Some(size) if size > 0 => Some(size as usize),
            Some(_) => {
                self.error(&format!("Array size for '{}' must be positive", name), span);
                None
            }
            None => None,
        }
    }

    fn add_global_slots(&mut self, name: &str, slots: usize, span: Span, global_slots: &mut usize) {
        if slots == 0 {
            return;
        }

        let Some(next_slots) = global_slots.checked_add(slots) else {
            self.error("Global storage size overflow", span);
            return;
        };

        if next_slots > limits::MAX_GLOBAL_SLOTS {
            self.error(
                &format!(
                    "Global storage exceeds {} slots at '{}' (needs {})",
                    limits::MAX_GLOBAL_SLOTS,
                    name,
                    next_slots
                ),
                span,
            );
        }

        *global_slots = next_slots;
    }

    fn add_local_slot(&mut self, name: &str, span: Span) {
        if self.current_local_names.insert(name.to_string()) {
            self.current_local_slots = self.current_local_slots.saturating_add(1);
            self.check_local_slot_limit(span);
        }
    }

    fn check_local_slot_limit(&mut self, span: Span) {
        if self.current_local_slots > limits::MAX_LOCAL_SLOTS {
            let function_name = self.current_function.as_deref().unwrap_or("<unknown>");
            self.error(
                &format!(
                    "Local storage exceeds {} slots in function '{}' (needs {})",
                    limits::MAX_LOCAL_SLOTS,
                    function_name,
                    self.current_local_slots
                ),
                span,
            );
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::IntLiteral { .. } => Type::Int,
            Expr::BoolLiteral { .. } => Type::Bool,

            Expr::Identifier { name, span } => {
                let Some(sym) = self.scope.lookup(name) else {
                    self.error(&format!("Undefined variable: {}", name), *span);
                    return Type::Error;
                };
                if sym.is_function {
                    self.error(&format!("Cannot use function as value: {}", name), *span);
                    return Type::Error;
                }
                if sym.is_array {
                    self.error(&format!("Cannot use array without index: {}", name), *span);
                    return Type::Error;
                }
                sym.sym_type
            }

            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                let left_type = self.analyze_expr(left);
                let right_type = self.analyze_expr(right);

                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        if left_type != Type::Int && left_type != Type::Error {
                            self.error("Arithmetic operator requires int operands", *span);
                        }
                        if right_type != Type::Int && right_type != Type::Error {
                            self.error("Arithmetic operator requires int operands", *span);
                        }
                        Type::Int
                    }

                    BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                        if left_type != Type::Int && left_type != Type::Error {
                            self.error("Comparison operator requires int operands", *span);
                        }
                        if right_type != Type::Int && right_type != Type::Error {
                            self.error("Comparison operator requires int operands", *span);
                        }
                        Type::Bool
                    }

                    BinaryOp::Eq | BinaryOp::Ne => {
                        if left_type != right_type
                            && left_type != Type::Error
                            && right_type != Type::Error
                        {
                            self.error("Equality operator requires same types", *span);
                        }
                        Type::Bool
                    }

                    BinaryOp::And | BinaryOp::Or => {
                        if !matches!(left_type, Type::Int | Type::Bool | Type::Error) {
                            self.error("Logical operator requires int or bool", *span);
                        }
                        if !matches!(right_type, Type::Int | Type::Bool | Type::Error) {
                            self.error("Logical operator requires int or bool", *span);
                        }
                        Type::Bool
                    }
                }
            }

            Expr::Unary { op, operand, span } => {
                let operand_type = self.analyze_expr(operand);

                match op {
                    UnaryOp::Neg => {
                        if operand_type != Type::Int && operand_type != Type::Error {
                            self.error("Negation requires int operand", *span);
                        }
                        Type::Int
                    }
                    UnaryOp::Not => {
                        if !matches!(operand_type, Type::Int | Type::Bool | Type::Error) {
                            self.error("Logical not requires int or bool", *span);
                        }
                        Type::Bool
                    }
                }
            }

            Expr::Call { name, args, span } => {
                let Some(func) = self.functions.get(name).cloned() else {
                    self.error(&format!("Undefined function: {}", name), *span);
                    return Type::Error;
                };

                if args.len() != func.param_types.len() {
                    self.error(
                        &format!(
                            "Wrong number of arguments: expected {}, got {}",
                            func.param_types.len(),
                            args.len()
                        ),
                        *span,
                    );
                    return Type::Int;
                }

                for (i, (arg, expected_type)) in args.iter().zip(&func.param_types).enumerate() {
                    let arg_type = self.analyze_expr(arg);
                    if arg_type != *expected_type && arg_type != Type::Error {
                        self.error(
                            &format!(
                                "Argument {} type mismatch: expected {:?}, got {:?}",
                                i + 1,
                                expected_type,
                                arg_type
                            ),
                            *span,
                        );
                    }
                }

                Type::Int
            }

            Expr::ArrayIndex {
                array_name,
                index,
                span,
            } => {
                let Some(sym) = self.scope.lookup(array_name).cloned() else {
                    self.error(&format!("Undefined variable: {}", array_name), *span);
                    return Type::Error;
                };

                if !sym.is_array {
                    self.error(&format!("Cannot index non-array: {}", array_name), *span);
                    return Type::Error;
                }

                let index_type = self.analyze_expr(index);
                if index_type != Type::Int && index_type != Type::Error {
                    self.error("Array index must be int", *span);
                }

                sym.sym_type
            }
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
