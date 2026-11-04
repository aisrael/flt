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
    /// A constant declaration: `const name = expr`, or `const name: type = expr` with a declared
    /// type ([ADR 025](https://github.com/aisrael/flt/blob/main/docs/adrs/025-const-declarations.md)).
    Const {
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

    /// A constant declaration without a type: `const name = expr`.
    pub fn constant<S: Into<String>>(name: S, expr: Expr) -> Self {
        Statement::Const {
            name: Identifier(name.into()),
            r#type: None,
            expr,
        }
    }

    /// A constant declaration with a type: `const name: type = expr`.
    pub fn constant_typed<S: Into<String>>(name: S, r#type: TypeExpr, expr: Expr) -> Self {
        Statement::Const {
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
            Statement::Let { expr, .. }
            | Statement::Const { expr, .. }
            | Statement::Assign(_, expr)
            | Statement::Expr(expr) => expr.contains_return(),
            Statement::Def(_) => false,
        }
    }
}

/// The number of lists of statements in `statements` that have a statement after a `return`
/// ([ADR 024](https://github.com/aisrael/flt/blob/main/docs/adrs/024-unreachable-code-warning.md)).
///
/// A list is `statements` itself and every brace block and arm of an `if` inside it. The count
/// follows the syntax, not the flow of the program: only a `return` in the same list makes the
/// statements after it unreachable. A list counts once, however many statements follow its
/// `return`, and the unreachable statements are not searched. The body of a `def` is not
/// searched either: it has its own count.
pub fn unreachable_lists(statements: &[Statement]) -> usize {
    match statements
        .iter()
        .position(|statement| matches!(statement, Statement::Return(_)))
    {
        Some(index) => {
            let reachable = &statements[..=index];
            let after = usize::from(index + 1 < statements.len());
            after
                + reachable
                    .iter()
                    .map(Statement::unreachable_lists)
                    .sum::<usize>()
        }
        None => statements.iter().map(Statement::unreachable_lists).sum(),
    }
}

impl Statement {
    /// The number of lists of statements inside this statement that have a statement after a
    /// `return`. See [`unreachable_lists`].
    fn unreachable_lists(&self) -> usize {
        match self {
            Statement::Return(Some(expr))
            | Statement::Let { expr, .. }
            | Statement::Const { expr, .. }
            | Statement::Assign(_, expr)
            | Statement::Expr(expr) => expr.unreachable_lists(),
            Statement::Return(None) | Statement::Def(_) => 0,
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
            Statement::Const {
                name,
                r#type: Some(r#type),
                expr,
            } => write!(f, "const {name}: {type} = {expr}"),
            Statement::Const {
                name,
                r#type: None,
                expr,
            } => write!(f, "const {name} = {expr}"),
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
    fn test_display_constant() {
        let statement = Statement::constant("A", Expr::literal_integer(1));
        assert_eq!(statement.to_string(), "const A = 1");
    }

    #[test]
    fn test_display_typed_constant() {
        let statement =
            Statement::constant_typed("A", TypeExpr::name("Number"), Expr::literal_integer(1));
        assert_eq!(statement.to_string(), "const A: Number = 1");
    }

    #[test]
    fn test_a_constant_is_not_a_declaration() {
        let constant = Statement::constant("A", Expr::literal_integer(1));
        assert_ne!(constant, Statement::declare("A", Expr::literal_integer(1)));
    }

    #[test]
    fn test_contains_return_in_a_constant() {
        let block = Expr::Block(vec![Statement::Return(None)]);
        assert!(Statement::constant("A", block).contains_return());
        assert!(!Statement::constant("A", Expr::literal_integer(1)).contains_return());
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

    fn ret() -> Statement {
        Statement::Return(None)
    }

    fn one() -> Statement {
        Statement::Expr(Expr::literal_integer(1))
    }

    fn block(statements: Vec<Statement>) -> Statement {
        Statement::Expr(Expr::Block(statements))
    }

    #[test]
    fn test_unreachable_lists_of_a_list_without_a_return() {
        assert_eq!(unreachable_lists(&[]), 0);
        assert_eq!(unreachable_lists(&[one(), one()]), 0);
    }

    #[test]
    fn test_unreachable_lists_of_a_return_at_the_end() {
        assert_eq!(unreachable_lists(&[one(), ret()]), 0);
        assert_eq!(unreachable_lists(&[ret()]), 0);
    }

    #[test]
    fn test_unreachable_lists_of_statements_after_a_return() {
        assert_eq!(unreachable_lists(&[ret(), one()]), 1);
        assert_eq!(unreachable_lists(&[ret(), one(), one(), one()]), 1);
        assert_eq!(unreachable_lists(&[one(), ret(), one()]), 1);
    }

    #[test]
    fn test_unreachable_lists_counts_the_first_return_only() {
        assert_eq!(unreachable_lists(&[ret(), ret(), one()]), 1);
    }

    #[test]
    fn test_unreachable_lists_in_blocks_and_arms() {
        assert_eq!(unreachable_lists(&[block(vec![ret(), one()])]), 1);
        assert_eq!(unreachable_lists(&[block(vec![one(), ret()]), one()]), 0);
        let arm = Expr::Block(vec![ret(), one()]);
        let if_expr = Expr::if_expr(Expr::literal_boolean(true), arm, None);
        assert_eq!(unreachable_lists(&[Statement::Expr(if_expr)]), 1);
    }

    #[test]
    fn test_unreachable_lists_in_the_else_arm_and_the_condition() {
        let then_arm = Expr::Block(vec![one()]);
        let else_arm = Expr::Block(vec![ret(), one()]);
        let if_expr = Expr::if_expr(Expr::literal_boolean(true), then_arm, Some(else_arm));
        assert_eq!(unreachable_lists(&[Statement::Expr(if_expr)]), 1);
        let condition = Expr::Block(vec![ret(), one()]);
        let if_expr = Expr::if_expr(condition, Expr::literal_integer(1), None);
        assert_eq!(unreachable_lists(&[Statement::Expr(if_expr)]), 1);
    }

    #[test]
    fn test_unreachable_lists_adds_the_lists_of_a_body_and_its_blocks() {
        let arm = Expr::Block(vec![ret(), one()]);
        let if_expr = Statement::Expr(Expr::if_expr(Expr::literal_boolean(true), arm, None));
        assert_eq!(unreachable_lists(&[if_expr, ret(), one()]), 2);
    }

    #[test]
    fn test_unreachable_lists_does_not_search_unreachable_statements() {
        let dead = block(vec![ret(), one()]);
        assert_eq!(unreachable_lists(&[ret(), dead]), 1);
    }

    #[test]
    fn test_unreachable_lists_searches_the_expression_of_a_return() {
        let value = Expr::Block(vec![ret(), one()]);
        assert_eq!(unreachable_lists(&[Statement::return_value(value)]), 1);
    }

    #[test]
    fn test_unreachable_lists_ignores_a_return_in_another_list() {
        let arm = Expr::Block(vec![ret()]);
        let if_expr = Statement::Expr(Expr::if_expr(Expr::literal_boolean(true), arm, None));
        assert_eq!(unreachable_lists(&[if_expr, one()]), 0);
    }

    #[test]
    fn test_unreachable_lists_in_a_let_and_in_operands() {
        let dead = Expr::Block(vec![ret(), one()]);
        assert_eq!(
            unreachable_lists(&[Statement::declare("x", dead.clone())]),
            1
        );
        let sum = Expr::BinaryExpr(
            Box::new(Expr::literal_integer(1)),
            crate::ast::BinaryOp::Add,
            Box::new(dead),
        );
        assert_eq!(unreachable_lists(&[Statement::Expr(sum)]), 1);
    }

    #[test]
    fn test_display_assignment() {
        let statement = Statement::assign("x", Expr::literal_integer(1));
        assert_eq!(statement.to_string(), "x = 1");
    }
}
