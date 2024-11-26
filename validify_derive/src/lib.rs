use proc_macro_error::proc_macro_error;
use quote::{quote, ToTokens};
use syn::{parse::Parse, ItemFn, LitStr, Token};

mod fields;
mod payload;
mod serde;
mod tokens;
mod validate;
mod validify;

/// Implements `Validate` and `Modify` in one trait.
///
/// Deriving this will allow you to annotate fields with `modify` attributes, in addition to `validate`.
/// Modifiers are simple functions that modify the struct before validation.
/// You can use the few out of the box ones or create your own.
///
/// Visit the [repository](https://github.com/biblius/validify) to see the list of available validations and
/// modifiers, as well as more examples.
///
///  ### Example
///
/// ```ignore
/// use validify::{Validify, Payload};
///
/// #[derive(Debug, Clone, serde::Deserialize, Validify, Payload)]
/// struct Testor {
///     #[modify(lowercase, trim)]
///     #[validate(length(equal = 8))]
///     pub a: String,
///     #[modify(trim, uppercase)]
///     pub b: Option<String>,
///     #[modify(custom(do_something))]
///     pub c: String,
///     #[modify(custom(do_something))]
///     pub d: Option<String>,
///     #[validify]
///     pub nested: Nestor,
/// }
///
/// #[derive(Debug, Clone, serde::Deserialize, Validify, Payload)]
/// struct Nestor {
///     #[modify(trim, uppercase)]
///     #[validate(length(equal = 12))]
///     a: String,
///     #[modify(capitalize)]
///     #[validate(length(equal = 14))]
///     b: String,
/// }
///
/// fn do_something(input: &mut String) {
///     *input = String::from("modified");
/// }
///
/// let mut test = Testor {
///   a: "   LOWER ME     ".to_string(),
///   b: Some("  makemeshout   ".to_string()),
///   c: "I'll never be the same".to_string(),
///   d: Some("Me neither".to_string()),
///   nested: Nestor {
///     a: "   notsotinynow   ".to_string(),
///       b: "capitalize me.".to_string(),
///   },
/// };
///
/// // The magic line
/// let res = test.validify();
///
/// assert!(matches!(res, Ok(_)));
///
/// // Parent
/// assert_eq!(test.a, "lower me");
/// assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
/// assert_eq!(test.c, "modified");
/// assert_eq!(test.d, Some("modified".to_string()));
/// // Nested
/// assert_eq!(test.nested.a, "NOTSOTINYNOW");
/// assert_eq!(test.nested.b, "Capitalize me.");
/// ```
#[proc_macro_derive(Validify, attributes(modify, validate, validify))]
#[proc_macro_error]
pub fn derive_validify(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    validify::r#impl::impl_validify(&ast).into()
}

/// Enables the use of `#[validate]` attributes on struct/enum fields and
/// implements the `Validate` trait based on the provided attributes.
///
/// Visit the [repository](https://github.com/biblius/validify) to see the list of available validations and
/// modifiers, as well as more examples.
///
/// ### Example
///
/// ```ignore
/// use validify::{ValidationError, ValidationErrors, Validate};
///
/// #[derive(Debug, Validate, Deserialize)]
/// #[validate(validate_signup)]
/// struct SignupData {
///     #[validate(email)]
///     mail: String,
///     #[validate(url)]
///     site: String,
///     #[validate(
///         length(min = 1),
///         custom(validate_unique_username)
///     )]
///     first_name: String,
///     #[validate(range(min = 18., max = 20.))]
///     age: u32,
/// }
///
/// fn validate_unique_username(username: &str) -> Result<(), ValidationError> {
///     if username == "xXxShad0wxXx" {
///         return Err(ValidationError::new_field(
///             "terrible_username",
///         ));
///     }
///
///     Ok(())
/// }
///
/// fn validate_signup(data: &SignupData) -> Result<(), ValidationErrors> {
///     let mut errs = ValidationErrors::new();
///     if data.mail.ends_with("gmail.com") && data.age == 18 {
///         errs.add(ValidationError::new_schema("stupid_rule"));
///     }
///     if errs.is_empty() {
///         return Ok(());
///     }
///     Err(errs)
/// }
///
/// let signup = SignupData {
///     mail: "bob@bob.com".to_string(),
///     site: "http://hello.com".to_string(),
///     first_name: "Bob".to_string(),
///     age: 18,
/// };
///     
/// assert!(signup.validate().is_ok());
/// ```
#[proc_macro_derive(Validate, attributes(validate))]
#[proc_macro_error]
pub fn derive_validate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse(input).unwrap();
    validate::r#impl::impl_validate(&input).into()
}

/// A shortcut for ergonomic error creation in custom schema validaton functions.
///
/// Prepends a `let mut errors = ValidationErrors::new()` to the beginning of the function block,
/// and appends a `if errors.is_empty() { Ok(()) } else { Err(errors) }` to the end.
///
/// Designed to be used in conjuction with the `schema_err` macro.
///
/// ```ignore
/// use validify::{ValidationErrors, Validify, schema_validation, schema_err};
///
/// #[derive(Debug, Clone, Validify)]
/// #[validate(validate_schema)]
/// struct Foo {
///     a: String,
///     b: usize,
/// }
///
/// #[schema_validation]
/// fn validate_schema(foo: &Foo) -> Result<(), ValidationErrors> {
///     if foo.b == 0 && foo.a == "no" {
///         schema_err("super no", "Done goofd");
///     }
/// }
/// ```
///
/// `validate_schema` Desugars to:
///
/// ```ignore
/// fn validate_schema(foo: &Foo) -> Result<(), ValidationErrors> {
///     let mut errors = validify::ValidationErrors::new();
///     if foo.b == 0 && foo.a == "no" {
///         errors.add(validify::ValidationError::new_schema("super no").with_message("Done goofd".to_string()));
///     }
///     if errors.is_empty() { Ok(()) } else { Err(errors) }
/// }
/// ```
#[proc_macro_attribute]
pub fn schema_validation(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut func: ItemFn = syn::parse(input).unwrap();

    // Add error and return value
    let start_tokens = syn::parse(
        quote! {
            let mut errors = ::validify::ValidationErrors::new();
        }
        .into(),
    )
    .unwrap();

    func.block.stmts.insert(0, start_tokens);

    let return_tokens = syn::parse(
        quote!(if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        })
        .into(),
    )
    .unwrap();

    func.block.stmts.push(return_tokens);
    func.to_token_stream().into()
}

/// Generates a struct with the same structure of the implementing struct
/// with all its fields as options. This can only be used on struct that implement `Validify`.
/// Any nested structs must also contain their corresponding payloads.
///
/// The payload struct is Deserializable, has `From` and `Into` impls for
/// the original, and implements `Validate`.
///
/// The original struct gets a `ValidatePayload` implementation with 2 associated functions;
///
/// `validify_from` which will validate the payload and call `Validify` on the original,
///
/// and
///
/// `validate_from` which does the same, but calls `Validate` instead of `Validify`
/// on the original.
///
/// Both functions return the original struct if the validation succeeds.
///
/// The payload can be used to represent a completely deserializable version of the struct
/// even when some fields are missing.
///
/// This can be used for more detailed descriptions of what fields are missing, along
/// with any other validation errors.
///
/// Example:
///
/// ```ignore
/// #[derive(Debug, Clone, serde::Deserialize, validify::Validify, validify::Payload)]
/// struct Data {
///     a: String,
///     b: Option<String>
/// }
/// ```
///
/// Expands to:
///
/// ```ignore
/// #[derive(Debug, validify::Validate, serde::Deserialize)]
/// struct DataPayload {
///     #[validate(required)]
///     a: Option<String>,
///     b: Option<String>
/// }
/// ```
#[proc_macro_derive(Payload)]
#[proc_macro_error]
pub fn derive_payload(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse(input).unwrap();
    payload::r#impl::impl_payload(&input).into()
}

/// Designed to be used with the [schema_validation] macro.
/// Used for ergonomic custom error handling.
///
/// Adds a schema validaton error to the generated `ValidationErrors`.
///
/// The errors argument should pass in an instance of `ValidationErrors`,
/// and usually is used with the one generated from `schema_validation`.
///
/// Schema errors always have a location of "/".
///
/// Accepts:
///
/// `("code")`
/// `("code", "custom message")`
#[proc_macro]
pub fn schema_err(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let SchemaErr { code, message } = syn::parse(input).expect("invalid tokens");
    let message = message.map(|m| quote!(.with_message(#m.to_string())));
    quote!(
        errors.add(::validify::ValidationError::new_schema(#code) #message);
    )
    .into()
}

struct SchemaErr {
    code: LitStr,
    message: Option<LitStr>,
}

impl Parse for SchemaErr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let code = input.parse()?;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        if input.is_empty() {
            return Ok(SchemaErr {
                code,
                message: None,
            });
        }

        let message = input.parse()?;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(SchemaErr { code, message })
    }
}
