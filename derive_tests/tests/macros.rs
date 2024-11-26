use validify::{field_err, schema_err, schema_validation, Validate, ValidationErrors, Validify};

#[derive(Debug, Validify)]
#[validate(schema)]
struct Test {
    a: usize,
    b: usize,
    c: String,
}

#[schema_validation]
fn schema(t: &Test) -> Result<(), ValidationErrors> {
    if t.a > t.b {
        schema_err!("a_b");
    }
    if t.c.parse::<usize>().is_err() {
        schema_err!("NaN", "not a number");
    }
}

#[test]
fn schema_err() {
    let t = Test {
        a: 2,
        b: 1,
        c: "2foio".to_string(),
    };
    let errs = t.validate().unwrap_err();
    let errs = errs.errors();
    assert_eq!(errs.len(), 2);
    assert_eq!(errs[0].code(), "a_b");
    assert!(errs[0].message().is_none());
    assert_eq!(errs[0].location(), "/");
    assert!(errs[0].field_name().is_none());

    assert_eq!(errs[1].code(), "NaN");
    assert_eq!(errs[1].message().unwrap(), "not a number");
    assert_eq!(errs[1].location(), "/");
    assert!(errs[1].field_name().is_none());
}

#[test]
fn field_err() {
    let err = field_err!("foo");
    assert_eq!(err.code(), "foo");
    assert_eq!(err.location(), "");
    assert!(err.message().is_none());
    assert!(err.field_name().is_none());

    let err = field_err!("foo", "bar");
    assert_eq!(err.code(), "foo");
    assert_eq!(err.location(), "");
    assert_eq!(err.message().unwrap(), "bar");
    assert!(err.field_name().is_none());

    let err = field_err!("foo", "bar", "field");
    assert_eq!(err.code(), "foo");
    assert_eq!(err.message().unwrap(), "bar");
    assert_eq!(err.field_name().unwrap(), "field");
    assert_eq!(err.location(), "/field");
}
