//! Union types: `A | B`, a value of any one of the member types

use std::fmt;

use crate::runtime::types::Type;

impl Type {
    /// The union `a | b`: nested unions are flattened, duplicates are removed (keeping
    /// the first occurrence), and a single remaining member is returned as-is.
    pub fn union(a: Type, b: Type) -> Self {
        let mut members = Vec::new();
        add_member(&mut members, a);
        add_member(&mut members, b);
        if members.len() == 1 {
            members.remove(0)
        } else {
            Type::Union(members)
        }
    }
}

/// Adds `t` to `members`, flattening nested unions and skipping types already present.
fn add_member(members: &mut Vec<Type>, t: Type) {
    match t {
        Type::Union(inner) => {
            for member in inner {
                add_member(members, member);
            }
        }
        other => {
            if !members.contains(&other) {
                members.push(other);
            }
        }
    }
}

/// Writes the members of a union separated by ` | `, e.g. `Number | None`.
pub(super) fn fmt_union(members: &[Type], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (i, member) in members.iter().enumerate() {
        if i > 0 {
            write!(f, " | ")?;
        }
        write!(f, "{member}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::add_member;
    use crate::runtime::types::Type;

    #[test]
    fn test_add_member_to_empty_members() {
        let mut members = Vec::new();
        add_member(&mut members, Type::number());
        assert_eq!(members, vec![Type::number()]);
    }

    #[test]
    fn test_add_member_appends_after_existing_members() {
        let mut members = vec![Type::number()];
        add_member(&mut members, Type::string());
        assert_eq!(members, vec![Type::number(), Type::string()]);
    }

    #[test]
    fn test_add_member_skips_a_type_already_present() {
        let mut members = vec![Type::number(), Type::string()];
        add_member(&mut members, Type::number());
        assert_eq!(members, vec![Type::number(), Type::string()]);
    }

    #[test]
    fn test_add_member_flattens_a_union_into_its_members() {
        let mut members = vec![Type::none()];
        add_member(
            &mut members,
            Type::Union(vec![Type::number(), Type::string()]),
        );
        assert_eq!(members, vec![Type::none(), Type::number(), Type::string()]);
    }

    #[test]
    fn test_add_member_skips_union_members_already_present() {
        let mut members = vec![Type::string()];
        add_member(
            &mut members,
            Type::Union(vec![Type::number(), Type::string(), Type::none()]),
        );
        assert_eq!(members, vec![Type::string(), Type::number(), Type::none()]);
    }

    #[test]
    fn test_add_member_flattens_a_hand_built_nested_union() {
        let mut members = Vec::new();
        add_member(
            &mut members,
            Type::Union(vec![
                Type::Union(vec![Type::number(), Type::string()]),
                Type::none(),
            ]),
        );
        assert_eq!(members, vec![Type::number(), Type::string(), Type::none()]);
    }

    #[test]
    fn test_add_member_of_an_empty_union_adds_nothing() {
        let mut members = vec![Type::number()];
        add_member(&mut members, Type::Union(vec![]));
        assert_eq!(members, vec![Type::number()]);
    }

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
