use validify::{Validate, ValidationErrors};

const MAX_CONST: usize = 10;
const MIN_CONST: usize = 0;

#[test]
fn can_validate_range_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5., max = 10.))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_only_min_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5.))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_only_max_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(max = 50.))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_range_value_crate_path_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = MIN_CONST, max = MAX_CONST))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn value_out_of_range_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5., max = 10.))]
        val: usize,
    }

    let s = TestStruct { val: 11 };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "range");
}

#[test]
fn value_out_of_range_fails_validation_with_crate_path() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = MIN_CONST, max = MAX_CONST))]
        val: usize,
    }

    let s = TestStruct { val: 16 };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "range");
}

#[test]
fn can_specify_code_for_range() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5., max = 10., code = "oops"))]
        val: usize,
    }
    let s = TestStruct { val: 11 };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "oops");
    assert_eq!(errs[0].params()["actual"], 11);
    assert_eq!(errs[0].params()["min"].as_f64().unwrap(), 5.0);
    assert_eq!(errs[0].params()["max"].as_f64().unwrap(), 10.0);
}

#[test]
fn can_specify_message_for_range() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5., max = 10., message = "oops"))]
        val: usize,
    }
    let s = TestStruct { val: 1 };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
}

#[test]
fn can_pass_reference_as_validate() {
    // This tests that the blanket Validate implementation on
    // `&T where T:Validate` works properly

    #[derive(Validate)]
    struct TestStruct {
        #[validate(range(min = -1., max = 1.))]
        num_field: f64,
    }

    fn validate<T: Validate>(value: &T) -> Result<(), ValidationErrors> {
        value.validate()
    }

    let val = TestStruct { num_field: 0.32 };
    assert!(validate(&val).is_ok());

    let val = TestStruct { num_field: 1.01 };
    assert!(validate(&val).is_err());
}
