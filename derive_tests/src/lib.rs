#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;
    use validify::{schema_err, schema_validation, ValidationError};
    use validify::{ValidationErrors, Validify};

    const ALLOWED: &[&str] = &["YOLO", "mcswag"];
    const DISALLOWED: [&str; 2] = ["nono", "NO"];
    const NUMBERS: &[i32] = &[1, 2, 3];
    const NO_NUMBERS: [i32; 3] = [4, 5, 6];

    #[derive(Debug, Clone, Validify)]
    #[validate(validator_test)]
    struct T {
        #[modify(custom(baz), trim, uppercase)]
        #[validate(
           length(min = 0, max = 12, code = "yea"),
           is_in(collection = ALLOWED, code = "CODE"),
           not_in(collection = DISALLOWED),
           contains(value = "YO", message = "hello"),
           custom(function = foo, code = "foo", message = "bar"),
           custom(bar),
        )]
        pub a: String,

        #[validate]
        b: U,

        #[modify(trim, lowercase)]
        #[validate(contains("lmeo"))]
        c: Vec<String>,

        #[modify(custom(baz), trim, uppercase)]
        #[validate(
            length(min = 1),
            is_in(collection = ALLOWED),
            not_in(collection = DISALLOWED)
        )]
        d: Option<String>,

        #[validate(
            is_in(NUMBERS),
            not_in(NO_NUMBERS),
            range(min = -20., max = 20.)
        )]
        e: Option<i32>,

        #[validate(ip)]
        f: String,

        #[validate(time(time = true, op = after, target = some_date))]
        g: NaiveDateTime,
    }

    fn some_date() -> NaiveDateTime {
        chrono::DateTime::parse_from_rfc3339("2023-04-16T14:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
            .naive_utc()
    }

    #[derive(Debug, Clone, Validify)]
    struct U {
        #[validate(range(min = 1.))]
        b: usize,
    }

    fn baz(_a: &mut String) {
        *_a = "YOLO".to_string()
    }

    fn foo(_a: &str) -> Result<(), ValidationError> {
        Ok(())
    }
    fn bar(_a: &str) -> Result<(), ValidationError> {
        Ok(())
    }

    #[schema_validation]
    fn validator_test(t: &T) -> Result<(), ValidationErrors> {
        if t.a == "YOLO" && t.e.is_none() {
            schema_err!("Invalid YOLO", "Can't yolo with non existent e");
        }
    }

    #[test]
    fn validate() {
        let mut _t = T {
            a: String::from("nono"),
            b: U { b: 2 },
            c: vec!["  LMEO  ".to_string()],
            d: Some("testovanje".to_string()),
            e: None,
            f: "0.0.0.0".to_string(),
            g: chrono::Utc::now().naive_utc(),
        };
        let res = _t.validify();
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.errors().len(), 1);
        assert_eq!(err.errors()[0].code(), "Invalid YOLO");
        assert_eq!(
            err.errors()[0].message(),
            Some("Can't yolo with non existent e".to_string())
        );
    }
}
