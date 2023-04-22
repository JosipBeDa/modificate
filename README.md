# Validify

![build](https://img.shields.io/github/actions/workflow/status/biblius/validify/check.yml?label=build&style=plastic)
![test](https://img.shields.io/github/actions/workflow/status/biblius/validify/test.yml?label=test&style=plastic)
![coverage](https://img.shields.io/codecov/c/github/biblius/validify?style=plastic)
![version](https://img.shields.io/crates/v/validify)
![downloads](https://img.shields.io/crates/d/validify?color=%2332AA)

A procedural macro that provides attributes for field validation and modifiers. Particularly useful in the context of web payloads.

## **Modifiers**

|   Modifier    |  Type    |        Description
|---------------|----------|-----------------------
|  trim*        |  String  | Removes surrounding whitespace
|  uppercase*   |  String  | Calls `.to_uppercase()`
|  lowercase*   |  String  | Calls `.to_lowercase()`
|  capitalize*  |  String  | Makes the first char of the string uppercase
|  custom       |    Any   | Takes a function whose argument is `&mut <Type>`
|  validify*    |  Struct  | Can only be used on fields that are structs implementing the `Validify` trait. Runs all the nested struct's modifiers and validations

\*Also works for Vec\<T> by running `validify` on each element.

## **Validators**

All validators also take in a `code` and `message` as parameters, their values are must be string literals if specified.

|       Validator  |    Type     |      Params     | Param type |        Description
|------------------|-------------|-----------------|------------|-----------
| email            |  String     |        --       | -- |Checks emails based on [this spec](https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address).
| ip               |  String     | format  | Ident (v4/v6) |Checks if the string is an IP address.
| url              |  String     |        --       | -- |Checks if the string is a URL.
| length           | Collection  | min, max, equal  | LitInt | Checks if the collection length is within the specified params. Works through the HasLen trait.
| range            |  Int/Float     |     min, max    | LitFloat |Checks if the value is in the specified range.
| must_match       |    Any      |       value       | Ident |Checks if the field matches another field of the struct. The value must be equal to a field identifier on the deriving struct.
| contains         | Collection  |      value    | Lit/Path |Checks if the collection contains the specified value. If used on a K,V collection, it checks whether it has the provided key.
| contains_not     | Collection  |      value     |Lit/Path | Checks if the collection doesn't contain the specified value. If used on a K,V collection, it checks whether it has the provided key.
| non_control_char |  String     |        --       | -- |Checks if the field contains control characters
| custom           |  Function   |      function     | Path |Executes custom validation on the field specified by the end user
| regex            |  String     |      path      | Path |Matches the provided regex against the field. Intended to be used with lazy_static by providing a path to an initialised regex.
| credit_card      |  String     |        --       | -- |Checks if the field's value is a valid credit card number
| phone            |  String     |        --       | -- |Checks if the field's value is a valid phone number
| required         |  Option\<T>     |        --       | -- |Checks whether the field's value is Some
| is_in            |  impl PartialEq |    collection   | Path |Checks whether the field's value is in the specified collection
| not_in           |  impl PartialEq |    collection   | Path |Checks whether the field's value is not in the specified collection
| time             | NaiveDate[Time] |  See below   | See below |Performs a check based on the specified op

### **Time operators**

All time operators can take in `inclusive = bool`, `in_period` and the `_from_now` operators are inclusive by default.

The `target` param must be a string literal date or a path to an argless function that returns a date\[time].

If the target is a string literal, it must contain a `format` param, as per [this](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

Accepted interval parameters are `seconds`, `minutes`, `hours`, `days`, `weeks`.

The `_from_now` operators should not use negative duration due to how they validate the inputs,
 negative duration for `in_period` works fine.

|  Op  |  Params | Description
|------|---------|-----------------|
| before | target | Check whether a date\[time] is before the target one |
| after | target | Check whether a date\[time] is after the target one |
| before_now | -- | Check whether a date\[time] is before today\[now] |
| after_now | -- | Check whether a date\[time] is after the today\[now] |
| before_from_now | interval | Check whether a date\[time] is before the specified interval from today\[now] |
| after_from_now | interval | Check whether a date\[time] is after the specified interval from the today\[now] |
| in_period | target, interval | Check whether a date\[time] falls within a certain period|

Annotate the struct you want to modify and validate with the `Validify` attribute (if you do not need payload modification, derive the `validify::Validate` trait):

```rust
use validify::Validify;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validify)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validify)]
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
let res = Testor::validify(test.into());

assert!(matches!(res, Ok(_)));

let test = res.unwrap();

// Parent
assert_eq!(test.a, "lower me");
assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
assert_eq!(test.c, "modified");
assert_eq!(test.d, Some("modified".to_string()));
// Nested
assert_eq!(test.nested.a, "NOTSOTINYNOW");
assert_eq!(test.nested.b, "Capitalize me.");
```

Notice how even though field `d` is an option, the function used to modify the field still takes in `&mut String`. This is because modifiers and validations are only executed when the field isn't `None`.

## How it works

Every struct annotated with `#[derive(Validify)]` gets an associated payload struct, e.g.

```rust
#[derive(Validify)]
struct Something {
  a: usize,
  b: String,
  c: Option<bool>
}
```

behind the scenes will generate an intermediary

```rust
#[derive(Debug, Clone, Deserialize, validify::Validate)]
struct SomethingPayload {
  #[validate(required)]
  a: Option<usize>,
  #[validate(required)]
  b: Option<String>
  c: Option<bool>

  /* From and Into impls */
}
```

Note that every field that isn't an option will be an 'optional' required field in the payload. This is done to avoid deserialization errors for missing fields.

- _Do note that if a field exists in the incoming client payload, but is of the wrong type, a deserialization error will still occur as the payload is only being validated for whether the necessary fields exist. The same applies for invalid date\[time] formats._

The `Validify` implementation first validates the required fields of the generated payload. If any required fields are missing, no further modification/validation is done and the errors are returned. Next, the payload is transformed to the original struct and modifications and validations are run on it.

Validify's `validify` method always takes in the generated payload and outputs the original struct if all validations have passed.

The macro automatically implements the `Validate` and `Modify` traits in the wrapper trait `Validify`. This wrapper trait contains only the method `validify` which:

1. Runs the required validations on the payload struct
2. Runs modifications on the original
3. Runs validations and returns the original struct.

## Schema validation

Schema level validations can be performed using the following:

```rust
#[derive(Validify)]
#[validate(validate_testor)]
struct Testor { 
    a: String,
    b: usize,
 }

#[schema_validation]
fn validate_testor(t: &Testor) -> Result<(), ValidationErrors> {
  if t.a.as_str() == "yolo" && t.b < 2 {
    validify::schema_err!("Invalid Yolo", "Cannot yolo with b < 2", errors);
  }
}
```

The `#[schema_validation]` proc macro expands the function to:

```rust
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

## Errors

The main ValidationError is an enum with 2 variants, Field and Schema. Field errors are, as the name suggests, created when fields fail validation and are usually automatically generated unless using custom handlers (custom field validation functions always must return a result whose Err variant is ValidationError).

If you want to provide a message along with the error, you can directly specify it in the attribute (the same goes for the code),
for example:

`#[validate(contains(value = "something", message = "Does not contain something", code = "MUST_CONTAIN"))]`

Keep in mind, when specifying validations this way, all attribute parameters MUST be specified as [NameValue](https://docs.rs/syn/latest/syn/struct.MetaNameValue.html) pairs. This means that if you write

`#[validate(contains("something", message = "Bla"))]`,

you will get an error, as the parser expects either a single value or multiple name value pairs.

Locations are tracked for each error in a similar manner to [JSON pointers](https://opis.io/json-schema/2.x/pointers.html). When using
custom validation, whatever field name you specify in the returned error will be used in the location for that field.

Original (client payload) field names are also taken into account if they are annotated with `#[serde(rename)]` on the field level. The struct level `rename_all` attribute is currently not taken into account, but will eventually be made to work.

Schema errors are usually created by the user in schema validation. The `schema_err!` and `field_err!` macros provide an ergonomic way to create errors. All errors are composed to a `ValidationErrors` struct which contains a vec of all the validation errors.

## **Examples**

### **With route handler**

```rust
    fn actix_test() {
      #[derive(Debug, Serialize, Validify)]
      struct JsonTest {
          #[modify(lowercase)]
          a: String,
          #[modify(trim, uppercase)]
          #[validate(length(equal = 11))]
          b: String,
      }

      let jt = JsonTest {
          a: "MODIFIED".to_string(),
          b: "    makemeshout    ".to_string(),
      };

      let json = actix_web::web::Json(jt.into());
      mock_handler(json)
    }

    fn mock_handler(data: actix_web::web::Json<JsonTestPayload> 
    /* OR data: actix_web::web::Json<<JsonTest as Validify>::Payload> */) {
      let data = data.0;
      let data = JsonTest::validify(data).unwrap();
      mock_service(data);
    }

    fn mock_service(data: JsonTest) {
      assert_eq!(data.a, "modified".to_string());
      assert_eq!(data.b, "MAKEMESHOUT".to_string())
    }
```

### **Big Boi**

```rust

const WORKING_HOURS: &[&str] = &["08", "09", "10", "11", "12", "13", "14", "15", "16"];
const CAREER_LEVEL: &[&str] = &["One", "Two", "Over 9000"];
const STATUSES: &[&str] = &["online", "offline"];
const CONTRACT_TYPES: &[&str] = &["Fulltime", "Temporary"];
const ALLOWED_MIME: &[&str] = &["jpeg", "png"];
const ALLOWED_DURATIONS: &[i32] = &[1, 2, 3];

#[derive(Clone, Deserialize, Debug, Validify)]
#[serde(rename_all = "camelCase")]
#[validate(schema_validation)]
struct BigBoi {
    #[modify(trim)]
    #[validate(length(max = 300))]
    title: String,

    #[modify(trim)]
    #[validate(is_in(STATUSES))]
    status: String,

    #[modify(capitalize, trim)]
    city_country: String,

    #[validate(length(max = 1000))]
    education: String,

    #[modify(capitalize)]
    type_of_workplace: Vec<String>,

    #[validate(is_in(WORKING_HOURS))]
    working_hours: String,

    part_time_period: Option<String>,

    #[modify(capitalize)]
    #[validate(is_in(CONTRACT_TYPES))]
    contract_type: String,

    indefinite_probation_period: bool,

    #[validate(is_in(ALLOWED_DURATIONS))]
    indefinite_probation_period_duration: Option<i32>,

    #[validate(is_in(CAREER_LEVEL))]
    career_level: String,

    #[modify(capitalize)]
    benefits: String,

    #[validate(length(max = 60))]
    meta_title: String,

    #[validate(length(max = 160))]
    meta_description: String,

    #[validate(is_in(ALLOWED_MIME))]
    meta_image: String,

    #[validate(custom(greater_than_now))]
    published_at: String,

    #[validate(custom(greater_than_now))]
    expires_at: String,

    #[validify]
    languages: Vec<TestLanguages>,

    #[validify]
    tags: TestTags,
}


#[schema_validation]
fn schema_validation(bb: &BigBoi) -> Result<(), ValidationErrors> {
    if bb.contract_type == "Fulltime" && bb.part_time_period.is_some() {
        schema_err!("Fulltime contract cannot have part time period", errors);
    }

    if bb.contract_type == "Fulltime"
        && bb.indefinite_probation_period
        && bb.indefinite_probation_period_duration.is_none()
    {
        schema_err!(
            "No probation duration",
            "Indefinite probation duration must be specified",
            errors
        );
    }
}

fn greater_than_now(date: &str) -> Result<(), ValidationError> {
    let parsed = chrono::NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S");
    match parsed {
        Ok(date) => {
            if date
                < chrono::NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
                    .unwrap()
            {
                Err(ValidationError::new_field(
                    "field",
                    "Date cannot be less than now",
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            Err(ValidationError::new_field("field", "Could not parse date"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Validify)]
#[serde(rename_all = "camelCase")]
struct TestTags {
    #[modify(trim)]
    #[validate(length(min = 1, max = 10), custom(validate_names))]
    names: Vec<String>,
}

fn validate_names(names: &[String]) -> Result<(), ValidationError> {
    for n in names.iter() {
        if n.len() > 10 || n.is_empty() {
            return Err(ValidationError::new_field(
                "names",
                "Maximum length of 10 exceeded for name",
            ));
        }
    }
    Ok(())
}

const PROFICIENCY: &[&str] = &["dunno", "killinit"];

#[derive(Serialize, Clone, Deserialize, Debug, Validify)]
#[serde(rename_all = "camelCase")]
struct TestLanguages {
    company_opening_id: String,
    #[modify(trim)]
    language: String,

    #[modify(trim)]
    #[validate(is_in(PROFICIENCY))]
    proficiency: Option<String>,

    required: Option<bool>,
    created_by: String,
}

fn biggest_of_bois() {
  let tags = TestTags {
        // Invalid length due to `validate_names`
        names: vec![
            "taggggggggggggggggggggggggg".to_string(),
            "tag".to_string(),
            "tag".to_string(),
        ],
    };

    let languages = vec![
        TestLanguages {
            company_opening_id: "yolo mcswag".to_string(),
            language: "    tommorrowlang     ".to_string(),

            // Invalid proficiency
            proficiency: Some("invalid      ".to_string()),
            required: Some(true),
            created_by: "me".to_string(),
        },
        TestLanguages {
            company_opening_id: "divops".to_string(),
            language: "go".to_string(),

            // Invalid proficiency
            proficiency: Some("    invalid".to_string()),
            required: None,
            created_by: "they".to_string(),
        },
    ];

    let big = BigBoi {
        title: "me so big".to_string(),

        // Invalid status
        status: "invalid".to_string(),

        city_country: "gradrzava".to_string(),
        description_roles_responsibilites: "ask no questions tell no lies".to_string(),
        education: "any".to_string(),
        type_of_workplace: vec!["dumpster".to_string(), "mcdonalds".to_string()],

        // Invalid working hours
        working_hours: "invalid".to_string(),

        // Part time period with fulltime contract type
        part_time_period: Some(String::new()),
        contract_type: "Fulltime".to_string(),

        // Fulltime period with no duration
        indefinite_probation_period: true,
        indefinite_probation_period_duration: None,

        // Invalid career level
        career_level: "Over 100000".to_string(),

        benefits: "none".to_string(),
        meta_title: "this struct is getting pretty big".to_string(),
        meta_description: "and it's kind of annoying".to_string(),

        // Invalid mime type
        meta_image: "heic".to_string(),

        // Invalid time
        published_at: "1999-01-01 00:00:00".to_string(),

        // Invalid time
        expires_at: "1999-01-01 00:00:00".to_string(),
        languages,
        tags,
    };

    let res = BigBoi::validify(big.into());
    assert!(matches!(res, Err(ref e) if e.errors().len() == 11));

    let schema_errs = res.as_ref().unwrap_err().schema_errors();
    let field_errs = res.unwrap_err().field_errors();

    assert_eq!(schema_errs.len(), 2);
    assert_eq!(field_errs.len(), 9);
}

```
