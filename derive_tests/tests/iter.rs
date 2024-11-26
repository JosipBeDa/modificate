use chrono::NaiveDate;
use regex::Regex;
use validify::{field_err, Validate, ValidationError};

#[test]
fn email_iter_works() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(email))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![
            String::from("foo@bar.com"),
            String::from("waifoo"),
            String::from("bar"),
        ],
    };
    let res = test.validate();
    let errors = res.unwrap_err();

    assert_eq!(2, errors.errors().len());

    let error = &errors.errors()[0];
    assert_eq!("/test/1", error.location());
    assert_eq!("email", error.code());

    let error = &errors.errors()[1];
    assert_eq!("/test/2", error.location());
    assert_eq!("email", error.code());
}

#[test]
fn ip_iter_works() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(ip))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![
            String::from("::1"),
            String::from("127.0.0.1"),
            String::from("nope"),
        ],
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/2", error.location());
    assert_eq!("ip", error.code());
}

#[test]
fn phone_iter_works() {
    #[derive(Validate)]
    struct IterTest {
        #[validate(iter(phone))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![String::from("123")],
    };

    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/0", error.location());
    assert_eq!("phone", error.code());
}

#[test]
fn credit_card_iter_works() {
    #[derive(Validate)]
    struct IterTest {
        #[validate(iter(credit_card))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![String::from("123")],
    };

    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/0", error.location());
    assert_eq!("credit_card", error.code());
}

#[test]
fn required_iter_works() {
    #[derive(Validate)]
    struct IterTest {
        #[validate(iter(required))]
        test: Vec<Option<String>>,
    }

    // Do not do this irl pls
    let test = IterTest {
        test: vec![Some(String::from("123")), None],
    };

    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/1", error.location());
    assert_eq!("required", error.code());
}

#[test]
fn url_iter_works() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(url))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![String::from("http://mycoolsite.com"), String::from("nope")],
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/1", error.location());
    assert_eq!("url", error.code());
}

#[test]
fn non_ctrl_char_iter_works() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(non_control_char))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![
            String::from("http://mycoolsite.com"),
            String::from("\u{000c}"),
        ],
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/1", error.location());
    assert_eq!("non_control_char", error.code());
}

#[test]
fn custom_iter_works() {
    #[derive(Validate)]
    struct IterTest {
        #[validate(iter(custom(custom_fn)))]
        test: Vec<Inner>,
    }

    struct Inner {
        s: String,
    }

    fn custom_fn(s: &Inner) -> Result<(), ValidationError> {
        if s.s == "nope" {
            Err(field_err!("done_goofd", "cannot nope"))
        } else {
            Ok(())
        }
    }

    let test = IterTest {
        test: vec![
            Inner {
                s: String::from("foo"),
            },
            Inner {
                s: String::from("nope"),
            },
        ],
    };

    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/1", error.location());
    assert_eq!("done_goofd", error.code());
    assert_eq!("cannot nope", error.message().unwrap());
}

#[test]
fn regex_iter_works() {
    lazy_static::lazy_static!(
     static ref REGEX: Regex = Regex::new(r#"\d+"#).unwrap();
    );

    #[derive(Validate)]
    struct IterTest {
        #[validate(iter(regex(path = REGEX)))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![String::from("123"), String::from("4"), String::from("nope")],
    };

    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/2", error.location());
}

#[test]
fn length_iter_works() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(length(min = 2)))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![
            String::from("foo"),
            String::from("bar"),
            String::from("qux"),
            String::from("b"),
            String::from("quz"),
        ],
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/3", error.location());
    assert_eq!("length", error.code());
}

#[test]
fn range_iter_works() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(range(min = 2., max = 5.)))]
        test: Vec<u8>,
    }

    let test = IterTest {
        test: vec![2, 3, 4, 5, 6],
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/4", error.location());
    assert_eq!("range", error.code());
}

#[test]
fn contains_iter_works() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(contains("foo")))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![
            String::from("foomao"),
            String::from("waifoo"),
            String::from("bar"),
        ],
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/2", error.location());
    assert_eq!("contains", error.code());
}

#[test]
fn contains_iter_works_vec() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(contains(3)))]
        test: Vec<Vec<usize>>,
    }

    let test = IterTest {
        test: vec![vec![4, 3], vec![2, 1]],
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/1", error.location());
    assert_eq!("contains", error.code());
}

#[test]
fn contains_not_iter_works() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(contains_not("foo")))]
        test: Vec<String>,
    }

    let test = IterTest {
        test: vec![String::from("foomao")],
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/0", error.location());
    assert_eq!("contains_not", error.code());
}

#[test]
fn in_not_in_iter_works() {
    const NUMS: [usize; 3] = [1, 2, 3];

    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(is_in(NUMS)))]
        should: Vec<usize>,
        #[validate(iter(not_in(NUMS)))]
        shouldnt: Vec<usize>,
    }

    let test = IterTest {
        should: vec![4],
        shouldnt: vec![1],
    };
    let res = test.validate();
    let errors = res.unwrap_err();

    assert_eq!(2, errors.errors().len());

    let error = &errors.errors()[0];
    assert_eq!("/should/0", error.location());
    assert_eq!("in", error.code());

    let error = &errors.errors()[1];
    assert_eq!("/shouldnt/0", error.location());
    assert_eq!("not_in", error.code());
}

#[test]
fn time_iter_works() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(time(op = before, target = "2500-04-20", format = "%Y-%m-%d", inclusive = true)))]
        test: Vec<NaiveDate>,
    }

    let test = IterTest {
        test: vec![
            NaiveDate::from_ymd_opt(2023, 3, 11).unwrap(),
            NaiveDate::from_ymd_opt(2600, 3, 11).unwrap(),
        ],
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/1", error.location());
    assert_eq!("before_or_equal", error.code());
}

#[test]
fn iter_works_with_option() {
    #[derive(Debug, Validate)]
    struct IterTest {
        #[validate(iter(email))]
        test: Option<Vec<String>>,
    }

    let test = IterTest { test: None };
    let res = test.validate();
    assert!(res.is_ok());

    let test = IterTest {
        test: Some(vec![String::from("foo")]),
    };
    let res = test.validate();
    let errors = res.unwrap_err();
    let error = &errors.errors()[0];
    assert_eq!("/test/0", error.location());
    assert_eq!("email", error.code());
}
