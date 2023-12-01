#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenameAttr<T> {
    /// `= "value"`
    Both(T),
    /// `(serialize = "value")`
    SerializeOnly(T),
    /// `(deserialize = "value")`
    DeserializeOnly(T),
    /// `(serialize = "value", deserialize = "value")`
    ExplicitBoth { serialize: T, deserialize: T },
}

impl<T> RenameAttr<T> {
    pub fn serialize_ref(&self) -> Option<&T> {
        match self {
            Self::Both(serialize)
            | Self::SerializeOnly(serialize)
            | Self::ExplicitBoth { serialize, .. } => Some(serialize),
            _ => None,
        }
    }

    pub fn deserialize_ref(&self) -> Option<&T> {
        match self {
            Self::Both(deserialize)
            | Self::DeserializeOnly(deserialize)
            | Self::ExplicitBoth { deserialize, .. } => Some(deserialize),
            _ => None,
        }
    }
}

impl<T: ParseLitStr> syn::parse::Parse for RenameAttr<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Token![=]) {
            let _eq = input.parse::<syn::Token![=]>()?;
            let lit = input.parse()?;
            let value = T::parse_lit_str(lit)?;
            Ok(Self::Both(value))
        } else if lookahead.peek(syn::token::Paren) {
            struct State<T> {
                serialize: Option<T>,
                deserialize: Option<T>,
            }

            impl<T: ParseLitStr> State<T> {
                const fn new() -> Self {
                    Self {
                        serialize: None,
                        deserialize: None,
                    }
                }

                fn parse(&mut self, input: syn::parse::ParseStream) -> syn::Result<()> {
                    let ident = input.parse::<syn::Ident>()?;
                    let _eq = input.parse::<syn::Token![=]>()?;
                    let lit = input.parse()?;
                    let value = T::parse_lit_str(lit)?;

                    // Clone the string only once. <Ident as PartialEq<&str>> clones the string
                    // every time.
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "serialize" if self.serialize.is_some() => {
                            Err(syn::Error::new(ident.span(), "duplicate `serialize`"))
                        }
                        "serialize" => {
                            self.serialize = Some(value);
                            Ok(())
                        }
                        "deserialize" if self.deserialize.is_some() => {
                            Err(syn::Error::new(ident.span(), "duplicate `deserialize`"))
                        }
                        "deserialize" => {
                            self.deserialize = Some(value);
                            Ok(())
                        }
                        _ => Err(syn::Error::new(
                            ident.span(),
                            "expected `serialize` or `deserialize`",
                        )),
                    }
                }

                fn into_attr(self) -> syn::Result<RenameAttr<T>> {
                    match (self.serialize, self.deserialize) {
                        (Some(serialize), Some(deserialize)) => Ok(RenameAttr::ExplicitBoth {
                            serialize,
                            deserialize,
                        }),
                        (Some(value), None) => Ok(RenameAttr::SerializeOnly(value)),
                        (None, Some(value)) => Ok(RenameAttr::DeserializeOnly(value)),
                        (None, None) => unreachable!(),
                    }
                }
            }

            let mut state = State::new();

            let tokens;
            let _paren = syn::parenthesized!(tokens in input);

            state.parse(&tokens)?;

            if !tokens.is_empty() {
                let _comma = tokens.parse::<syn::Token![,]>()?;
                state.parse(&tokens)?;
            }

            if tokens.is_empty() {
                state.into_attr()
            } else {
                Err(tokens.error("expected `serialize` or `deserialize`"))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

pub trait ParseLitStr: Sized {
    fn parse_lit_str(lit: syn::LitStr) -> syn::Result<Self>;
}

impl ParseLitStr for syn::LitStr {
    fn parse_lit_str(lit: syn::LitStr) -> syn::Result<Self> {
        Ok(lit)
    }
}

impl ParseLitStr for String {
    fn parse_lit_str(lit: syn::LitStr) -> syn::Result<Self> {
        Ok(lit.value())
    }
}

impl ParseLitStr for crate::case::RenameRule {
    fn parse_lit_str(lit: syn::LitStr) -> syn::Result<Self> {
        let s = lit.value();
        Self::from_str(&s).map_err(|e| syn::Error::new(lit.span(), e))
    }
}
