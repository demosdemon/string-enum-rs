#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct InvalidVariantError {
    _priv: (),
}

impl InvalidVariantError {
    pub const fn new() -> Self {
        Self { _priv: () }
    }
}

impl core::default::Default for InvalidVariantError {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Display for InvalidVariantError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("invalid variant")
    }
}

impl core::fmt::Debug for InvalidVariantError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("InvalidVariantError")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidVariantError {}
