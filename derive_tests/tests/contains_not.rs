use std::collections::HashMap;
use validify::Validate;

#[test]
fn can_validate_does_not_contain_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains_not("asdf"))]
        val: String,
    }

    let s = TestStruct {
        val: "hello".to_string(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn container_containing_needle_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains_not("asdf"))]
        val: HashMap<String, usize>,
    }

    let mut val = HashMap::new();
    val.insert("asdf".to_string(), 1);

    let s = TestStruct { val };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "contains_not");
    assert_eq!(errs[0].location(), "/val");
    assert_eq!(errs[0].params()["target"], "asdf");
}

#[test]
fn string_containing_needle_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains_not("he"))]
        val: String,
    }

    let s = TestStruct {
        val: "hello".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "contains_not");
    assert_eq!(errs[0].location(), "/val");
    assert_eq!(errs[0].params()["target"], "he");
}

#[test]
fn vec_containing_needle_usize_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains_not(4))]
        val: Vec<usize>,
    }

    let s = TestStruct { val: vec![4] };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "contains_not");
    assert_eq!(errs[0].location(), "/val");
    assert_eq!(errs[0].params()["target"], 4);
}

#[test]
fn vec_containing_needle_string_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains_not("foo"))]
        val: Vec<String>,
    }

    let s = TestStruct {
        val: vec!["foo".to_string()],
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "contains_not");
    assert_eq!(errs[0].location(), "/val");
    assert_eq!(errs[0].params()["target"], "foo");
}

#[test]
fn map_containing_needle_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains_not(4))]
        val: HashMap<usize, usize>,
    }

    let s = TestStruct {
        val: HashMap::from([(4, 4)]),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "contains_not");
    assert_eq!(errs[0].location(), "/val");
    assert_eq!(errs[0].params()["target"], 4);
}

#[test]
fn can_specify_code_for_does_not_contain() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains_not(value = "he", code = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "hello".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "oops");
    assert_eq!(errs[0].params()["target"], "he");
}

#[test]
fn can_specify_message_for_does_not_contain() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains_not(value = "he", message = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "hello".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
    assert_eq!(errs[0].params()["target"], "he");
}
