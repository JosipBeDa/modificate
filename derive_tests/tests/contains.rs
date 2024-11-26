use validify::{traits::Contains, Validate};

#[test]
fn can_validate_contains_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains("he"))]
        val: String,
    }

    let s = TestStruct {
        val: "hello".to_string(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn string_not_containing_needle_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains("he"))]
        val: String,
    }

    let s = TestStruct { val: String::new() };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "contains");
    assert_eq!(errs[0].params()["target"], "he");
}

#[test]
fn validates_number_vec() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains(3))]
        val: Vec<u64>,
    }

    let s = TestStruct {
        val: vec![32, 4, 2],
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "contains");
    assert_eq!(errs[0].location(), "/val");
    assert_eq!(errs[0].params()["target"], 3);

    let s = TestStruct {
        val: vec![32, 4, 2, 3],
    };
    let res = s.validate();
    assert!(res.is_ok());
}

#[test]
fn validates_struct_vec() {
    #[derive(Debug, PartialEq)]
    struct Params {
        a: u64,
        b: &'static str,
    }

    const PARAM: Params = Params {
        a: 2,
        b: "hello_world",
    };

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains(PARAM))]
        val: Vec<Params>,
    }

    let s = TestStruct {
        val: vec![Params { a: 3, b: "Hello" }, Params { a: 4, b: "world" }],
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "contains");
    assert_eq!(errs[0].location(), "/val");
    assert!(!errs[0].params().contains_key("target"));

    let s = TestStruct {
        val: vec![
            Params {
                a: 2,
                b: "hello_world",
            },
            Params { a: 4, b: "world" },
        ],
    };

    assert!(s.validate().is_ok())
}

#[test]
fn can_specify_code_for_contains() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains(value = "he", code = "dis dont have he yo"))]
        val: String,
    }
    let s = TestStruct { val: String::new() };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "dis dont have he yo");
}

#[test]
fn can_specify_message_for_contains() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(contains(value = "he", message = "oops"))]
        val: String,
    }
    let s = TestStruct { val: String::new() };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
}

#[test]
fn works_with_custom_type() {
    #[derive(Debug, Validate)]
    struct Container {
        #[validate(contains(value = 3, message = "oops"))]
        val: Containee,
    }

    #[derive(Debug)]
    struct Containee {
        x: usize,
    }

    impl Contains<usize> for Containee {
        fn has_element(&self, needle: &usize) -> bool {
            self.x == *needle
        }
    }

    let s = Container {
        val: Containee { x: 3 },
    };

    assert!(s.validate().is_ok());

    let s = Container {
        val: Containee { x: 4 },
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].message().unwrap(), "oops");
    assert_eq!(errs[0].code(), "contains");
}
