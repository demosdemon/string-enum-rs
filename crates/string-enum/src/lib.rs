#![no_std]

mod error;

#[cfg(feature = "derive")]
pub use string_enum_derive::StringEnum;

pub use crate::error::InvalidVariantError;

pub trait StringEnum: Copy + Sized + 'static {
    const VARIANTS: &'static [Self];

    fn as_str(&self) -> &'static str;
}

#[cfg(all(test, feature = "derive"))]
mod test {
    #![allow(clippy::unwrap_used)]

    extern crate alloc;

    use core::fmt::Debug;
    use core::str::FromStr;

    use serde::Deserialize;
    use serde::Serialize;
    use string_enum::StringEnum;

    use crate as string_enum;
    use crate::InvalidVariantError;

    #[test]
    fn test_empty_enum() {
        #[derive(Debug, Clone, Copy, PartialEq, StringEnum)]
        enum EmptyEnum {}

        assert_eq!(EmptyEnum::VARIANTS.len(), 0);

        let err = EmptyEnum::from_str("invalid").unwrap_err();

        let err = alloc::format!("{err}");
        assert_eq!(err, "invalid variant");
    }

    #[test]
    fn test_non_exhaustive() {
        #[derive(Debug, Clone, Copy, PartialEq, StringEnum)]
        #[non_exhaustive]
        enum NonExhaustiveEnum {
            Alpha,
            Beta,
        }

        test_enum(
            &[
                TestCase::new(NonExhaustiveEnum::Alpha, "Alpha", "Alpha"),
                TestCase::new(NonExhaustiveEnum::Beta, "Beta", "Beta"),
            ],
            "invalid variant, expected one of: Alpha or Beta",
        );
    }

    #[test]
    fn test_with_rename_rule() {
        #[derive(Debug, Clone, Copy, PartialEq, string_enum::StringEnum)]
        #[str = "camelCase"]
        enum WithRenameRule {
            SelectOne,
            SelectTwo,
            #[str = "OVERRIDE"]
            Override,
        }

        test_enum(
            &[
                TestCase::new(WithRenameRule::SelectOne, "selectOne", "selectOne"),
                TestCase::new(WithRenameRule::SelectTwo, "selectTwo", "selectTwo"),
                TestCase::new(WithRenameRule::Override, "OVERRIDE", "OVERRIDE"),
            ],
            "invalid variant, expected one of: selectOne, selectTwo or OVERRIDE",
        );
    }

    #[test]
    fn test_with_serde_rules() {
        #[derive(Debug, Clone, Copy, PartialEq, StringEnum, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        enum WithSerdeRules {
            SelectOne,
            SelectTwo,
            #[serde(rename = "OVERRIDE")]
            Override,
        }

        test_enum(
            &[
                TestCase::new(WithSerdeRules::SelectOne, "selectOne", "selectOne"),
                TestCase::new(WithSerdeRules::SelectTwo, "selectTwo", "selectTwo"),
                TestCase::new(WithSerdeRules::Override, "OVERRIDE", "OVERRIDE"),
            ],
            "invalid variant, expected one of: selectOne, selectTwo or OVERRIDE",
        );
    }

    #[test]
    fn test_with_mixed_serde_rules() {
        #[derive(Debug, Clone, Copy, PartialEq, StringEnum, Serialize, Deserialize)]
        #[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
        enum WithMixedSerdeRules {
            #[serde(rename(deserialize = "select1"))]
            SelectOne,
            #[serde(rename(serialize = "select2"))]
            SelectTwo,
            #[serde(rename = "OVERRIDE")]
            Override,
        }

        test_enum(
            &[
                TestCase::new(WithMixedSerdeRules::SelectOne, "selectOne", "select1"),
                TestCase::new(WithMixedSerdeRules::SelectTwo, "select2", "select_two"),
                TestCase::new(WithMixedSerdeRules::Override, "OVERRIDE", "OVERRIDE"),
            ],
            "invalid variant, expected one of: select1, select_two or OVERRIDE",
        );
    }

    #[test]
    fn test_with_ignore_serde_rules() {
        #[derive(Debug, Clone, Copy, PartialEq, StringEnum, Serialize, Deserialize)]
        #[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
        #[str = "PascalCase"]
        enum WithIgnoreSerdeRules {
            #[serde(rename(deserialize = "select1"))]
            #[str = "SelectOne"]
            SelectOne,
            #[serde(rename(serialize = "select2"))]
            #[str = "SelectTwo"]
            SelectTwo,
            #[serde(rename = "OVERRIDE")]
            #[str = "Override"]
            Override,
        }

        test_enum(
            &[
                TestCase::new(WithIgnoreSerdeRules::SelectOne, "SelectOne", "SelectOne"),
                TestCase::new(WithIgnoreSerdeRules::SelectTwo, "SelectTwo", "SelectTwo"),
                TestCase::new(WithIgnoreSerdeRules::Override, "Override", "Override"),
            ],
            "invalid variant, expected one of: SelectOne, SelectTwo or Override",
        );
    }

    struct TestCase<'a, E> {
        variant: E,
        as_str: &'a str,
        from_str: &'a str,
    }

    impl<'a, E> TestCase<'a, E>
    where
        E: Debug + PartialEq + FromStr<Err = InvalidVariantError> + StringEnum,
    {
        fn new(variant: E, as_str: &'a str, from_str: &'a str) -> Self {
            Self {
                variant,
                as_str,
                from_str,
            }
        }
    }

    fn test_enum<E>(cases: &[TestCase<E>], error: &str)
    where
        E: Debug + PartialEq + FromStr<Err = InvalidVariantError> + StringEnum,
    {
        assert_eq!(E::VARIANTS.len(), cases.len());
        let name = core::any::type_name::<E>();

        for TestCase {
            variant,
            as_str,
            from_str,
        } in cases
        {
            assert_eq!(
                variant.as_str(),
                *as_str,
                "{name}::as_str {variant:?} {} != {as_str:?}",
                variant.as_str()
            );

            assert_eq!(
                Ok(*variant),
                E::from_str(from_str),
                "{name}::from_str {variant:?} {from_str:?}"
            );
        }

        let err = E::from_str("invalid").unwrap_err();
        let err = alloc::format!("{err}");
        assert_eq!(err, error);
    }
}
