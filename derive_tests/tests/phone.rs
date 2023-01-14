use validator::Validate;

#[test]
fn can_validate_phone_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(phone)]
        val: String,
    }

    let s = TestStruct {
        val: "+14152370800".to_string(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn bad_phone_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(phone)]
        val: String,
    }

    let s = TestStruct {
        val: "bob".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "phone");
}

#[test]
fn can_specify_code_for_phone() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(phone(code = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "bob".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "oops");
    assert_eq!(errs[0].params()["value"], "bob");
}

#[test]
fn can_specify_message_for_phone() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(phone(message = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "bob".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
}
