use quote::{format_ident, quote};
use syn::{parenthesized, punctuated::Punctuated, Attribute, Expr, Ident, Path, Token};

/// Represents whether custom serde comes from `deserialize_with` or just `with`.
#[derive(Debug)]
pub enum CustomDe {
    /// `deserialize_with`
    Fn(Path),
    /// `with`
    Mod(Path),
}

/// Quote the necessary deserialization function tokens for the payload. This will call the original function
/// and simply wrap its resulting type in an `Option`. Any serde errors are propagated.
///
/// If the field is an option with custom serde, we don't touch the return value.
pub fn quote_custom_serde_payload_field(
    field_id: &Ident,
    ty: &syn::Type,
    custom_de: CustomDe,
    option: bool,
) -> (Ident, proc_macro2::TokenStream) {
    let (id, fn_path) = match custom_de {
        CustomDe::Fn(ref p) => (p.segments.last().unwrap(), p),
        CustomDe::Mod(ref p) => (p.segments.last().unwrap(), p),
    };

    let custom_fn_id = format_ident!("{}_{field_id}_payload", id.ident);

    // `with` must always be a module with `serialize` and `deserialize` as per serde
    let module_de = matches!(custom_de, CustomDe::Mod(_)).then_some(quote!(::deserialize));

    let res = if option {
        quote!(res)
    } else {
        quote!(Some(res))
    };

    let ty = if option {
        quote!(#ty)
    } else {
        quote!(Option<#ty>)
    };

    let tokens = quote!(
        fn #custom_fn_id<'de, D>(deserializer: D) -> Result<#ty, D::Error>
        where
          D: serde::Deserializer<'de>
        {
            match #fn_path #module_de (deserializer) {
                Ok(res) => Ok(#res),
                Err(e) => Err(e)
            }
        }
    );

    (custom_fn_id, tokens)
}

/// Attempts to find `serde(with = "..")` and returns it as the first element with the remaining attributes as the second
pub fn extract_custom_serde<'a>(
    serde_attrs: &[&'a Attribute],
) -> (Option<CustomDe>, Vec<&'a Attribute>) {
    let mut custom_fn = None;
    let mut rest = vec![];

    for attr in serde_attrs {
        if custom_fn.is_some() {
            rest.push(*attr);
            continue;
        }

        // Safe to unwrap because we know all serde attrs are lists
        let metas = attr.meta.require_list().unwrap();

        let parsed = metas.parse_nested_meta(|meta| {
            // Covers `deserialize_with = "function"`
            if meta.path.is_ident("deserialize_with") && meta.input.peek(Token!(=)) {
                let content = meta.value()?;
                if let Ok(lit) = content.parse::<syn::LitStr>() {
                    custom_fn = Some(CustomDe::Fn(syn::parse_str::<Path>(&lit.value())?));
                    return Ok(());
                }
            }

            // Covers `with = "module"`
            if meta.path.is_ident("with") && meta.input.peek(Token!(=)) {
                let content = meta.value()?;
                if let Ok(lit) = content.parse::<syn::LitStr>() {
                    custom_fn = Some(CustomDe::Mod(syn::parse_str::<Path>(&lit.value())?));
                    return Ok(());
                }
            }

            // Return an error here because we want to push the serde
            // attr to the rest vec if we didn't find a custom deser
            Err(meta.error("will get caught"))
        });

        if parsed.is_err() {
            rest.push(*attr);
            continue;
        };
    }

    (custom_fn, rest)
}

/// Attempts to find serde's `serde(rename_all = "..")` attribute and returns the specified rename rule.
pub fn find_rename_all(attrs: &[syn::Attribute]) -> Option<RenameRule> {
    let mut rule = None;

    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let Ok(metas) = attr.meta.require_list() else {
            continue;
        };

        let parsed = metas.parse_nested_meta(|meta| {
            // Covers `rename_all = "something"`
            if meta.path.is_ident("rename_all") && meta.input.peek(Token!(=)) {
                let content = meta.value()?;
                if let Ok(lit) = content.parse::<syn::LitStr>() {
                    rule = RenameRule::from_str(&lit.value());
                    return Ok(());
                }
            }

            if meta.input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in meta.input);

                // Covers `rename_all(deserialize = "something")`
                let name_values =
                    Punctuated::<syn::MetaNameValue, Token![,]>::parse_separated_nonempty(
                        &content,
                    )?;

                for pair in name_values.pairs() {
                    let name_value = pair.into_value();
                    // Only interested in deserialize, since client payloads are originally in
                    // the given case, we want the errors to match it
                    if name_value.path.is_ident("deserialize") {
                        let Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(ref lit),
                            ..
                        }) = name_value.value
                        else {
                            return Ok(());
                        };
                        rule = RenameRule::from_str(&lit.value());
                    }
                }
                return Ok(());
            }

            Ok(())
        });

        if parsed.is_err() {
            continue;
        }
    }

    rule
}

/// Attempts to find the `serde(rename = "..")` value to use in the generated errors
pub fn find_rename(field: &syn::Field) -> Option<String> {
    let mut original_name = None;

    for attr in field.attrs.iter() {
        if !attr.path().is_ident("serde") {
            continue;
        }

        // serde field attributes are always lists
        let Ok(serde_meta) = attr.meta.require_list() else {
            continue;
        };

        // The function will stop as soon as this errors
        let parsed = serde_meta.parse_nested_meta(|meta| {
            if !meta.path.is_ident("rename") {
                return Ok(());
            }

            // Covers `rename = "something"`
            if meta.input.peek(Token!(=)) {
                let content = meta.value()?;
                original_name = Some(content.parse::<syn::LitStr>()?.value());
                return Ok(());
            }

            // Covers `rename(deserialize = "something")`
            if meta.input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in meta.input);

                let name_values =
                    Punctuated::<syn::MetaNameValue, Token![,]>::parse_separated_nonempty(
                        &content,
                    )?;

                for pair in name_values.pairs() {
                    let name_value = pair.into_value();

                    // We're only interested in the deserialize property as that is the
                    // one related to the client payload
                    if name_value.path.is_ident("deserialize") {
                        let Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(ref lit),
                            ..
                        }) = name_value.value
                        else {
                            return Ok(());
                        };
                        original_name = Some(lit.value())
                    }
                }
                return Ok(());
            }

            Ok(())
        });

        if parsed.is_err() {
            continue;
        }
    }

    original_name
}

/// Taken from [serde](https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/case.rs).
/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RenameRule {
    /// Rename direct children to "lowercase" style.
    Lower,
    /// Rename direct children to "UPPERCASE" style.
    Upper,
    /// Rename direct children to "Pascal" style, as typically used for
    /// enum variants.
    Pascal,
    /// Rename direct children to "camel" style.
    Camel,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    Snake,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnake,
    /// Rename direct children to "kebab-case" style.
    Kebab,
    /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
    ScreamingKebab,
}

impl RenameRule {
    pub fn from_str(rename_all_str: &str) -> Option<Self> {
        for (name, rule) in RENAME_RULES {
            if rename_all_str == *name {
                return Some(*rule);
            }
        }
        None
    }

    /// Apply a renaming rule to a struct field, returning the version expected in the source.
    pub fn apply_to_field(self, field: &str) -> String {
        use RenameRule as RR;
        match self {
            RR::Lower | RR::Snake => field.to_owned(),
            RR::Upper => field.to_ascii_uppercase(),
            RR::Pascal => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            RR::Camel => {
                let pascal = RR::Pascal.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            RR::ScreamingSnake => field.to_ascii_uppercase(),
            RR::Kebab => field.replace('_', "-"),
            RR::ScreamingKebab => RR::ScreamingSnake.apply_to_field(field).replace('_', "-"),
        }
    }
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", RenameRule::Lower),
    ("UPPERCASE", RenameRule::Upper),
    ("PascalCase", RenameRule::Pascal),
    ("camelCase", RenameRule::Camel),
    ("snake_case", RenameRule::Snake),
    ("SCREAMING_SNAKE_CASE", RenameRule::ScreamingSnake),
    ("kebab-case", RenameRule::Kebab),
    ("SCREAMING-KEBAB-CASE", RenameRule::ScreamingKebab),
];
