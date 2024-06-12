#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct InvalidVariantError {
    variants: &'static [&'static str],
}

impl InvalidVariantError {
    pub const fn new(variants: &'static [&'static str]) -> Self {
        Self { variants }
    }
}

impl core::fmt::Display for InvalidVariantError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.variants.is_empty() {
            f.pad("invalid variant")
        } else {
            write!(f, "invalid variant, {}", Expected(self.variants))
        }
    }
}

impl core::fmt::Debug for InvalidVariantError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("InvalidVariantError")
    }
}

impl core::error::Error for InvalidVariantError {}

struct Expected(&'static [&'static str]);

impl core::fmt::Display for Expected {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            [] => unreachable!(),
            [single] => write!(f, "expected {single}"),
            many => write!(f, "expected {}", OneOf(many)),
        }
    }
}

struct OneOf(&'static [&'static str]);

impl core::fmt::Display for OneOf {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("one of: ")?;

        oxford_comma(f, self.0, "or")
    }
}

/// Writes a list of items with an Oxford comma.
#[inline]
fn oxford_comma(w: &mut dyn core::fmt::Write, items: &[&str], joiner: &str) -> core::fmt::Result {
    let [many @ .., n_sub_1, n_sub_0] = items else {
        // SAFETY: `Expected` does not call this function with less than 2 items.
        unsafe { core::hint::unreachable_unchecked() };
    };
    for item in many {
        write!(w, "{item}, ")?;
    }
    write!(w, "{n_sub_1} {joiner} {n_sub_0}")
}
