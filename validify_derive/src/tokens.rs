use crate::validate::parser::ValueOrPath;
use crate::validate::validation::{
    Contains, CreditCard, Custom, Describe, Email, In, Ip, Length, NonControlChar, Phone, Range,
    Regex, Required, SchemaValidation, Time, TimeMultiplier, Url, Validator,
};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::spanned::Spanned;

/// Utility for generating error messages
pub trait ValidationErrorTokens {
    /// Quote an error based on the validation's settings
    fn quote_error(&self, field_name: &str) -> proc_macro2::TokenStream
    where
        Self: Describe,
    {
        let message = if let Some(ref m) = self.message() {
            quote!(err.set_message(String::from(#m));)
        } else {
            quote!()
        };

        let code = self.code();

        quote!(
            let mut err = ::validify::ValidationError::new_field(#code);
            err.set_field(#field_name);
            #message
        )
    }
}

macro_rules! impl_error {
    ($($id:ident),*) => (
        $(
            impl ValidationErrorTokens for $id {}
        )*
    )
}

impl_error! {
    Length,
    Range,
    Email,
    Url,
    CreditCard,
    Phone,
    Custom,
    NonControlChar,
    Required,
    Regex,
    Contains,
    Time,
    In,
    Ip
}

/// Whether the tokens are for nested or direct validations.
pub enum ValidationTokens {
    Normal(proc_macro2::TokenStream),
    Nested(proc_macro2::TokenStream),
}

/// Output the necessary tokens for schema validation when implementing `Validate`.
pub fn quote_schema_validation(validation: &[SchemaValidation]) -> Vec<proc_macro2::TokenStream> {
    validation
        .iter()
        .map(|v| {
            let fn_ident = &v.function;
            quote!(
                if let Err(mut errs) = #fn_ident(&self) {
                        errors.merge(errs);
                };
            )
        })
        .collect()
}

impl Validator {
    pub fn to_validate_tokens(
        &self,
        field_info: &crate::fields::FieldInfo,
        validator_param: TokenStream,
    ) -> ValidationTokens {
        let field_name = field_info.name();

        match self {
            Validator::Email(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Url(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::CreditCard(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Phone(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Ip(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Custom(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Range(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Length(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::NonControlCharacter(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Regex(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Contains(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Required(v) => {
                let validator_param = field_info
                    .ident_override
                    .as_ref()
                    .map(|ident| quote!(#ident))
                    .unwrap_or_else(|| {
                        let ident = field_info.field.ident.as_ref();
                        quote!(&self.#ident)
                    });
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(tokens)
            }
            Validator::In(v) => {
                let validator_param = field_info
                    .ident_override
                    .as_ref()
                    .map(|ident| quote!(#ident))
                    .unwrap_or_else(|| {
                        let ident = field_info.field.ident.as_ref();
                        quote!(&self.#ident)
                    });

                let tokens = v.to_validify_tokens(
                    field_name,
                    validator_param,
                    field_info.is_option(),
                    false,
                );
                ValidationTokens::Normal(tokens)
            }
            Validator::Time(v) => {
                let tokens = v.to_validify_tokens(field_name, validator_param, false);
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Iter(v) => {
                let validator_param = quote!(el);
                let inner_tokens = v.iter().map(|v| match v {
                    Validator::Iter(_) => {
                        abort!(field_info.field.span(), "`iter` validator cannot be nested.")
                    }
                    Validator::Nested => {
                        abort!(field_info.field.span(), "`validate/validify` is not valid in `iter`. To validate collections of type T, use `validate/validify` directly on the field.")
                    },
                    Validator::Email(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Url(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::CreditCard(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Phone(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Custom(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Range(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Length(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Ip(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::NonControlCharacter(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Required(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Regex(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Contains(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    }
                    Validator::Time(v) => {
                        v.to_validify_tokens(field_name.clone(), validator_param.clone(), true)
                    },
                    Validator::In(v) => v.to_validify_tokens(field_name.clone(), validator_param.clone(), false, true),
                });
                let validator_param = field_info
                    .ident_override
                    .as_ref()
                    .map(|ident| quote!(#ident))
                    .unwrap_or_else(|| {
                        let ident = field_info.field.ident.as_ref();
                        if field_info.is_option() {
                            return quote!(#ident);
                        }
                        quote!(self.#ident)
                    });

                let tokens = quote!(
                    for (__i, el) in #validator_param.iter().enumerate() {
                        #(#inner_tokens)*
                    }
                );
                ValidationTokens::Normal(field_info.wrap_tokens_if_option(tokens))
            }
            Validator::Nested => {
                let validator_field = field_info
                    .ident_override
                    .as_ref()
                    .map(|id| quote!(#id))
                    .unwrap_or_else(|| {
                        let ident = &field_info.field.ident;

                        if field_info.is_option() || field_info.is_list() || field_info.is_map() {
                            quote!(#ident)
                        } else {
                            quote!(self.#ident)
                        }
                    });

                let field_name = field_info.name();

                let tokens = quote!(
                    if let Err(mut errs) = #validator_field.validate() {
                        errs.errors_mut().iter_mut().for_each(|err| err.set_location(#field_name));
                        errors.merge(errs);
                    }
                );
                ValidationTokens::Nested(field_info.wrap_tokens_if_option(
                    field_info.wrap_validator_if_collection(validator_field, tokens),
                ))
            }
        }
    }
}

impl Ip {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };

        let Ip { ref format, .. } = self;

        let validate_fn = match format {
            Some(format) => match format {
                crate::validate::validation::IpFormat::V4 => quote!(validate_ip_v4),
                crate::validate::validation::IpFormat::V6 => quote!(validate_ip_v6),
            },
            None => quote!(validate_ip),
        };

        quote!(
            if !::validify::#validate_fn(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                #error_location
                errors.add(err);
            }
        )
    }
}

impl Length {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let Length {
            ref min,
            ref max,
            ref equal,
            ..
        } = self;

        let quoted_error = self.quote_error(&field_name);
        let error_param =
            quote!(err.add_param("actual", &::validify::Length::length(&#validator_param)););
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };

        let min_err_param_quoted = if let Some(ref v) = min {
            quote!(err.add_param("min", &#v);)
        } else {
            quote!()
        };

        let max_err_param_quoted = if let Some(ref v) = max {
            quote!(err.add_param("max", &#v);)
        } else {
            quote!()
        };

        let equal_err_param_quoted = if let Some(ref v) = equal {
            quote!(err.add_param("equal", &#v);)
        } else {
            quote!()
        };

        let min_tokens = min
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as usize)))
            .unwrap_or(quote!(None));

        let max_tokens = max
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as usize)))
            .unwrap_or(quote!(None));

        let equal_tokens = equal
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as usize)))
            .unwrap_or(quote!(None));

        quote!(
            if !::validify::validate_length(
                #validator_param,
                #min_tokens,
                #max_tokens,
                #equal_tokens
            ) {
                #quoted_error
                #min_err_param_quoted
                #max_err_param_quoted
                #equal_err_param_quoted
                #error_param
                #error_location
                errors.add(err);
            }
        )
    }
}

impl Range {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };

        let Range {
            ref min, ref max, ..
        } = self;

        let min_err_param_quoted = if let Some(v) = min {
            quote!(err.add_param("min", &#v);)
        } else {
            quote!()
        };

        let max_err_param_quoted = if let Some(v) = max {
            quote!(err.add_param("max", &#v);)
        } else {
            quote!()
        };

        let min_tokens = min
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as f64)))
            .unwrap_or(quote!(None));

        let max_tokens = max
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as f64)))
            .unwrap_or(quote!(None));

        quote!(
            if !::validify::validate_range(
                *#validator_param as f64,
                #min_tokens,
                #max_tokens
            ) {
                #quoted_error
                #min_err_param_quoted
                #max_err_param_quoted
                err.add_param("actual", &#validator_param);
                #error_location
                errors.add(err);
            }
        )
    }
}

impl CreditCard {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };
        quote!(
            if !::validify::validate_credit_card(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                #error_location
                errors.add(err);
            }
        )
    }
}

impl Phone {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };
        quote!(
            if !::validify::validate_phone(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                #error_location
                errors.add(err);
            }
        )
    }
}

impl Custom {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let Custom { ref path, .. } = self;

        let err_with_msg = if let Some(msg) = self.message() {
            quote!(err.with_message(#msg.to_string()))
        } else {
            quote!(err)
        };
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };

        quote!(
            if let Err(mut err) = #path(#validator_param) {
                let f_name = err.field_name().map(|s|s.to_string());
                if let Some(field_name) = f_name {
                    err.set_location(field_name);
                } else {
                    err.set_field(#field_name);
                    #error_location
                }
                errors.add(#err_with_msg);
            };
        )
    }
}

impl NonControlChar {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };
        quote!(
            if !::validify::validate_non_control_character(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                #error_location
                errors.add(err);
            }
        )
    }
}

impl Url {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };
        quote!(
            if !::validify::validate_url(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                #error_location
                errors.add(err);
            }
        )
    }
}

impl Email {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };
        quote!(
            if !::validify::validate_email(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                #error_location
                errors.add(err);
            }
        )
    }
}

impl Regex {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let Regex { ref path, .. } = self;
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };

        quote!(
            if !#path.is_match(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                #error_location
                errors.add(err);
            }
        )
    }
}

impl In {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        is_option: bool,
        in_iter: bool,
    ) -> TokenStream {
        let In { ref expr, not, .. } = self;

        let quoted_error = self.quote_error(&field_name);

        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };

        if is_option {
            quote!(
                if let Some(ref param) = #validator_param {
                    if !::validify::validate_contains(&#expr, &param, #not) {
                        #quoted_error
                        #error_location
                        errors.add(err);
                    }
                }
            )
        } else {
            quote!(
                if !::validify::validate_contains(&#expr, &#validator_param, #not) {
                    #quoted_error
                    #error_location
                    errors.add(err);
            })
        }
    }
}

impl Contains {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let Contains { not, ref value, .. } = self;

        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };

        // Only add the target if it's a literal since otherwise it will just be the variable name
        let added_param = matches!(value, Some(ValueOrPath::Value(_)))
            .then_some(quote!(err.add_param("target", &#value);));

        quote!(
            if !::validify::validate_contains(#validator_param, &#value, #not) {
                #quoted_error
                #added_param
                #error_location
                errors.add(err);
            }
        )
    }
}

impl Required {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };
        quote!(
            if !::validify::validate_required(#validator_param) {
                #quoted_error
                #error_location
                errors.add(err);
            }
        )
    }
}

impl Time {
    fn to_validify_tokens(
        &self,
        field_name: String,
        validator_param: TokenStream,
        in_iter: bool,
    ) -> TokenStream {
        let quoted_error = self.quote_error(&field_name);
        let error_location = if in_iter {
            quote!(err.set_location_idx(__i, #field_name);)
        } else {
            quote!(err.set_location(#field_name);)
        };

        let Time {
            op,
            inclusive,
            has_time,
            multiplier,
            ref format,
            ref duration,
            ref target,
            ..
        } = self;
        let has_time = *has_time;

        let code = self.code();
        let quoted_parse_error = quote!(
            let mut err = ::validify::ValidationError::new_field(#code);
            err.set_field(#field_name);
            err.add_param("actual", &#validator_param);
            err.add_param("format", &#format);
            #error_location
            errors.add(err);
        );

        let duration = duration.as_ref().map(|duration| match duration {
            // The value will be in seconds due to the way it is parsed in parse_time [crate::validate::parser::parse_time]
            ValueOrPath::Value(val) => quote!(chrono::Duration::seconds(#val)),
            ValueOrPath::Path(path) => match multiplier {
                TimeMultiplier::Seconds => quote!(chrono::Duration::seconds(#path)),
                TimeMultiplier::Minutes => quote!(chrono::Duration::minutes(#path)),
                TimeMultiplier::Hours => quote!(chrono::Duration::hours(#path)),
                TimeMultiplier::Days => quote!(chrono::Duration::days(#path)),
                TimeMultiplier::Weeks => quote!(chrono::Duration::weeks(#path)),
                TimeMultiplier::None => unreachable!(),
            },
        });

        use crate::validate::validation::TimeOp as TO;
        let validation_fn = match op {
            TO::BeforeNow => {
                if has_time {
                    quote!(::validify::time::before_now(#validator_param, #inclusive))
                } else {
                    quote!(::validify::time::before_today(#validator_param, #inclusive))
                }
            }
            TO::AfterNow => {
                if has_time {
                    quote!(::validify::time::after_now(#validator_param, #inclusive))
                } else {
                    quote!(::validify::time::after_today(#validator_param, #inclusive))
                }
            }
            TO::BeforeFromNow => {
                if has_time {
                    quote!(::validify::time::before_from_now(#validator_param, #duration))
                } else {
                    quote!(::validify::time::before_from_now_date(#validator_param, #duration))
                }
            }
            TO::AfterFromNow => {
                if has_time {
                    quote!(::validify::time::after_from_now(#validator_param, #duration))
                } else {
                    quote!(::validify::time::after_from_now_date(#validator_param, #duration))
                }
            }
            TO::Before => {
                let validation = {
                    let validation_fn = if has_time {
                        quote!(!::validify::time::before(#validator_param, target, #inclusive))
                    } else {
                        quote!(!::validify::time::before_date(#validator_param, target, #inclusive))
                    };
                    quote!(
                        if #validation_fn {
                            #quoted_error
                            err.add_param("actual", &#validator_param);
                            err.add_param("target", target);
                            #error_location
                            errors.add(err);
                        }
                    )
                };
                return quote_time_with_target(
                    target.as_ref().unwrap(),
                    validation,
                    quoted_parse_error,
                    format.as_deref(),
                    has_time,
                );
            }
            TO::After => {
                let validation = {
                    let validation_fn = if has_time {
                        quote!(!::validify::time::after(#validator_param, target, #inclusive))
                    } else {
                        quote!(!::validify::time::after_date(#validator_param, target, #inclusive))
                    };
                    quote!(
                        if #validation_fn {
                            #quoted_error
                            err.add_param("actual", &#validator_param);
                            err.add_param("target", target);
                            #error_location
                            errors.add(err);
                        }
                    )
                };
                return quote_time_with_target(
                    target.as_ref().unwrap(),
                    validation,
                    quoted_parse_error,
                    format.as_deref(),
                    has_time,
                );
            }
            TO::InPeriod => {
                let validation = {
                    let validation_fn = if has_time {
                        quote!(!::validify::time::in_period(#validator_param, target, #duration))
                    } else {
                        quote!(!::validify::time::in_period_date(#validator_param, target, #duration))
                    };

                    // We can safely unwrap since we do a check for overflow before quoting
                    quote!(
                        if #validation_fn {
                            #quoted_error
                            err.add_param("actual", &#validator_param);
                            let end = target.checked_add_signed(#duration).unwrap();
                            if end < *target {
                                err.add_param("from", &end);
                                err.add_param("to", target);
                            } else {
                                err.add_param("from", target);
                                err.add_param("to", &end);
                            }
                            #error_location
                            errors.add(err);
                        }
                    )
                };
                return quote_time_with_target(
                    target.as_ref().unwrap(),
                    validation,
                    quoted_parse_error,
                    format.as_deref(),
                    has_time,
                );
            }
            TO::None => unreachable!(),
        };

        quote!(
            if !#validation_fn {
                #quoted_error
                err.add_param("actual", &#validator_param);
                #error_location
                errors.add(err);
            }
        )
    }
}

/// Quotes the tokens based on the target `value_or_path`. If the target is a string,
/// it will be parsed based on the format.
fn quote_time_with_target(
    value_or_path: &ValueOrPath<String>,
    quoted_validation: proc_macro2::TokenStream,
    quoted_parse_error: proc_macro2::TokenStream,
    format: Option<&str>,
    has_time: bool,
) -> proc_macro2::TokenStream {
    match value_or_path {
        ValueOrPath::Value(value) => {
            let format = format.unwrap();
            let parse_fn = if has_time {
                quote!(chrono::NaiveDateTime::parse_from_str(#value, #format))
            } else {
                quote!(chrono::NaiveDate::parse_from_str(#value, #format))
            };
            quote!(
                if let Ok(ref target) = #parse_fn {
                    #quoted_validation
                } else {
                    #quoted_parse_error
                }
            )
        }
        ValueOrPath::Path(target) => {
            quote!(
                let target = &#target();
                #quoted_validation
            )
        }
    }
}
