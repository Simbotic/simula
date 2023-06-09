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
        "[0] STARTED Run until any good",
        "[1] STARTED Do a thing",
        "[2] STARTED Do another",
        "[3] STARTED Do more",
        "[1] FAILURE Do a thing",
        "[3] FAILURE Do more",
        "[2] SUCCESS Do another",
        "[0] SUCCESS Run until any good",
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
        "[0] STARTED Run until any good",
        "[1] STARTED Do a thing",
        "[2] STARTED Do another",
        "[3] STARTED Do more",
        "[1] FAILURE Do a thing",
        "[2] FAILURE Do another",
        "[3] FAILURE Do more",
        "[0] FAILURE Run until any good",
    ]);
    assert_eq!(&trace, &expected_trace);
}
