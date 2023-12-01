// original https://github.com/serde-rs/serde/blob/44613c7d0190dbb5ecd2d5ec19c636f45b7488cc/serde_derive/src/internals/case.rs
// original license: MIT OR Apache-2.0

//! Code to convert the Rust-styled field/variant (e.g. `my_field`, `MyType`) to
//! the case of the source (e.g. `my-field`, `MY_FIELD`).

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::{self};

use self::RenameRule::*;

/// The different possible ways to change case of fields in a struct, or
/// variants in an enum.
#[derive(Default, Copy, Clone, PartialEq)]
pub enum RenameRule {
    /// Don't apply a default rename rule.
    #[default]
    None,
    /// Rename direct children to "lowercase" style.
    LowerCase,
    /// Rename direct children to "UPPERCASE" style.
    UpperCase,
    /// Rename direct children to "PascalCase" style, as typically used for
    /// enum variants.
    PascalCase,
    /// Rename direct children to "camelCase" style.
    CamelCase,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    SnakeCase,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnakeCase,
    /// Rename direct children to "kebab-case" style.
    KebabCase,
    /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
    ScreamingKebabCase,
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", LowerCase),
    ("UPPERCASE", UpperCase),
    ("PascalCase", PascalCase),
    ("camelCase", CamelCase),
    ("snake_case", SnakeCase),
    ("SCREAMING_SNAKE_CASE", ScreamingSnakeCase),
    ("kebab-case", KebabCase),
    ("SCREAMING-KEBAB-CASE", ScreamingKebabCase),
];

impl RenameRule {
    pub fn from_str(rename_all_str: &str) -> Result<Self, ParseError> {
        for (name, rule) in RENAME_RULES {
            if rename_all_str == *name {
                return Ok(*rule);
            }
        }
        Err(ParseError {
            unknown: rename_all_str,
        })
    }

    /// Apply a renaming rule to an enum variant, returning the version expected
    /// in the source.
    pub fn apply_to_variant(self, variant: impl Into<String>) -> String {
        let mut variant = variant.into();
        match self {
            None | PascalCase => {}
            LowerCase => variant.make_ascii_lowercase(),
            UpperCase => variant.make_ascii_uppercase(),
            CamelCase => variant[..1].make_ascii_lowercase(),
            SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                variant = snake;
            }
            ScreamingSnakeCase => {
                variant = SnakeCase.apply_to_variant(variant);
                variant.make_ascii_uppercase();
            }
            KebabCase => {
                variant = SnakeCase.apply_to_variant(variant);
                str_replace_inline(&mut variant, b'_', b'-');
            }
            ScreamingKebabCase => {
                variant = ScreamingSnakeCase.apply_to_variant(variant);
                str_replace_inline(&mut variant, b'_', b'-');
            }
        }

        variant
    }
}

pub struct ParseError<'a> {
    unknown: &'a str,
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unknown rename rule `rename_all = ")?;
        Debug::fmt(self.unknown, f)?;
        f.write_str("`, expected one of ")?;
        for (i, (name, _rule)) in RENAME_RULES.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            Debug::fmt(name, f)?;
        }
        Ok(())
    }
}

fn str_replace_inline(haystack: &mut String, needle: u8, replace: u8) {
    debug_assert!(needle.is_ascii());
    debug_assert!(replace.is_ascii());

    unsafe {
        let haystack = haystack.as_mut_vec();
        for byte in haystack {
            if *byte == needle {
                *byte = replace;
            }
        }
    }
}

#[test]
fn rename_variants() {
    for &(original, lower, upper, camel, snake, screaming, kebab, screaming_kebab) in &[
        (
            "Outcome", "outcome", "OUTCOME", "outcome", "outcome", "OUTCOME", "outcome", "OUTCOME",
        ),
        (
            "VeryTasty",
            "verytasty",
            "VERYTASTY",
            "veryTasty",
            "very_tasty",
            "VERY_TASTY",
            "very-tasty",
            "VERY-TASTY",
        ),
        ("A", "a", "A", "a", "a", "A", "a", "A"),
        ("Z42", "z42", "Z42", "z42", "z42", "Z42", "z42", "Z42"),
    ] {
        assert_eq!(None.apply_to_variant(original), original);
        assert_eq!(LowerCase.apply_to_variant(original), lower);
        assert_eq!(UpperCase.apply_to_variant(original), upper);
        assert_eq!(PascalCase.apply_to_variant(original), original);
        assert_eq!(CamelCase.apply_to_variant(original), camel);
        assert_eq!(SnakeCase.apply_to_variant(original), snake);
        assert_eq!(ScreamingSnakeCase.apply_to_variant(original), screaming);
        assert_eq!(KebabCase.apply_to_variant(original), kebab);
        assert_eq!(
            ScreamingKebabCase.apply_to_variant(original),
            screaming_kebab
        );
    }
}
