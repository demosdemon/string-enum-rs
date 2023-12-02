#![cfg_attr(not(feature = "std"), no_std)]

mod error;

pub use string_enum_derive::StringEnum;

pub use crate::error::InvalidVariantError;

pub trait StringEnum: Copy + Sized + 'static {
    const VARIANTS: &'static [Self];

    fn as_str(&self) -> &'static str;
}

pub fn parse_str<T: StringEnum>(s: &str) -> Result<T, InvalidVariantError> {
    T::VARIANTS
        .iter()
        .find(|v| v.as_str() == s)
        .copied()
        .ok_or(InvalidVariantError::new())
}

pub fn parse_str_ignore_ascii_case<T: StringEnum>(s: &str) -> Result<T, InvalidVariantError> {
    T::VARIANTS
        .iter()
        .find(|v| v.as_str().eq_ignore_ascii_case(s))
        .copied()
        .ok_or(InvalidVariantError::new())
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]

    use core::fmt::Debug;
    use core::str::FromStr;

    use serde::Deserialize;
    use serde::Serialize;

    // use string_enum::StringEnum;
    use crate as string_enum;
    use crate::InvalidVariantError;

    #[derive(Debug, Clone, Copy, PartialEq, string_enum::StringEnum)]
    enum EmptyEnum {}

    #[test]
    fn test_non_exhaustive() {
        #[derive(Debug, Clone, Copy, PartialEq, string_enum::StringEnum)]
        #[non_exhaustive]
        enum NonExhaustiveEnum {
            Alpha,
            Beta,
        }

        test_enum(&[
            (NonExhaustiveEnum::Alpha, "Alpha", "Alpha"),
            (NonExhaustiveEnum::Beta, "Beta", "Beta"),
        ]);
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

        test_enum(&[
            (WithRenameRule::SelectOne, "selectOne", "selectOne"),
            (WithRenameRule::SelectTwo, "selectTwo", "selectTwo"),
            (WithRenameRule::Override, "OVERRIDE", "OVERRIDE"),
        ]);
    }

    #[test]
    fn test_with_serde_rules() {
        #[derive(
            Debug, Clone, Copy, PartialEq, string_enum::StringEnum, Serialize, Deserialize,
        )]
        #[serde(rename_all = "camelCase")]
        enum WithSerdeRules {
            SelectOne,
            SelectTwo,
            #[serde(rename = "OVERRIDE")]
            Override,
        }

        test_enum(&[
            (WithSerdeRules::SelectOne, "selectOne", "selectOne"),
            (WithSerdeRules::SelectTwo, "selectTwo", "selectTwo"),
            (WithSerdeRules::Override, "OVERRIDE", "OVERRIDE"),
        ]);
    }

    #[test]
    fn test_with_mixed_serde_rules() {
        #[derive(
            Debug, Clone, Copy, PartialEq, string_enum::StringEnum, Serialize, Deserialize,
        )]
        #[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
        enum WithMixedSerdeRules {
            #[serde(rename(deserialize = "select1"))]
            SelectOne,
            #[serde(rename(serialize = "select2"))]
            SelectTwo,
            #[serde(rename = "OVERRIDE")]
            Override,
        }

        test_enum(&[
            (WithMixedSerdeRules::SelectOne, "selectOne", "select1"),
            (WithMixedSerdeRules::SelectTwo, "select2", "select_two"),
            (WithMixedSerdeRules::Override, "OVERRIDE", "OVERRIDE"),
        ]);
    }

    #[test]
    fn test_with_ignore_serde_rules() {
        #[derive(
            Debug, Clone, Copy, PartialEq, string_enum::StringEnum, Serialize, Deserialize,
        )]
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

        test_enum(&[
            (WithIgnoreSerdeRules::SelectOne, "SelectOne", "SelectOne"),
            (WithIgnoreSerdeRules::SelectTwo, "SelectTwo", "SelectTwo"),
            (WithIgnoreSerdeRules::Override, "Override", "Override"),
        ]);
    }

    fn test_enum<E>(cases: &[(E, &str, &str)])
    where
        E: Debug + PartialEq + FromStr<Err = InvalidVariantError> + string_enum::StringEnum,
    {
        assert_eq!(E::VARIANTS.len(), cases.len());
        let name = core::any::type_name::<E>();

        for (variant, as_str, from_str) in cases {
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

        assert!(E::from_str("invalid").is_err());
    }
}
