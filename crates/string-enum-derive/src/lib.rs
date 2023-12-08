mod case;
mod rename;

use proc_macro2::Delimiter;
use proc_macro2::Group;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use rename::ParseLitStr;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Ident;
use syn::LitStr;
use syn::MacroDelimiter;
use syn::Meta;
use syn::MetaList;
use syn::Variant;

use crate::case::RenameRule;
use crate::rename::RenameAttr;

#[proc_macro_derive(StringEnum, attributes(str))]
pub fn derive_string_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    Enum::try_from(input)
        .map(|e| e.derive())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

struct Enum {
    ident: Ident,
    non_exhaustive: bool,
    rename_all: Option<RenameAttr<RenameRule>>,
    variants: Vec<EnumVariant>,
}

struct EnumVariant {
    ident: Ident,
    rename: Option<RenameAttr<LitStr>>,
}

struct Attrs<T> {
    non_exhaustive: bool,
    rename: Option<RenameAttr<T>>,
}

enum Source {
    Str,
    Serde,
}

enum AttrTokens {
    Skip,
    NonExhaustive,
    Str(Source, proc_macro2::Span, TokenStream),
}

impl Enum {
    fn derive(&self) -> TokenStream {
        let Enum {
            ident,
            non_exhaustive,
            rename_all,
            variants,
        } = self;

        let const_variants_elems = variants.iter().map(|v| {
            let ident = &v.ident;
            quote!(Self::#ident)
        });

        let as_str_arms = variants.iter().map(|v| {
            let ident = &v.ident;
            let name = variant_name(ident, rename_all.serialize_ref(), v.serialize_ref());
            quote!(Self::#ident => #name)
        });
        let as_str_remainder = if *non_exhaustive {
            quote!(_ => {
                #[cold]
                fn non_exhaustive_unreachable() -> ! {
                    ::core::unreachable!("non-exhaustive enum")
                }
                non_exhaustive_unreachable()
            })
        } else {
            TokenStream::new()
        };

        let from_str_arms = variants.iter().map(|v| {
            let ident = &v.ident;
            let name = variant_name(ident, rename_all.deserialize_ref(), v.deserialize_ref());
            quote!(#name => ::core::result::Result::Ok(Self::#ident))
        });

        quote! {
            impl string_enum::StringEnum for #ident {
                const VARIANTS: &'static [Self] = &[#(#const_variants_elems,)*];

                fn as_str(&self) -> &'static str {
                    match *self {
                        #(#as_str_arms,)*
                        #as_str_remainder
                    }
                }
            }

            impl ::core::str::FromStr for #ident {
                type Err = string_enum::InvalidVariantError;

                fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                    match s {
                        #(#from_str_arms,)*
                        _ => ::core::result::Result::Err(string_enum::InvalidVariantError::new()),
                    }
                }
            }

            impl ::core::fmt::Display for #ident {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::pad(f, string_enum::StringEnum::as_str(self))
                }
            }
        }
    }
}

impl TryFrom<DeriveInput> for Enum {
    type Error = Error;

    fn try_from(value: DeriveInput) -> syn::Result<Self> {
        let DeriveInput {
            attrs, ident, data, ..
        } = value;

        let Attrs {
            rename: rename_all,
            non_exhaustive,
        } = Attrs::parse_attrs(attrs, "rename_all")?;

        let variants = match data {
            Data::Enum(data) => data.variants.into_iter().map(TryFrom::try_from).collect(),
            Data::Struct(ref data) => Err(Error::new(data.struct_token.span, "expected enum")),
            Data::Union(ref data) => Err(Error::new(data.union_token.span, "expected enum")),
        }?;

        Ok(Self {
            ident,
            non_exhaustive,
            rename_all,
            variants,
        })
    }
}

impl TryFrom<Variant> for EnumVariant {
    type Error = Error;

    fn try_from(value: Variant) -> syn::Result<Self> {
        let Variant {
            attrs,
            ident,
            fields,
            ..
        } = value;

        let Attrs { rename, .. } = Attrs::parse_attrs(attrs, "rename")?;

        if matches!(fields, syn::Fields::Unit) {
            Ok(Self { ident, rename })
        } else {
            Err(Error::new(ident.span(), "expected unit variant"))
        }
    }
}

impl<T: ParseLitStr> Attrs<T> {
    fn parse_attrs(attrs: Vec<Attribute>, serde_attr: &str) -> syn::Result<Self> {
        let mut rename = None;
        let mut non_exhaustive = false;

        for attr in attrs {
            let (source, span, tokens) = match get_attr_tokens(serde_attr, attr.meta)? {
                AttrTokens::Skip => {
                    continue;
                }
                AttrTokens::NonExhaustive => {
                    non_exhaustive = true;
                    continue;
                }
                AttrTokens::Str(source, span, tokens) => (source, span, tokens),
            };

            if matches!(&rename, Some((Source::Str, _))) {
                match source {
                    Source::Str => {
                        return Err(Error::new(span, "duplicate #[str = \"...\"] attribute"));
                    }
                    Source::Serde => {
                        continue;
                    }
                }
            }

            rename = Some((source, syn::parse2(tokens)?));
        }

        Ok(Self {
            non_exhaustive,
            rename: rename.map(|(_, v)| v),
        })
    }
}

fn get_attr_tokens(serde_attr: &str, meta: Meta) -> syn::Result<AttrTokens> {
    macro_rules! some {
        ($expr:expr) => {
            match $expr {
                Some(value) => value,
                None => return Ok(AttrTokens::Skip),
            }
        };
    }

    match meta {
        Meta::Path(path) => {
            let ident = some!(path.get_ident());
            let ident_str = ident.to_string();
            match ident_str.as_str() {
                "non_exhaustive" => Ok(AttrTokens::NonExhaustive),
                "str" => Err(Error::new(ident.span(), "expected #[str = \"...\"]")),
                _ => Ok(AttrTokens::Skip),
            }
        }
        Meta::List(meta) => {
            let ident = some!(meta.path.get_ident());
            let ident_str = ident.to_string();
            match ident_str.as_str() {
                "str" => Ok(AttrTokens::Str(
                    Source::Str,
                    ident.span(),
                    surround(meta.delimiter, meta.tokens),
                )),
                "serde" => Ok(AttrTokens::Str(
                    Source::Serde,
                    ident.span(),
                    some!(serde_rename(serde_attr, meta)?),
                )),
                _ => Ok(AttrTokens::Skip),
            }
        }
        Meta::NameValue(meta) => {
            let ident = some!(meta.path.get_ident());
            if ident != "str" {
                return Ok(AttrTokens::Skip);
            }
            let mut tokens = TokenStream::new();
            meta.eq_token.to_tokens(&mut tokens);
            meta.value.to_tokens(&mut tokens);
            Ok(AttrTokens::Str(Source::Str, ident.span(), tokens))
        }
    }
}

fn serde_rename(name: &str, meta: MetaList) -> syn::Result<Option<TokenStream>> {
    let mut res = None;
    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident(name) {
            res = Some(meta.input.parse()?);
        }
        Ok(())
    })?;
    Ok(res)
}

fn surround(delimiter: MacroDelimiter, tokens: TokenStream) -> TokenStream {
    let (delim, span) = match delimiter {
        MacroDelimiter::Paren(paren) => (Delimiter::Parenthesis, paren.span),
        MacroDelimiter::Brace(brace) => (Delimiter::Brace, brace.span),
        MacroDelimiter::Bracket(bracket) => (Delimiter::Bracket, bracket.span),
    };
    let mut g = Group::new(delim, tokens);
    g.set_span(span.join());
    g.into_token_stream()
}

fn variant_name(ident: &Ident, rename_all: Option<&RenameRule>, rename: Option<&LitStr>) -> String {
    if let Some(rename) = rename {
        rename.value()
    } else if let Some(rule) = rename_all {
        rule.apply_to_variant(ident.to_string())
    } else {
        ident.to_string()
    }
}

trait SerDe<T> {
    fn serialize_ref(&self) -> Option<&T>;
    fn deserialize_ref(&self) -> Option<&T>;
}

impl<T> SerDe<T> for RenameAttr<T> {
    fn serialize_ref(&self) -> Option<&T> {
        RenameAttr::serialize_ref(self)
    }

    fn deserialize_ref(&self) -> Option<&T> {
        RenameAttr::deserialize_ref(self)
    }
}

impl<T, V: SerDe<T>> SerDe<T> for Option<V> {
    fn serialize_ref(&self) -> Option<&T> {
        self.as_ref().and_then(SerDe::serialize_ref)
    }

    fn deserialize_ref(&self) -> Option<&T> {
        self.as_ref().and_then(SerDe::deserialize_ref)
    }
}

impl SerDe<LitStr> for EnumVariant {
    fn serialize_ref(&self) -> Option<&LitStr> {
        self.rename.serialize_ref()
    }

    fn deserialize_ref(&self) -> Option<&LitStr> {
        self.rename.deserialize_ref()
    }
}
