#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]

mod error;
pub mod traits;
mod validation;

pub use error::{ValidationError, ValidationErrors};
pub use traits::{Contains, Length};
pub use validation::{
    cards::validate_credit_card,
    contains::validate_contains,
    email::validate_email,
    ip::{validate_ip, validate_ip_v4, validate_ip_v6},
    length::validate_length,
    non_control_char::validate_non_control_character,
    phone::validate_phone,
    range::validate_range,
    required::validate_required,
    time,
    urls::validate_url,
};
pub use validify_derive::{schema_err, schema_validation, Payload, Validate, Validify};

/// Validates the struct/enum based on the provided `#[validate]` attributes.
/// Deriving [Validate] allows you to specify schema and field validation on structs using the `#[validate]` attribute.
/// See the [repository](https://github.com/biblius/validify#validators) for a full list of possible validations.
pub trait Validate {
    /// Apply the provided validations to self
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// Modifies the struct/enum based on the provided `#[modify]` attributes.
/// Automatically implemented when deriving [Validify].
/// See the [repository](https://github.com/biblius/validify#modifiers) for a full list of possible modifiers.
pub trait Modify {
    /// Apply the provided modifiers to self
    fn modify(&mut self);
}

/// Validates and modifies the struct/enum based on the provided `#[validate]` and `#[modify]` attributes.
/// Deriving [Validify] allows you to modify structs before they are validated by providing
/// out of the box validation implementations as well as the ability to write custom ones.
///
/// ### Example
///
/// ```
/// use validify::Validify;
///
/// #[derive(Debug, Clone, serde::Deserialize, Validify)]
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
/// #[derive(Debug, Clone, serde::Deserialize, Validify)]
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
pub trait Validify: Modify + Validate {
    /// Apply the provided modifiers to self and run validations.
    fn validify(&mut self) -> Result<(), ValidationErrors>;
}

/// Exposes validify functionality on generated [Payload] structs.
pub trait ValidifyPayload: Sized {
    type Payload: Validate;

    /// Validates the payload then runs validations on the original struct, returning it
    /// if all validations pass.
    fn validate_from(payload: Self::Payload) -> Result<Self, ValidationErrors>;

    /// Validates the payload then runs modifications and validations on the original struct,
    /// returning it if all validations pass.
    fn validify_from(payload: Self::Payload) -> Result<Self, ValidationErrors>;
}

/// Creates a new field validation error.
/// Serves as a shorthand for writing out errors for custom functions
/// and schema validations.
/// Accepts:
///
/// - `("code")`
/// - `("code", "message")`
/// - `("field_name", "code", "message")`
///
/// ```rust
///  use validify::field_err;
///
///  let err = field_err!("foo");
///  assert_eq!(err.code(), "foo");
///  assert_eq!(err.location(), "");
///  assert!(err.message().is_none());
///  assert!(err.field_name().is_none());
///
///  let err = field_err!("foo", "bar");
///  assert_eq!(err.code(), "foo");
///  assert_eq!(err.location(), "");
///  assert_eq!(err.message().unwrap(), "bar");
///  assert!(err.field_name().is_none());
///
///  let err = field_err!("foo", "bar", "field");
///  assert_eq!(err.code(), "foo");
///  assert_eq!(err.message().unwrap(), "bar");
///  assert_eq!(err.field_name().unwrap(), "field");
///  assert_eq!(err.location(), "/field");
/// ```
#[macro_export]
macro_rules! field_err {
    ($code:literal) => {
        ::validify::ValidationError::new_field($code)
    };
    ($code:literal, $message:literal) => {
        ::validify::ValidationError::new_field($code).with_message($message.to_string())
    };
    ($code:literal, $message:literal, $field:literal) => {{
        let mut __e = ::validify::ValidationError::new_field_named($field, $code)
            .with_message($message.to_string());
        __e.set_location($field);
        __e
    }};
}
