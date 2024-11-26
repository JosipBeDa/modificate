use super::parser::*;
use super::validation::{
    Contains, CreditCard, Custom, Email, In, Ip, NonControlChar, Phone, Regex, Required,
    SchemaValidation, Url, Validator,
};
use crate::fields::{Fields, Variants};
use crate::tokens::quote_schema_validation;
use crate::validate::ValidationMeta;
use proc_macro_error::abort;
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::parenthesized;
use syn::spanned::Spanned;

const VALIDATE: &str = "validate";
const VALIDIFY: &str = "validify";

const EMAIL: &str = "email";
const URL: &str = "url";
const LENGTH: &str = "length";
const RANGE: &str = "range";
const CONTAINS: &str = "contains";
const CONTAINS_NOT: &str = "contains_not";
const NON_CONTROL_CHAR: &str = "non_control_char";
const CUSTOM: &str = "custom";
const REGEX: &str = "regex";
const CREDIT_CARD: &str = "credit_card";
const PHONE: &str = "phone";
const REQUIRED: &str = "required";
const IS_IN: &str = "is_in";
const NOT_IN: &str = "not_in";
const IP: &str = "ip";
const TIME: &str = "time";
const ITER: &str = "iter";

/// Entrypoint for `#[derive(Validate)]`.
pub fn impl_validate(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &input.ident;

    match input.data {
        syn::Data::Struct(ref data_struct) => {
            let fields = Fields::collect(&input.attrs, &data_struct.fields);
            let field_validation = fields.to_validate_tokens();

            let schema_validation = collect_schema_validation(&input.attrs).unwrap();
            let schema_validation = quote_schema_validation(&schema_validation);

            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

            quote!(
                impl #impl_generics ::validify::Validate for #ident #ty_generics #where_clause {
                    fn validate(&self) -> ::std::result::Result<(), ::validify::ValidationErrors> {
                        let mut errors = ::validify::ValidationErrors::new();

                        #(#field_validation)*

                        #(#schema_validation)*

                        if errors.is_empty() {
                            ::std::result::Result::Ok(())
                        } else {
                            ::std::result::Result::Err(errors)
                        }
                    }
                }
            )
        }
        syn::Data::Enum(ref data_enum) => {
            let variant_validation = Variants::collect(data_enum).to_validate_tokens();

            let schema_validation = collect_schema_validation(&input.attrs).unwrap();
            let schema_validation = quote_schema_validation(&schema_validation);

            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

            quote!(
                impl #impl_generics ::validify::Validate for #ident #ty_generics #where_clause {
                    fn validate(&self) -> ::std::result::Result<(), ::validify::ValidationErrors> {
                        let mut errors = ::validify::ValidationErrors::new();

                        #variant_validation

                        #(#schema_validation)*

                        if errors.is_empty() {
                            ::std::result::Result::Ok(())
                        } else {
                            ::std::result::Result::Err(errors)
                        }
                    }
                }
            )
        }
        syn::Data::Union(_) => abort!(
            input.span(),
            "#[derive(Validate)] can only be used on structs with named fields or enums"
        ),
    }
}

/// Find if a struct or enum has some schema validation and returns the info if so.
fn collect_schema_validation(
    attrs: &[syn::Attribute],
) -> Result<Vec<SchemaValidation>, syn::Error> {
    let mut validations = vec![];
    let filtered = attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident(VALIDATE));

    for attr in filtered {
        attr.parse_nested_meta(|meta| {
            validations.push(SchemaValidation {
                function: meta.path,
            });
            Ok(())
        })?;
    }

    Ok(validations)
}

pub fn collect_validation(field: &syn::Field) -> Vec<Validator> {
    let mut validators = vec![];

    for attr in field.attrs.iter() {
        if !attr.path().is_ident(VALIDATE) && !attr.path().is_ident(VALIDIFY) {
            continue;
        }

        let syn::Meta::List(ref list) = attr.meta else {
            let syn::Meta::Path(_) = attr.meta else {
                abort!(
                    attr.meta.span(),
                    "Validate must be applied as a list `#[validate(/*...*/)]` or as a path `#[validate]` for nested validation"
                )
            };
            validators.push(Validator::Nested);
            continue;
        };

        list.parse_nested_meta(|meta| {
            if meta.path.is_ident(ITER) {
                let mut validators_iter = vec![];
                meta.parse_nested_meta(|meta| {
                    parse_single_validation(meta, &mut validators_iter)?;
                    Ok(())
                })?;
                validators.push(Validator::Iter(validators_iter));
            } else {
                parse_single_validation(meta, &mut validators)?;
            }
            Ok(())
        })
        .unwrap_or_else(|e| abort!(e.span(), e));
    }

    validators
}

fn parse_single_validation(
    meta: ParseNestedMeta<'_>,
    validators: &mut Vec<Validator>,
) -> Result<(), syn::Error> {
    if meta.path.is_ident(EMAIL) {
        if meta.is_full_pattern() {
            let validation = parse_email_full(&meta)?;
            validators.push(Validator::Email(validation));
        } else {
            validators.push(Validator::Email(Email::default()));
        }
        return Ok(());
    }

    if meta.path.is_ident(URL) {
        if meta.is_full_pattern() {
            let validation = parse_url_full(&meta)?;
            validators.push(Validator::Url(validation));
        } else {
            validators.push(Validator::Url(Url::default()));
        }

        return Ok(());
    }

    if meta.path.is_ident(LENGTH) {
        let validation = parse_length(&meta)?;
        validators.push(Validator::Length(validation));
        return Ok(());
    }

    if meta.path.is_ident(RANGE) {
        let validation = parse_range(&meta)?;
        validators.push(Validator::Range(validation));
        return Ok(());
    }

    if meta.path.is_ident(CONTAINS) {
        if meta.is_single_lit(CONTAINS) {
            let content;
            parenthesized!(content in meta.input);
            let Ok(lit) = content.parse::<syn::Lit>() else {
                return Err(meta.error(
                    "Invalid value given for `contains` validation, must be a path or literal",
                ));
            };
            validators.push(Validator::Contains(Contains::new(
                ValueOrPath::Value(lit),
                false,
            )));
        } else if meta.is_single_path(CONTAINS) {
            let content;
            parenthesized!(content in meta.input);
            let Ok(path) = content.parse::<syn::Path>() else {
                return Err(
                    meta.error("Invalid value given for `contains`, must be a literal or path")
                );
            };
            validators.push(Validator::Contains(Contains::new(
                ValueOrPath::Path(path),
                false,
            )))
        } else {
            let validation = parse_contains_full(&meta, false)?;
            validators.push(Validator::Contains(validation));
        }
        return Ok(());
    }

    if meta.path.is_ident(CONTAINS_NOT) {
        if meta.is_single_lit(CONTAINS_NOT) {
            let content;
            parenthesized!(content in meta.input);
            let Ok(lit) = content.parse::<syn::Lit>() else {
                return Err(meta.error(
                    "Invalid value given for `contains` validation, must be a path or literal",
                ));
            };
            validators.push(Validator::Contains(Contains::new(
                ValueOrPath::Value(lit),
                true,
            )));
        } else if meta.is_single_path(CONTAINS_NOT) {
            let content;
            parenthesized!(content in meta.input);
            let Ok(path) = content.parse::<syn::Path>() else {
                return Err(
                    meta.error("Invalid value given for `contains`, must be a literal or path")
                );
            };
            validators.push(Validator::Contains(Contains::new(
                ValueOrPath::Path(path),
                true,
            )))
        } else {
            let validation = parse_contains_full(&meta, true)?;
            validators.push(Validator::Contains(validation));
        }
        return Ok(());
    }

    if meta.path.is_ident(NON_CONTROL_CHAR) {
        if meta.is_full_pattern() {
            let validation = parse_non_control_char_full(&meta)?;
            validators.push(Validator::NonControlCharacter(validation))
        } else {
            validators.push(Validator::NonControlCharacter(NonControlChar::default()));
        }
        return Ok(());
    }

    if meta.path.is_ident(CUSTOM) {
        if meta.is_single_path(CUSTOM) {
            let content;
            parenthesized!(content in meta.input);
            let Ok(function) = content.parse::<syn::Path>() else {
                return Err(meta.error("Invalid value given for `custom` validation"));
            };
            validators.push(Validator::Custom(Custom::new(function)));
        } else {
            let validation = parse_custom_full(&meta)?;
            validators.push(Validator::Custom(validation));
        }
        return Ok(());
    }

    if meta.path.is_ident(REGEX) {
        if meta.is_single_path(REGEX) {
            let content;
            parenthesized!(content in meta.input);
            let Ok(path) = content.parse::<syn::Path>() else {
                return Err(
                    meta.error("Invalid value given for `regex` validation, must be a path")
                );
            };
            validators.push(Validator::Regex(Regex::new(path)));
        } else {
            let validation = parse_regex_full(&meta)?;
            validators.push(Validator::Regex(validation));
        }
        return Ok(());
    }

    if meta.path.is_ident(CREDIT_CARD) {
        if meta.is_full_pattern() {
            let validation = parse_credit_card_full(&meta)?;
            validators.push(Validator::CreditCard(validation));
        } else {
            validators.push(Validator::CreditCard(CreditCard::default()));
        }
        return Ok(());
    }

    if meta.path.is_ident(PHONE) {
        if meta.is_full_pattern() {
            let validation = parse_phone_full(&meta)?;
            validators.push(Validator::Phone(validation));
        } else {
            validators.push(Validator::Phone(Phone::default()));
        }
        return Ok(());
    }

    if meta.path.is_ident(REQUIRED) {
        if meta.is_full_pattern() {
            let validation = parse_required_full(&meta)?;
            validators.push(Validator::Required(validation));
        } else {
            validators.push(Validator::Required(Required::default()));
        }
        return Ok(());
    }

    if meta.path.is_ident(IS_IN) {
        if meta.is_single_path(IS_IN) || meta.is_single_lit(IS_IN) {
            let content;
            parenthesized!(content in meta.input);
            let Ok(expr) = content.parse::<syn::Expr>() else {
                return Err(meta.error("Invalid value provided"));
            };
            let mut validator = In::new(false);
            validator.expr = Some(expr);
            validators.push(Validator::In(validator));
        } else {
            let validation = parse_in_full(&meta, false)?;
            validators.push(Validator::In(validation));
        }
        return Ok(());
    }

    if meta.path.is_ident(NOT_IN) {
        if meta.is_single_path(NOT_IN) {
            let content;
            parenthesized!(content in meta.input);
            let Ok(expr) = content.parse::<syn::Expr>() else {
                return Err(meta.error("Invalid value provided"));
            };
            let mut validator = In::new(true);
            validator.expr = Some(expr);
            validators.push(Validator::In(validator));
        } else {
            let validation = parse_in_full(&meta, true)?;
            validators.push(Validator::In(validation));
        }
        return Ok(());
    }

    if meta.path.is_ident(IP) {
        if meta.is_full_pattern() {
            let validation = parse_ip_full(&meta)?;
            validators.push(Validator::Ip(validation));
        } else {
            validators.push(Validator::Ip(Ip::default()));
        }
        return Ok(());
    }

    if meta.path.is_ident(TIME) {
        let validation = parse_time(&meta)?;
        validators.push(Validator::Time(validation));
        return Ok(());
    }

    Err(meta.error("Unrecognized validate parameter"))
}
