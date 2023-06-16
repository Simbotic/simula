use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn any_simple() {
    let behavior = r#"
    (
        "Run until any good",
        Any(()),
        [
            ("Do a thing", Debug((fail: true))),
            ("Do another", Debug(())),
            ("Do more", Debug((fail: true))),
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Run until any good",
        "[2] STARTED Do a thing",
        "[3] STARTED Do another",
        "[4] STARTED Do more",
        "[2] FAILURE Do a thing",
        "[4] FAILURE Do more",
        "[3] SUCCESS Do another",
        "[1] SUCCESS Run until any good",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn any_simple_failure() {
    let behavior = r#"
    (
        "Run until any good",
        Any(()),
        [
            ("Do a thing", Debug((fail: true))),
            ("Do another", Debug((fail: true))),
            ("Do more", Debug((fail: true))),
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Run until any good",
        "[2] STARTED Do a thing",
        "[3] STARTED Do another",
        "[4] STARTED Do more",
        "[2] FAILURE Do a thing",
        "[3] FAILURE Do another",
        "[4] FAILURE Do more",
        "[1] FAILURE Run until any good",
    ]);
    assert_eq!(&trace, &expected_trace);
}
