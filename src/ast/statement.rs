use std::fmt::Display;

use super::expr::Expr;
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
            Statement::Expr(expr) => write!(f, "{expr}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_declaration() {
        let statement = Statement::declare("x", Expr::literal_number(1));
        assert_eq!(statement.to_string(), "let x = 1");
    }

    #[test]
    fn test_display_typed_declaration() {
        let statement =
            Statement::declare_typed("x", TypeExpr::name("Number"), Expr::literal_number(1));
        assert_eq!(statement.to_string(), "let x: Number = 1");
    }

    #[test]
    fn test_display_assignment() {
        let statement = Statement::assign("x", Expr::literal_number(1));
        assert_eq!(statement.to_string(), "x = 1");
    }
}
