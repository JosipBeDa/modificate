# Validify

[![Build](https://img.shields.io/github/actions/workflow/status/biblius/validify/check.yml?logo=github&style=plastic)](https://github.com/biblius/validify)
[![test](https://img.shields.io/github/actions/workflow/status/biblius/validify/test.yml?label=test&logo=github&style=plastic)](https://github.com/biblius/validify)
[![coverage](https://img.shields.io/codecov/c/github/biblius/validify?color=%23383&logo=codecov&style=plastic)](https://app.codecov.io/gh/biblius/validify/tree/master)
[![docs](https://img.shields.io/docsrs/validify?logo=rust&style=plastic)](https://docs.rs/validify/latest/validify/)
[![version](https://img.shields.io/crates/v/validify?logo=rust&style=plastic)](https://crates.io/crates/validify)

Procedural macros that provide attributes for data validation and modification.

## Attributes

### **Modifiers**

| Modifier   | Type                                 | Description                                                                                                                                          |
| ---------- | ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| trim       | `String / Vec<String>`               | Removes surrounding whitespace / in each string in the iterator                                                                                      |
| uppercase  | `String / Vec<String>`               | Calls `.to_uppercase()` / in each string in the iterator                                                                                             |
| lowercase  | `String / Vec<String>`               | Calls `.to_lowercase()` / in each string in the iterator                                                                                             |
| capitalize | `String / Vec<String>`               | Makes the first char of the string uppercase / in each string in the iterator                                                                        |
| custom     | `T`                                  | Takes a function whose argument is `&mut <Type>`                                                                                                     |
| validify   | `impl Validify / Vec<impl Validify>` | Can only be used on fields that are structs (or vecs of) implementing the `Validify` trait. Runs all the child's struct's modifiers and validations. |

### **Validators**

The syntax is either

- `#[validate(<Validator>)]` if `Parameters` is `--`,

- `#[validate(<Validator>( [param = val1 ]+ )` if `Parameters` is specified.

All validators also take in a `code` and `message` as parameters and their values are must be string literals if specified.

All validators are valid on their respective `Option` types. Fields only get validated if they are `Some`.

| Validator        | Field type         | Parameters         | Parameter type | Description                                                                                                                     |
| ---------------- | ------------------ | ------------------ | -------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| validate         | `impl Validate`    | --                 | --             | Calls the child struct's `validate` implementation.                                                                             |
| email            | `String`           | --                 | --             | Checks emails based on [this spec](https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address).                     |
| credit_card      | `String`           | --                 | --             | Checks if the field's value is a valid credit card number.                                                                      |
| phone            | `String`           | --                 | --             | Checks if the field's value is a valid phone number.                                                                            |
| required         | `Option\<T>`       | --                 | --             | Checks whether the field's value is `Some`.                                                                                     |
| url              | `String`           | --                 | --             | Checks if the string is a URL.                                                                                                  |
| non_control_char | `String`           | --                 | --             | Checks if the field contains control characters.                                                                                |
| ip               | `String`           | format             | Ident (v4/v6)  | Checks if the string is an IP address.                                                                                          |
| length           | `impl Length`      | min, max, equal    | LitInt         | Checks if the collection length is within the specified params. Works via the `Length` trait.                                   |
| range            | `Int/Float`        | min, max           | LitFloat       | Checks if the value is in the specified range.                                                                                  |
| contains         | `impl Contains`    | value              | Lit/Path       | Checks if the collection contains the specified value. Works via the `Contains` trait.                                          |
| contains_not     | `impl Contains`    | value              | Lit/Path       | Checks if the collection doesn't contain the specified value. Works via the `Contains` trait.                                   |
| custom           | `T`                | function           | Path           | Executes custom validation on the field by calling the provided function.                                                       |
| regex            | `String`           | path               | Path           | Matches the provided regex against the field. Intended to be used with lazy_static by providing a path to an initialised regex. |
| is_in            | `impl Contains`    | collection         | Path           | Checks whether the field's value is in the specified collection.                                                                |
| not_in           | `impl Contains`    | collection         | Path           | Checks whether the field's value is not in the specified collection.                                                            |
| iter             | `impl Iterator`    | List of validators | Validator      | Runs the provided validators on each element of the iterable.                                                                   |
| time             | `NaiveDate\[Time]` | See below          | See below      | Performs a check based on the specified op.                                                                                     |

### **Time operators**

All time operators may take in `inclusive = bool`.
All time operator must take in `time = bool` when validating datetimes, by default time validators will attempt to validate dates.

`in_period` and the `*_from_now` operators are inclusive by default.

The `target` param must be a string literal date or a path to an argless function that returns a date\[time].

If the target is a string literal, it must contain a `format` param, as per [this](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

Accepted interval parameters are `seconds`, `minutes`, `hours`, `days`, `weeks`.

The `_from_now` operators should not use negative duration due to how they validate the inputs,
negative duration for `in_period` works fine.

| Op              | Parameters       | Description                                                                      |
| --------------- | ---------------- | -------------------------------------------------------------------------------- |
| before          | target           | Check whether a date\[time] is before the target one                             |
| after           | target           | Check whether a date\[time] is after the target one                              |
| before_now      | --               | Check whether a date\[time] is before today\[now]                                |
| after_now       | --               | Check whether a date\[time] is after today\[now]                                 |
| before_from_now | interval         | Check whether a date\[time] is before the specified interval from today\[now]    |
| after_from_now  | interval         | Check whether a date\[time] is after the specified interval from the today\[now] |
| in_period       | target, interval | Check whether a date\[time] falls within a certain period                        |

## Derive

Annotate the struct or enum you want to modify and/or validate with the `Validify` attribute
(if you do not need modification, derive only `Validate`) and call its `validate/validify` function:

```rust
use validify::Validify;

#[derive(Debug, Clone, serde::Deserialize, Validify)]
struct Testor {
    #[modify(lowercase, trim)]
    #[validate(length(equal = 8))]
    pub a: String,
    #[modify(trim, uppercase)]
    pub b: Option<String>,
    #[modify(custom(do_something))]
    pub c: String,
    #[modify(custom(do_something))]
    pub d: Option<String>,
    #[validify]
    pub nested: Nestor,
}

#[derive(Debug, Clone, serde::Deserialize, Validify)]
struct Nestor {
    #[modify(trim, uppercase)]
    #[validate(length(equal = 12))]
    a: String,
    #[modify(capitalize)]
    #[validate(length(equal = 14))]
    b: String,
}

fn do_something(input: &mut String) {
    *input = String::from("modified");
}

let mut test = Testor {
  a: "   LOWER ME     ".to_string(),
  b: Some("  makemeshout   ".to_string()),
  c: "I'll never be the same".to_string(),
  d: Some("Me neither".to_string()),
  nested: Nestor {
    a: "   notsotinynow   ".to_string(),
      b: "capitalize me.".to_string(),
  },
};

// The magic line
let res = test.validify();

assert!(matches!(res, Ok(_)));

// Parent
assert_eq!(test.a, "lower me");
assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
assert_eq!(test.c, "modified");
assert_eq!(test.d, Some("modified".to_string()));
// Nested
assert_eq!(test.nested.a, "NOTSOTINYNOW");
assert_eq!(test.nested.b, "Capitalize me.");
```

## Traits

Validify is built around 3 traits:

- Validate
- Modify
- Validify (Validate + Modify)

These traits should theoretically never have to be implemented manually.

As their names suggest, the first two traits perform validation and modification, while the third combines those 2 actions into a single one - `validify`.

The traits contain a single function which is constructed based on field annotations when deriving them.

## Payload

Structs annotated with `#[derive(Payload)]` get an associated payload struct, e.g.

```rust
#[derive(validify::Validify, validify::Payload)]
struct Something {
  a: usize,
  b: String,
  c: Option<bool>
}
```

behind the scenes will generate an intermediary

```rust
#[derive(Debug, Clone, serde::Deserialize, validify::Validate)]
struct SomethingPayload {
  #[validate(required)]
  a: Option<usize>,
  #[validate(required)]
  b: Option<String>,
  c: Option<bool>,

  /* From and Into impls */
}
```

The motivation for this is to aid in deserializing potentially missing fields. Even though the payload struct cannot help with deserializing wrong types, it can still prove useful and provide a bit more meaningful error messages when fields are missing.

The original struct gets a `ValidifyPayload` implementation with 2 associated fns: `validate_from` and `validify_from` whose whose respective arguments are the generated payload.

The `ValidifyPayload` implementations first validate the required fields of the payload. Then, if any required fields are missing, no further modification/validation is done and the errors are returned. Next, the payload is transformed to the original struct and modifications and/or validations are run on it.

When a struct contains nested validifies (child structs annotated with `#[validify]`), all the children in the payload will also be transformed and validated as payloads first. This means that any nested structs must also derive `Payload`.

The `Payload` derive macro does not work on enums.

## The payload and serde

Struct level attributes, such as `rename_all` are propagated to the payload. When attributes that modify field names are present, any field names in returned errors will be represented as the original (i.e. client payload).

There are a few special serde attributes that validify treats differently; `rename`, `with` and `deserialize_with`.
It is **highly** advised these attributes are kept in a separate annotation from any other serde attributes, due to the way
they are parsed for the payload.

The `rename` attribute is used by validify to set the field name in any errors during validation. The `with` and `deserialize_with` will be transfered to the payload field and will create a special deserialization function that will call the original and wrap the result in an option. If the custom deserializer already returns an option, it will do nothing.

## Schema validation

Schema level validation can be performed using the following:

```rust
use validify::{Validify, ValidationErrors, schema_validation, schema_err};
#[derive(validify::Validify)]
#[validate(validate_testor)]
struct Testor {
    a: String,
    b: usize,
}

#[schema_validation]
fn validate_testor(t: &Testor) -> Result<(), ValidationErrors> {
  if t.a.as_str() == "yolo" && t.b < 2 {
    schema_err!("Invalid Yolo", "Cannot yolo with b < 2");
  }
}
```

The `#[schema_validation]` proc macro expands the function to:

```rust, ignore
fn validate_testor(t: &Testor) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();
    if t.a == "yolo" && t.b < 2 {
        errors.add(ValidationError::new_schema("Invalid Yolo").with_message("Cannot yolo with b < 2".to_string()));
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

This makes schema validations a bit more ergonomic and concise.
Like field level validation, schema level validation is performed after modification.

When you have annotated a function with `#[schema_validation]`, you can use the `schema_err!` macro to ergonomically
create schema errors.

## Errors

The main ValidationError is an enum with 2 variants, Field and Schema. Field errors are, as the name suggests, created when fields fail validation and are usually automatically generated unless using custom handlers (custom field validation functions always must return a result whose Err variant is ValidationError).

If you want to provide a message along with the error, you can directly specify it in the attribute (the same goes for the code),
for example:

`#[validate(contains(value = "something", message = "Does not contain something", code = "MUST_CONTAIN"))]`

Keep in mind, when specifying validations this way, all attribute parameters MUST be specified as [NameValue](https://docs.rs/syn/latest/syn/struct.MetaNameValue.html) pairs. This means that if you write

`#[validate(contains("something", message = "Bla"))]`,

you will get an error because the parser expects either a single value or multiple name value pairs.

The [field_err!](https://docs.rs/validify/latest/validify/macro.field_err.html) macro provides a more ergonomic way to create field errors.

### Location

Locations are tracked for each error in a similar manner to [JSON pointers](https://opis.io/json-schema/2.x/pointers.html).
When using custom validation, whatever field name you specify in the returned error will be used in the location for that field.
Keep in mind locations are not reliable when dealing with hashed map/set collections as the item ordering for those is not guaranteed.

Error location display will depend on the original client payload, i.e.
they will be displayed in the original case the payload was received (e.g. when using serde's `rename_all` or `rename` attributes).
Any overriden field names will be displayed as such.

### Schema

Schema errors are usually created by the user in schema validation.
The `schema_err!` macro alongside `#[schema_validation]` provides an ergonomic way to create schema errors.
All errors are composed to a `ValidationErrors` struct which contains a vec of all the validation errors.
All schema errors generated by `validify` will have their location set to '/'.

### Params

When sensible, validify automatically appends failing parameters and the target values they were validated against to the errors created to provide more clarity to the client and to save some manual work.

One parameter that is often appended is the `actual` field which represents the value of the violating field's target property during the validation. Some validators append additional data to the errors representing the expected values for the field.

## **Examples**

### **Date\[times]s**

```rust, ignore
use chrono::{NaiveDate, NaiveDateTime};

#[derive(Debug, validify::Validate)]
struct DateTimeExamples {
    #[validate(time(op = before, target = "2500-04-20", format = "%Y-%m-%d", inclusive = true))]
    before: NaiveDate,
    #[validate(time(op = before, target = "2500-04-20T12:00:00.000", format = "%Y-%m-%-dT%H:%M:%S%.3f"))]
    before_dt: NaiveDateTime,
    #[validate(time(op = after, target = "2022-04-20", format = "%Y-%m-%d"))]
    after: NaiveDate,
    #[validate(time(op = after, target = "2022-04-20T12:00:00.000", format = "%Y-%m-%-dT%H:%M:%S%.3f"))]
    after_dt: NaiveDateTime,
    #[validate(time(op = in_period, target = "2022-04-20", format = "%Y-%m-%d", weeks = -2))]
    period: NaiveDate,
}
```

### **With route handler**

```rust
    use validify::{Validify, Payload, ValidifyPayload};

    #[derive(Debug, Validify, Payload)]
    struct JsonTest {
        #[modify(lowercase)]
        a: String,
        #[modify(trim, uppercase)]
        #[validate(length(equal = 11))]
        b: String,
    }

    // This would normally come from a framework
    struct Json<T>(T);

    fn test() {
      let jt = JsonTest {
          a: "MODIFIED".to_string(),
          b: "    makemeshout    ".to_string(),
      };
      let json = Json(JsonTestPayload::from(jt));
      mock_handler(json)
    }

    fn mock_handler(data: Json<JsonTestPayload>) {
      let data = data.0;
      let data = JsonTest::validify_from(data.into()).unwrap();
      mock_service(data);
    }

    fn mock_service(data: JsonTest) {
      assert_eq!(data.a, "modified".to_string());
      assert_eq!(data.b, "MAKEMESHOUT".to_string())
    }
```

See more examples in [the test directory](./derive_tests/tests).

### Contributing

If you have any ideas on how to improve Validify, such as common validations you find useful or better error messages, do not hesitate to open an issue or PR. All ideas are welcome!
