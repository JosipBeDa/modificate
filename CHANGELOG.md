# Changelog

- ## 2.0.0

- Enum validation.
- Removes `must_match` validation due to quirks with enums in favor of much simpler `schema` validation.
- Rename `HasLen` trait to `Length`.
- Refactor `Contains` to take in generic and implement on common types.
- Fixes some invalid attribute matching.

- ## 1.4.0

- Add new validator - `iter`.
- `validate(length)` - Change "actual" parameter to be the length of the item instead of the item contents.
- `validate(time)` - Add `time` key which now must explicitly be set to true if validating datetimes (as opposed to just dates). The validator no longer
  checks the field to determine whether datetimes should be used. This now makes it interoperable with the new `iter` validator.
- `validate(contains)` - Remove `actual` parameter since it can be derived from the error location.
- `validate(is_in/not_in)` - The `collection` parameter must now explicitly be specified when dealing with string slices.
  It now takes in an expression instead of a path due to quirks when trying to use `&[&str]` collections.
  Now the collections can be mapped inline to match the type of the field.
  The collection now also must have a `Contains` implementation with `Contains::Needle` as the type being validated.
  See [the test directory](./derive_tests/lib.rs) for examples.
- Make the `Contains` trait public.

- ## 1.3.0

- Added the `ValidifyPayload` trait to associate payloads to original structs. Moved
  `validate_into` and `validify_into` to the trait as `validate_from` and `validify_from`.
  The functions are now called from the original instead of the payload.
  This is done to make it easier to interop with crates like `axum-valid`.

- ## 1.2.0

- The `Validify` trait now has only one function: `validify` which is now used
  solely to perform modifications and validations on the implementing struct.
- Removed the associated type `Payload` from `Validify` and make the generation optional.
- Move payload generation to a separate macro (`#[derive(Payload)]`).
- Payloads now have a `validate_into` and `validify_into` functions for easy conversion
  between them and the original.

- ## 1.1.0

- Rehaul derive macro infrastructure. Improve type checking by using proper syn structs.
- Remove `field` argument from `ValidationError::new_field` and add `ValidationError::new_field_named`.
- `custom` validation now automatically appends the field name to the error when `new_field` is used. If the error
  already has a field name (i.e. was created with `new_field_named`), it will not change it.
- `schema_err!` is now a proc macro and no longer takes in errors as a param. It can now only be used inside functions
  annotated with `schema_validation`.
- `field_err!` can now be used outside `schema_validation` and is the preferred way of consisely constructing field errors.
- Fields in field errors are now renamed back to original (whatever it was before deserialization) if the struct has a `rename_all` serde attribute.
- Field level attributes are now propagated to payload fields.
- Field level custom deserialization is now applied on payload fields as well.

- ## 1.0.12

- All error params now represent the violating field's value with `actual` and the expected value as `target` if applicable.
- Remove params in errors where validation failed for whole structs and collections due to the error size being pretty massive
- Due to the previous change, structs deriving `Validify` no longer have to implement `Serialize` as all the info is contained in the field and location of the error.
- Remove redundant params from `required` errors.

- ## 1.0.11

- Struct level serde attributes are now propagated to the payload

- ## 1.0.0

- Attributes now follow rust conventional syntax
- `contains` and `is_in` can now be applied to any type
- Added simple time validation
- Most validators can now directly specify a path instead of string literals
- Errors now have a location similar to a JSON pointer
- Nested validifies now result in payload structs to also contain payload versions of their children.
