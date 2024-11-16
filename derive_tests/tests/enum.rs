macro_rules! simple_enum {
    ($validation:meta, $ty:ty) => {
        #[derive(Debug, serde::Deserialize, validify::Validate, serde::Serialize)]
        enum TestEnum {
            Simple(#[$validation] $ty),

            Multiple(#[$validation] $ty, #[$validation] Option<$ty>),

            Nested(#[validate] TestStruct),

            DoublyNested(#[validate] TestStruct, #[validate] Option<TestStruct>),

            #[serde(rename_all = "camelCase")]
            Named {
                #[$validation]
                #[serde(rename = "foo")]
                foo_bar: $ty,
                #[$validation]
                qux_quack: Option<$ty>,
            },
        }

        #[derive(Debug, serde::Deserialize, validify::Validate, serde::Serialize)]
        struct TestStruct {
            #[$validation]
            val: $ty,
        }
    };
    let err = named.validate().unwrap_err();
    let err = err.errors();
    assert_eq!(1, err.len());

macro_rules! valid_validation_test {
    ($fn_id:ident, $value:expr) => {
        use serde_json::json;
        use validify::Validate;

    let named = TestEnum::Named {
        foo_bar: "bob".to_string(),
        qux_quack: Some(0),
    };
    let err = named.validate().unwrap_err();
    let errors = err.errors();
    assert_eq!(2, errors.len());

    let err = &errors[0];
    assert_eq!(err.code(), "email");
    assert_eq!(err.location(), "/foo");

    let err = &errors[1];
    assert_eq!("range", err.code());
    assert_eq!("/quxQuack", err.location());
    assert_eq!(1.0, err.params()["min"]);
    assert_eq!(0.0, err.params()["actual"]);
}

macro_rules! invalid_validation_test {
    (
        $simple_fn_id:ident,
        $multiple_fn_id:ident,
        $nested_fn_id:ident,
        $named_fn_id:ident, 
        $value:expr
    ) => {
        #[test]
        fn simple_enum_invalid() {
            let simple = TestEnum::Simple($value);
            let err = simple.validate().unwrap_err();
            let err = err.errors();

            assert_eq!(1, err.len());
        
            let err = &err[0];
            assert_eq!(err.location(), "/0");
        }
        
        #[test]
        fn multiple_enum_invalid() {
            let multiple = TestEnum::Multiple($value, None);
            let err = multiple.validate().unwrap_err();
            let err = err.errors();

            assert_eq!(1, err.len());
        
            let err = &err[0];
            assert_eq!(err.location(), "/0");
        
            let multiple = TestEnum::Multiple($value, Some($value));
            let err = multiple.validate().unwrap_err();
            let errors = err.errors();

            assert_eq!(2, errors.len());
        
            let err = &errors[0];
            assert_eq!(err.location(), "/0");
        
            let err = &errors[1];
            assert_eq!(err.location(), "/1");
        }

        #[test]
        fn nested_enum_invalid() {
            let nested = TestEnum::Nested(TestStruct {
                val: $value,
            });
            let err = nested.validate().unwrap_err();
            let err = err.errors();

            assert_eq!(1, err.len());

            let err = &err[0];
            assert_eq!(err.location(), "/0/val");
        }

        #[test]
        fn doubly_nested_enum_invalid() {
            let nested = TestEnum::DoublyNested(
                TestStruct {
                    val: $value,
                },
                None,
            );
            let err = nested.validate().unwrap_err();
            let err = err.errors();

            assert_eq!(1, err.len());

            let err = &err[0];
            assert_eq!(err.location(), "/0/val");

            let nested = TestEnum::DoublyNested(
                TestStruct {
                    val: $value,
                },
                Some(TestStruct {
                    val: $value,
                }),
            );
            let err = nested.validate().unwrap_err();
            let errors = err.errors();

            assert_eq!(2, errors.len());

            let err = &errors[0];
            assert_eq!(err.location(), "/0/val");

            let err = &errors[1];
            assert_eq!(err.location(), "/1/val");
        }

        #[test]
        fn named_enum_invalid() {
            let named = TestEnum::Named {
                foo_bar: $value,
                qux_quack: None,
            };
            let err = named.validate().unwrap_err();
            let err = err.errors();

            assert_eq!(1, err.len());

            assert_eq!(err[0].location(), "/foo");

            let named = TestEnum::Named {
                foo_bar: $value,
                qux_quack: Some($value),
            };
            let err = named.validate().unwrap_err();
            let errors = err.errors();

            assert_eq!(2, errors.len());

            let err = &errors[0];
            assert_eq!(err.location(), "/foo");

            let err = &errors[1];
            assert_eq!("/quxQuack", err.location());
        }
    };

    assert!(nested.validate().is_ok());

    let nested = NestedEnum::Variant {
        foo_bar: "bob".to_string(),
        nest: TestEnum::Named {
            foo_bar: "bob".to_string(),
            qux_quack: Some(0),
        },
    };

    let err = nested.validate().unwrap_err();
    let errors = err.errors();

    assert_eq!(3, errors.len());

    let err = &errors[0];
    assert_eq!(err.code(), "email");
    assert_eq!(err.location(), "/fooBar");

    let err = &errors[1];
    assert_eq!("email", err.code());
    assert_eq!("/nest/foo", err.location()); // Because of serde(rename)

    let err = &errors[2];
    assert_eq!("range", err.code());
    assert_eq!("/nest/quxQuack", err.location());
    assert_eq!(1.0, err.params()["min"]);
    assert_eq!(0.0, err.params()["actual"]);
}

mod range {
    simple_enum! { validate(range(min = 1., max = 10.)), usize }
    valid_validation_test!(successfully_validates_range, 5);
    invalid_validation_test! {
        validates_range_simple,
        validates_range_multiple,
        validates_range_nested,
        validates_range_named,
        100
    }
}

mod contains {
    simple_enum! { validate(contains("bob")), String }
    valid_validation_test!(successfully_validates_contains, "bob");
    invalid_validation_test! {
        validates_contains_simple,
        validates_contains_multiple,
        validates_contains_nested,
        validates_contains_named,
        "not".to_string()
    }
}

mod contains_not {
    simple_enum! { validate(contains_not("bob")), String }
    valid_validation_test!(successfully_validates_contains_not, "not");
    invalid_validation_test! {
        validates_contains_not_simple,
        validates_contains_not_multiple,
        validates_contains_not_nested,
        validates_contains_not_named,
        "bob".to_string()
    }
}

mod custom {
    use validify::ValidationError;

    fn validate_string(s: &str) -> Result<(), ValidationError> {
        if s == "bob" {
            Ok(())
        } else {
            Err(ValidationError::new_field("not bob"))
        }
    }
    simple_enum! { validate(custom(validate_string)), String }
    valid_validation_test!(successfully_validates_custom, "bob");
    invalid_validation_test! {
        validates_contains_not_simple,
        validates_contains_not_multiple,
        validates_contains_not_nested,
        validates_contains_not_named,
        "not".to_string()
    }
}

mod regex {
    lazy_static::lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r"^[a-z]+$").unwrap();
    }

    simple_enum! { validate(regex(RE)), String }
    valid_validation_test!(successfully_validates_regex, "bob");
    invalid_validation_test! {
        validates_regex_simple,
        validates_regex_multiple,
        validates_regex_nested,
        validates_regex_named,
        "  bob  ".to_string()
    }
}

mod credit_card {
    simple_enum! { validate(credit_card), String }
    valid_validation_test!(successfully_validates_credit_card, "5236313877109142");
    invalid_validation_test! {
        validates_credit_card_simple,
        validates_credit_card_multiple,
        validates_credit_card_nested,
        validates_credit_card_named,
        "bob".to_string()
    }
}

mod phone {
    simple_enum! { validate(phone), String }
    valid_validation_test!(successfully_validates_phone, "+14152370800");
    invalid_validation_test! {
        validates_phone_simple,
        validates_phone_multiple,
        validates_phone_nested,
        validates_phone_named,
        "bob".to_string()
    }
}

mod is_in {
    const ALLOWED_STRS: &[&str] = &["YES"];
    fn str_slice_to_string(slice: &[&str]) -> Vec<String> {
        slice.iter().map(|el| String::from(*el)).collect()
    }
    simple_enum! { validate(is_in(collection = str_slice_to_string(ALLOWED_STRS))), String }
    valid_validation_test!(successfully_validates_is_in, "YES");
    invalid_validation_test! {
        validates_is_in_simple,
        validates_is_in_multiple,
        validates_is_in_nested,
        validates_is_in_named,
        "NO".to_string()
    }
}

mod not_in {
    const DISALLOWED_STRS: &[&str] = &["NO"];
    fn str_slice_to_string(slice: &[&str]) -> Vec<String> {
        slice.iter().map(|el| String::from(*el)).collect()
    }
    simple_enum! { validate(not_in(collection = str_slice_to_string(DISALLOWED_STRS))), String }
    valid_validation_test!(successfully_validates_not_in, "YES");
    invalid_validation_test! {
        validates_not_in_simple,
        validates_not_in_multiple,
        validates_not_in_nested,
        validates_not_in_named,
        "NO".to_string()
    }
}

mod required {
    simple_enum! { validate(required), Option<usize> }
    valid_validation_test!(successfully_validates_required, Some(1));
}

mod iter {
    use serde::{Deserialize, Serialize};
    use validify::Validate;

    #[derive(Debug, Validate, Serialize, Deserialize)]
    enum TestEnum {
        Unnamed(#[validate(iter(email))] Vec<String>),
        Named {
            #[validate(iter(email))]
            iter: Vec<String>,            
        },
    }

    #[test]
    fn validates_unnamed_iter() {
        let v = TestEnum::Unnamed(vec!["bob@bob.com".to_string(), "not".to_string()]);
        let res = v.validate();

        let err = res.unwrap_err();
        let err = err.errors();

        assert_eq!(1, err.len());
        let err = &err[0];
        assert_eq!(err.location(), "/0/1");

    }   

    #[test]
    fn validates_named_iter() {
        let v = TestEnum::Named {
            iter: vec!["bob@bob.com".to_string(), "not".to_string()],
        };
        let res = v.validate();

        let err = res.unwrap_err();
        let err = err.errors();

        assert_eq!(1, err.len());
        let err = &err[0];
        assert_eq!(err.location(), "/iter/1");
    }
}
