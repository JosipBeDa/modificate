use validify::Validify;

#[derive(Debug, Validify)]
enum TestEnum {
    Unnamed(
        #[validate(length(equal = 8))]
        #[modify(custom(modify_foobar))]
        #[modify(trim)]
        String,
        #[validify] TestStruct,
        #[validify] Vec<TestStruct>,
    ),
    Named {
        #[validate(length(equal = 8))]
        #[modify(custom(modify_foobar))]
        #[modify(trim)]
        basic: String,

        #[validify]
        nested: TestStruct,

        #[validify]
        list: Vec<TestStruct>,
    },
}

#[derive(Debug, Validify)]
struct TestStruct {
    #[validate(email)]
    #[modify(lowercase)]
    val: String,

    #[validate(length(equal = 8))]
    #[modify(custom(modify_foobar))]
    #[modify(trim)]
    modify_me: String,
}

fn modify_foobar(s: &mut String) {
    *s = String::from("   modified   ");
}

#[test]
fn unnamed_enum_validify_validation_success() {
    let mut unnamed = TestEnum::Unnamed(
        String::from("   bob@bob.com    "),
        TestStruct {
            val: String::from("BOB@BOB.COM"),
            modify_me: String::from(""),
        },
        vec![TestStruct {
            val: String::from("BOB@BOB.COM"),
            modify_me: String::from(""),
        }],
    );

    let res = unnamed.validify();

    assert!(res.is_ok());

    let TestEnum::Unnamed(basic, nested, list) = unnamed else {
        unreachable!();
    };

    // Gets modified and then trimmed
    assert_eq!(basic, "modified");
    assert_eq!(nested.val, "bob@bob.com");
    assert_eq!(nested.modify_me, "modified");
    assert_eq!(list[0].val, "bob@bob.com");
    assert_eq!(list[0].modify_me, "modified");
}

#[test]
fn named_enum_validify_validation_success() {
    let mut named = TestEnum::Named {
        basic: String::from("  bob@bob.com  "),
        nested: TestStruct {
            val: String::from("BOB@BOB.COM"),
            modify_me: String::from(""),
        },
        list: vec![TestStruct {
            val: String::from("BOB@BOB.COM"),
            modify_me: String::from(""),
        }],
    };

    let res = named.validify();

    assert!(res.is_ok());

    let TestEnum::Named {
        basic,
        nested,
        list,
    } = named
    else {
        unreachable!();
    };

    // Gets modified and then trimmed
    assert_eq!(basic, "modified");
    assert_eq!(nested.val, "bob@bob.com");
    assert_eq!(nested.modify_me, "modified");
    assert_eq!(list[0].val, "bob@bob.com");
    assert_eq!(list[0].modify_me, "modified");
}
