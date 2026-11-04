use std::fmt::Display;

use super::expr::Expr;
use super::function::FunctionDeclaration;
use super::identifier::Identifier;
use super::type_expr::TypeExpr;

/// A statement in the language.
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    /// A declaration: `let name = expr`, or `let name: type = expr` with a declared type.
    Let {
        name: Identifier,
        r#type: Option<TypeExpr>,
        expr: Expr,
    },
    /// An assignment: `name = expr`.
    Assign(Identifier, Expr),
    /// A function declaration: `def name(params) -> type … end`.
    Def(FunctionDeclaration),
    /// A return: `return` or `return expr`. It stops the call of the enclosing function, and
    /// gives `expr`, or `None` without one.
    Return(Option<Expr>),
    /// A bare expression used as a statement.
    Expr(Expr),
}

impl Statement {
    /// A declaration without a type: `let name = expr`.
    pub fn declare<S: Into<String>>(name: S, expr: Expr) -> Self {
        Statement::Let {
            name: Identifier(name.into()),
            r#type: None,
            expr,
        }
    }

    /// A declaration with a type: `let name: type = expr`.
    pub fn declare_typed<S: Into<String>>(name: S, r#type: TypeExpr, expr: Expr) -> Self {
        Statement::Let {
            name: Identifier(name.into()),
            r#type: Some(r#type),
            expr,
        }
    }

    /// An assignment: `name = expr`.
    pub fn assign<S: Into<String>>(name: S, expr: Expr) -> Self {
        Statement::Assign(Identifier(name.into()), expr)
    }
}

impl Statement {
    /// A return with an expression: `return expr`.
    pub fn return_value(expr: Expr) -> Self {
        Statement::Return(Some(expr))
    }

    /// Whether this statement is, or holds, a `return` that would leave the function around
    /// it. The body of a `def` is not searched: a `return` there leaves that function.
    pub fn contains_return(&self) -> bool {
        match self {
            Statement::Return(_) => true,
            Statement::Let { expr, .. } | Statement::Assign(_, expr) | Statement::Expr(expr) => {
                expr.contains_return()
            }
            Statement::Def(_) => false,
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let {
                name,
                r#type: Some(r#type),
                expr,
            } => write!(f, "let {name}: {type} = {expr}"),
            Statement::Let {
                name,
                r#type: None,
                expr,
            } => write!(f, "let {name} = {expr}"),
            Statement::Assign(name, expr) => write!(f, "{name} = {expr}"),
            Statement::Def(decl) => write!(f, "{decl}"),
            Statement::Return(Some(expr)) => write!(f, "return {expr}"),
            Statement::Return(None) => write!(f, "return"),
            Statement::Expr(expr) => write!(f, "{expr}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_declaration() {
        let statement = Statement::declare("x", Expr::literal_integer(1));
        assert_eq!(statement.to_string(), "let x = 1");
    }

    #[test]
    fn test_display_typed_declaration() {
        let statement =
            Statement::declare_typed("x", TypeExpr::name("Number"), Expr::literal_integer(1));
        assert_eq!(statement.to_string(), "let x: Number = 1");
    }

    #[test]
    fn test_display_return() {
        let statement = Statement::return_value(Expr::literal_integer(1));
        assert_eq!(statement.to_string(), "return 1");
        assert_eq!(Statement::Return(None).to_string(), "return");
    }

    #[test]
    fn test_contains_return() {
        assert!(Statement::Return(None).contains_return());
        assert!(!Statement::declare("x", Expr::literal_integer(1)).contains_return());
        let block = Expr::Block(vec![Statement::Return(None)]);
        assert!(Statement::Expr(block.clone()).contains_return());
        assert!(Statement::declare("x", block.clone()).contains_return());
        let nested = Expr::if_expr(Expr::literal_boolean(true), block, None);
        assert!(Statement::Expr(nested).contains_return());
    }

    #[test]
    fn test_display_assignment() {
        let statement = Statement::assign("x", Expr::literal_integer(1));
        assert_eq!(statement.to_string(), "x = 1");
    }
}
