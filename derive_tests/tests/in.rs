use validify::Validate;

const ALLOWED_STRS: &[&str] = &["YES", "GOOD"];
const DISALLOWED_STRS: [&str; 2] = ["NO", "BAD"];

const ALLOWED_NUMS: &[u64] = &[1, 2, 3];
const DISALLOWED_NUMS: [u64; 3] = [4, 5, 6];

#[test]
fn properly_validates() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(is_in(collection = ALLOWED_STRS))]
        a: String,
        #[validate(is_in(ALLOWED_NUMS))]
        b: u64,
        #[validate(not_in(collection = DISALLOWED_STRS))]
        c: String,
        #[validate(not_in(DISALLOWED_NUMS))]
        d: u64,
    }

    let s = TestStruct {
        a: "YES".to_string(),
        b: 1,
        c: "Relax".to_string(),
        d: 7,
    };

    assert!(s.validate().is_ok());

    let s = TestStruct {
        a: "GOOD".to_string(),
        b: 3,
        c: "Hello".to_string(),
        d: 1,
    };

    assert!(s.validate().is_ok());
}

#[test]
fn properly_errors() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(is_in(collection = ALLOWED_STRS, message = "NOT_IN_ALLOWED"))]
        a: String,
        #[validate(is_in(collection = ALLOWED_NUMS, message = "NOT_IN_ALLOWED"))]
        b: u64,
        #[validate(not_in(collection = DISALLOWED_STRS, message = "IN_DISALLOWED"))]
        c: String,
        #[validate(not_in(collection = DISALLOWED_NUMS, message = "IN_DISALLOWED"))]
        d: u64,
    }

    let s = TestStruct {
        a: "NO".to_string(),
        b: 1,
        c: "Still fine".to_string(),
        d: 1,
    };

    let err = s.validate();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert_eq!(err.errors()[0].code(), "in");
    assert_eq!(err.errors()[0].message().unwrap(), "NOT_IN_ALLOWED");

    let s = TestStruct {
        a: "YES".to_string(),
        b: 4,
        c: "Still fine".to_string(),
        d: 1,
    };

    let err = s.validate();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert_eq!(err.errors()[0].code(), "in");
    assert_eq!(err.errors()[0].message().unwrap(), "NOT_IN_ALLOWED");

    let s = TestStruct {
        a: "GOOD".to_string(),
        b: 1,
        c: "BAD".to_string(),
        d: 1,
    };

    let err = s.validate();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert_eq!(err.errors()[0].code(), "not_in");
    assert_eq!(err.errors()[0].message().unwrap(), "IN_DISALLOWED");

    let s = TestStruct {
        a: "YES".to_string(),
        b: 2,
        c: "Still fine".to_string(),
        d: 6,
    };

    let err = s.validate();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert_eq!(err.errors()[0].code(), "not_in");
    assert_eq!(err.errors()[0].message().unwrap(), "IN_DISALLOWED");
    err.errors();
}

#[test]
fn properly_validates_option() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(is_in(collection = ALLOWED_STRS, message = "NOT_IN_ALLOWED"))]
        a: Option<String>,
        #[validate(is_in(collection = ALLOWED_NUMS, code = "FUK", message = "NOT_IN_ALLOWED"))]
        b: Option<u64>,
    }

    let test = TestStruct {
        a: Some("NO".to_string()),
        b: Some(1),
    };

    let res = test.validate();
    assert!(res.is_err());
    let res = res.unwrap_err();
    assert_eq!(res.errors()[0].code(), "in");
    assert_eq!(res.errors()[0].message().unwrap(), "NOT_IN_ALLOWED");

    let test = TestStruct {
        a: Some("NO".to_string()),
        b: Some(0),
    };

    let res = test.validate();
    assert!(res.is_err());
    let res = res.unwrap_err();
    assert_eq!(res.errors().len(), 2);
    assert_eq!(res.errors()[0].code(), "in");
    assert_eq!(res.errors()[0].message().unwrap(), "NOT_IN_ALLOWED");
    assert_eq!(res.errors()[1].code(), "FUK");
    assert_eq!(res.errors()[1].message().unwrap(), "NOT_IN_ALLOWED");
}

#[test]
fn properly_validates_structs() {
    #[derive(Debug, PartialEq)]
    struct Something(usize, usize);

    const ALLOWED: &[Something] = &[Something(1, 2), Something(3, 4)];

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(is_in(collection = ALLOWED, message = "NOT_IN_ALLOWED"))]
        a: Option<Something>,
        #[validate(is_in(collection = ALLOWED, code = "FUK", message = "NOT_IN_ALLOWED"))]
        b: Something,
    }

    let test = TestStruct {
        a: Some(Something(5, 6)),
        b: Something(7, 8),
    };

    let res = test.validate();
    assert!(res.is_err());
    let res = res.unwrap_err();
    assert_eq!(res.errors().len(), 2);
    assert_eq!(res.errors()[0].code(), "in");
    assert_eq!(res.errors()[0].message().unwrap(), "NOT_IN_ALLOWED");
    assert_eq!(res.errors()[1].code(), "FUK");
    assert_eq!(res.errors()[1].message().unwrap(), "NOT_IN_ALLOWED");

    let test = TestStruct {
        a: Some(Something(1, 2)),
        b: Something(3, 4),
    };

    let res = test.validate();
    assert!(res.is_ok());
}

#[test]
fn can_specify_literal_as_parameter() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(is_in(collection = ["foo", "bar"]))]
        val: String,
    }

    let s = TestStruct {
        val: "foo".to_string(),
    };
    assert!(s.validate().is_ok());

    let s = TestStruct {
        val: "bar".to_string(),
    };
    assert!(s.validate().is_ok());

    let s = TestStruct {
        val: "baz".to_string(),
    };
    let err = s.validate();
    let err = err.unwrap_err();
    assert_eq!(err.errors()[0].code(), "in");
}
