use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn all_simple() {
    let behavior = r#"
    (
        "Run until all good",
        All(()),
        [
            ("Do a thing", Debug(())),
            ("Do another", Debug(())),
            ("Do more", Debug(())),
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Run until all good",
        "[2] STARTED Do a thing",
        "[3] STARTED Do another",
        "[4] STARTED Do more",
        "[2] SUCCESS Do a thing",
        "[3] SUCCESS Do another",
        "[4] SUCCESS Do more",
        "[1] SUCCESS Run until all good",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn all_simple_fails() {
    let behavior = r#"
    (
        "Run until all good",
        All(()),
        [
            ("Do a thing", Debug(())),
            ("Do another", Debug((fail: true))),
            ("Do more", Debug(())),
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Run until all good",
        "[2] STARTED Do a thing",
        "[3] STARTED Do another",
        "[4] STARTED Do more",
        "[2] SUCCESS Do a thing",
        "[4] SUCCESS Do more",
        "[3] FAILURE Do another",
        "[1] FAILURE Run until all good",
    ]);
    assert_eq!(&trace, &expected_trace);
}
