//! Union types: `A | B`, a value of any one of the member types

use std::fmt;

use crate::runtime::types::Type;

impl Type {
    /// The union `a | b`: nested unions are flattened, duplicates are removed (keeping
    /// the first occurrence), and a single remaining member is returned as-is.
    pub fn union(_a: Type, _b: Type) -> Self {
        todo!("Type::union")
    }
}

/// Writes the members of a union separated by ` | `, e.g. `Number | None`.
pub(super) fn fmt_union(_members: &[Type], _f: &mut fmt::Formatter<'_>) -> fmt::Result {
    todo!("Display for Type::Union")
}

#[cfg(test)]
mod tests {
    use crate::runtime::types::Type;

    #[test]
    fn test_union_of_two_distinct_types() {
        let union = Type::union(Type::number(), Type::string());
        assert_eq!(union, Type::Union(vec![Type::number(), Type::string()]));
        assert_eq!(union.to_string(), "Number | String");
    }

    #[test]
    fn test_union_with_none() {
        let union = Type::union(Type::number(), Type::none());
        assert_eq!(union, Type::Union(vec![Type::number(), Type::none()]));
        assert_eq!(union.to_string(), "Number | None");
    }

    #[test]
    fn test_union_of_the_same_type_collapses() {
        assert_eq!(Type::union(Type::number(), Type::number()), Type::number());
    }

    #[test]
    fn test_union_flattens_nested_union_on_the_left() {
        let union = Type::union(Type::union(Type::number(), Type::string()), Type::none());
        assert_eq!(
            union,
            Type::Union(vec![Type::number(), Type::string(), Type::none()])
        );
        assert_eq!(union.to_string(), "Number | String | None");
    }

    #[test]
    fn test_union_flattens_nested_union_on_the_right() {
        let union = Type::union(Type::none(), Type::union(Type::number(), Type::string()));
        assert_eq!(union.to_string(), "None | Number | String");
    }

    #[test]
    fn test_union_removes_duplicates_keeping_first_occurrence() {
        let union = Type::union(Type::union(Type::number(), Type::string()), Type::number());
        assert_eq!(union, Type::Union(vec![Type::number(), Type::string()]));
        assert_eq!(union.to_string(), "Number | String");
    }

    #[test]
    fn test_union_of_overlapping_unions() {
        let union = Type::union(
            Type::union(Type::number(), Type::string()),
            Type::union(Type::string(), Type::none()),
        );
        assert_eq!(union.to_string(), "Number | String | None");
    }

    #[test]
    fn test_union_of_identical_unions_collapses_to_that_union() {
        let union = Type::union(
            Type::union(Type::number(), Type::none()),
            Type::union(Type::number(), Type::none()),
        );
        assert_eq!(union, Type::Union(vec![Type::number(), Type::none()]));
    }
}
