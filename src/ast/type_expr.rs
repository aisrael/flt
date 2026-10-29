use std::fmt::Display;

/// A type written in flt source, e.g. `Number | None` in `let x: Number | None = None`. It holds
/// names, which a runtime resolves to types, since only the runtime knows its native types.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeExpr {
    /// A type's name, e.g. `Number`, `None` or a native type's name
    Name(String),
    /// A union of types, `A | B`. A member that's itself a union was written in parentheses.
    Union(Vec<TypeExpr>),
}

impl TypeExpr {
    /// A type's name, e.g. `TypeExpr::name("Number")`
    pub fn name<S: Into<String>>(name: S) -> Self {
        TypeExpr::Name(name.into())
    }
}

impl Display for TypeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeExpr::Name(name) => write!(f, "{name}"),
            TypeExpr::Union(members) => {
                for (i, member) in members.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    match member {
                        TypeExpr::Union(_) => write!(f, "({member})")?,
                        TypeExpr::Name(_) => write!(f, "{member}")?,
                    }
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name() {
        assert_eq!(TypeExpr::name("Number").to_string(), "Number");
    }

    #[test]
    fn test_display_union() {
        let union = TypeExpr::Union(vec![TypeExpr::name("Number"), TypeExpr::name("None")]);
        assert_eq!(union.to_string(), "Number | None");
    }

    #[test]
    fn test_display_brackets_a_nested_union() {
        let union = TypeExpr::Union(vec![
            TypeExpr::Union(vec![TypeExpr::name("Number"), TypeExpr::name("String")]),
            TypeExpr::name("None"),
        ]);
        assert_eq!(union.to_string(), "(Number | String) | None");
    }
}
