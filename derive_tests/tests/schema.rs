use validify::{Validate, ValidationError, ValidationErrors};

#[test]
fn can_validate_schema_fn_ok() {
    fn valid_schema_fn(_: &TestStruct) -> Result<(), ValidationErrors> {
        Ok(())
    }

    #[allow(dead_code)]
    #[derive(Debug, Validate)]
    #[validate(valid_schema_fn)]
    struct TestStruct {
        val: String,
    }

    let s = TestStruct {
        val: "hello".into(),
    };

    assert!(s.validate().is_ok());
}

mod some_defining_mod {
    use validify::Validate;

    #[derive(Debug, Validate)]
    #[validate(crate::some_validation_mod::valid_schema_fn)]
    pub struct TestStructValid {
        pub _val: String,
    }

    #[derive(Debug, Validate)]
    #[validate(crate::some_validation_mod::invalid_schema_fn)]
    pub struct TestStructInvalid {
        pub _val: String,
    }
}

mod some_validation_mod {
    use validify::{ValidationError, ValidationErrors};

    use crate::some_defining_mod::{TestStructInvalid, TestStructValid};

    pub fn valid_schema_fn(_: &TestStructValid) -> Result<(), ValidationErrors> {
        Ok(())
    }

    pub fn invalid_schema_fn(_: &TestStructInvalid) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new_schema("fuk"));
        Err(errors)
    }
}

#[test]
fn can_validate_fully_qualified_fn_ok() {
    let s = some_defining_mod::TestStructValid {
        _val: "hello".into(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn can_fail_fully_qualified_fn_validation() {
    let s = some_defining_mod::TestStructInvalid {
        _val: "hello".into(),
    };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.schema_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "fuk");
}

#[test]
fn can_validate_multiple_schema_fn_ok() {
    fn valid_schema_fn(_: &TestStruct) -> Result<(), ValidationErrors> {
        Ok(())
    }

    fn valid_schema_fn2(_: &TestStruct) -> Result<(), ValidationErrors> {
        Ok(())
    }

    #[allow(dead_code)]
    #[derive(Debug, Validate)]
    #[validate(valid_schema_fn)]
    #[validate(valid_schema_fn2)]
    struct TestStruct {
        val: String,
    }

    let s = TestStruct {
        val: "hello".into(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn can_fail_schema_fn_validation() {
    fn invalid_schema_fn(_: &TestStruct) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new_schema("fuk"));
        Err(errors)
    }

    #[allow(dead_code)]
    #[derive(Debug, Validate)]
    #[validate(invalid_schema_fn)]
    struct TestStruct {
        val: String,
    }

    let s = TestStruct { val: String::new() };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.schema_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "fuk");
}

#[test]
fn can_fail_multiple_schema_fn_validation() {
    fn invalid_schema_fn(_: &TestStruct) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new_schema("fuk1"));
        Err(errors)
    }

    fn invalid_schema_fn2(_: &TestStruct) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new_schema("fuk2"));
        Err(errors)
    }

    #[allow(dead_code)]
    #[derive(Debug, Validate)]
    #[validate(invalid_schema_fn)]
    #[validate(invalid_schema_fn2)]
    struct TestStruct {
        val: String,
    }

    let s = TestStruct { val: String::new() };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.schema_errors();
    assert_eq!(errs.len(), 2);
    assert_eq!(errs[0].code(), "fuk1");
    assert_eq!(errs[1].code(), "fuk2");
}

#[test]
fn can_specify_message_for_schema_fn() {
    fn invalid_schema_fn(_: &TestStruct) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new_schema("fuk").with_message("oops".to_string()));
        Err(errors)
    }

    #[allow(dead_code)]
    #[derive(Debug, Validate)]
    #[validate(invalid_schema_fn)]
    struct TestStruct {
        val: String,
    }
    let s = TestStruct { val: String::new() };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.schema_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
}

#[test]
fn can_choose_to_run_schema_validation_even_after_schema_errors() {
    fn invalid_schema_fn(_: &TestStruct) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new_schema("fuk"));
        Err(errors)
    }
    #[allow(dead_code)]
    #[derive(Debug, Validate)]
    #[validate(invalid_schema_fn)]
    struct TestStruct {
        val: String,
        #[validate(range(min = 1., max = 10.))]
        num: usize,
    }

    let s = TestStruct {
        val: "hello".to_string(),
        num: 0,
    };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.schema_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().code(), "fuk");
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().code(), "range");
}
