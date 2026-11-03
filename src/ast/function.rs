//! AST types for function calls.

use std::fmt::Display;

use indexmap::IndexMap;

use super::expr::Expr;
use super::identifier::Identifier;
use super::statement::Statement;
use super::type_expr::TypeExpr;
use crate::utils::escape_string;
use crate::utils::is_bare_name;

/// A call's keyword arguments, keyed by their unquoted text, in source order.
pub type KeywordArgs = IndexMap<String, Expr>;

/// A function call: name, positional arguments, then optional keyword arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    /// Positional arguments (must come first).
    pub positional_args: Vec<Expr>,
    /// Trailing keyword arguments (e.g. `foo(1, bar: true)`).
    pub keyword_args: KeywordArgs,
}

/// Displays the call as `name(p1, p2, k: v)`.
impl Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        write!(f, "(")?;
        for (i, arg) in self.positional_args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            arg.fmt(f)?;
        }
        for (i, (key, value)) in self.keyword_args.iter().enumerate() {
            if i > 0 || !self.positional_args.is_empty() {
                write!(f, ", ")?;
            }
            if is_bare_name(key) {
                write!(f, "{key}: {value}")?;
            } else {
                write!(f, "\"{}\": {value}", escape_string(key))?;
            }
        }
        write!(f, ")")
    }
}

/// The kind of a parameter in a function declaration
#[derive(Clone, Debug, PartialEq)]
pub enum ParameterKind {
    /// `name: Type`, filled by the position of an argument
    Positional,
    /// `*name: Type`, which takes zero or more trailing positional arguments
    Variadic,
    /// `name: Type = default`, filled by a keyword argument, or else by the default
    Keyword(Expr),
    /// `**name: Type`, which takes any other keyword arguments
    RestKeywords,
}

/// A parameter in a function declaration. Every parameter has a type.
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub name: Identifier,
    pub kind: ParameterKind,
    pub r#type: TypeExpr,
}

impl Parameter {
    /// A positional parameter, `name: type`
    pub fn positional<S: Into<String>>(name: S, r#type: TypeExpr) -> Self {
        Self::new(name, ParameterKind::Positional, r#type)
    }

    /// A variadic parameter, `*name: type`
    pub fn variadic<S: Into<String>>(name: S, r#type: TypeExpr) -> Self {
        Self::new(name, ParameterKind::Variadic, r#type)
    }

    /// A keyword parameter, `name: type = default`
    pub fn keyword<S: Into<String>>(name: S, r#type: TypeExpr, default: Expr) -> Self {
        Self::new(name, ParameterKind::Keyword(default), r#type)
    }

    /// A rest keywords parameter, `**name: type`
    pub fn rest_keywords<S: Into<String>>(name: S, r#type: TypeExpr) -> Self {
        Self::new(name, ParameterKind::RestKeywords, r#type)
    }

    fn new<S: Into<String>>(name: S, kind: ParameterKind, r#type: TypeExpr) -> Self {
        Self {
            name: Identifier(name.into()),
            kind,
            r#type,
        }
    }

    /// The position of the kind of parameter in a declaration: positional parameters come
    /// first, then the variadic one, then keywords, then the rest keywords.
    pub fn order(&self) -> u8 {
        match self.kind {
            ParameterKind::Positional => 0,
            ParameterKind::Variadic => 1,
            ParameterKind::Keyword(_) => 2,
            ParameterKind::RestKeywords => 3,
        }
    }
}

/// Displays the parameter as written, e.g. `*values: Any` or `level: Symbol = :info`.
impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ParameterKind::Positional => write!(f, "{}: {}", self.name, self.r#type),
            ParameterKind::Variadic => write!(f, "*{}: {}", self.name, self.r#type),
            ParameterKind::Keyword(default) => {
                write!(f, "{}: {} = {default}", self.name, self.r#type)
            }
            ParameterKind::RestKeywords => write!(f, "**{}: {}", self.name, self.r#type),
        }
    }
}

/// A function declaration: `def name(params) -> type`, a body of statements, and `end`.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDecl {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    /// The return type, or `None` if the declaration has no `->`, which means `-> None`
    pub return_type: Option<TypeExpr>,
    pub body: Vec<Statement>,
}

/// Displays the declaration over several lines, with each statement of the body indented, in
/// a form that parses back to the same declaration.
impl Display for FunctionDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "def {}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{param}")?;
        }
        write!(f, ")")?;
        if let Some(return_type) = &self.return_type {
            write!(f, " -> {return_type}")?;
        }
        for statement in &self.body {
            // A nested declaration's lines are indented as well
            for line in statement.to_string().lines() {
                write!(f, "\n  {line}")?;
            }
        }
        write!(f, "\nend")
    }
}

#[cfg(test)]
mod tests {
    use super::Expr;
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(Expr::function_call("f", vec![]).to_string(), "f()");
        assert_eq!(
            Expr::function_call_with_keywords(
                "f",
                vec![Expr::literal_integer(1)],
                vec![
                    ("k", Expr::literal_integer(2)),
                    ("c d", Expr::literal_integer(3))
                ],
            )
            .to_string(),
            r#"f(1, k: 2, "c d": 3)"#
        );
        assert_eq!(
            Expr::function_call_with_keywords("f", vec![], vec![("k", Expr::literal_integer(2))])
                .to_string(),
            "f(k: 2)"
        );
    }

    #[test]
    fn test_display_declaration() {
        let declaration = FunctionDecl {
            name: Identifier("log".to_string()),
            params: vec![
                Parameter::positional("label", TypeExpr::name("String")),
                Parameter::variadic("values", TypeExpr::name("Any")),
                Parameter::keyword("level", TypeExpr::name("Symbol"), Expr::literal_none()),
                Parameter::rest_keywords("extra", TypeExpr::name("Any")),
            ],
            return_type: Some(TypeExpr::Union(vec![
                TypeExpr::name("Integer"),
                TypeExpr::name("None"),
            ])),
            body: vec![Statement::Expr(Expr::literal_integer(1))],
        };
        assert_eq!(
            declaration.to_string(),
            "def log(label: String, *values: Any, level: Symbol = None, **extra: Any) -> Integer | None\n  1\nend"
        );
    }

    #[test]
    fn test_display_declaration_without_a_body_or_return_type() {
        let declaration = FunctionDecl {
            name: Identifier("nothing".to_string()),
            params: vec![],
            return_type: None,
            body: vec![],
        };
        assert_eq!(declaration.to_string(), "def nothing()\nend");
    }

    #[test]
    fn test_param_order() {
        let t = || TypeExpr::name("Any");
        assert!(Parameter::positional("a", t()).order() < Parameter::variadic("a", t()).order());
        assert!(
            Parameter::variadic("a", t()).order()
                < Parameter::keyword("a", t(), Expr::literal_none()).order()
        );
        assert!(
            Parameter::keyword("a", t(), Expr::literal_none()).order()
                < Parameter::rest_keywords("a", t()).order()
        );
    }
}
