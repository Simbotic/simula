use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn repeater_simple() {
    let behavior = r#"
    (
        "Do a few times",
        Repeater((repeat:Times(2))),
        [
            ("Do an action", Debug((message:"Hello, from DebugMessage0!"))),
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Do a few times",
        "[2] STARTED Do an action",
        "[2] SUCCESS Do an action",
        "[1] STARTED Do a few times",
        "[2] STARTED Do an action",
        "[2] SUCCESS Do an action",
        "[1] SUCCESS Do a few times",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn repeater_nested() {
    let behavior = r#"
    (
        "Do a few times",
        Repeater((repeat:Times(2))),
        [
            (
                "Do a few more times",
                Repeater((repeat:Times(2))),
                [
                    ("Do an action", Debug((message:"Hello, from DebugMessage0!"))),
                ]
            )
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Do a few times",
        "[2] STARTED Do a few more times",
        "[3] STARTED Do an action",
        "[3] SUCCESS Do an action",
        "[2] STARTED Do a few more times",
        "[3] STARTED Do an action",
        "[3] SUCCESS Do an action",
        "[2] SUCCESS Do a few more times",
        "[1] STARTED Do a few times",
        "[2] STARTED Do a few more times",
        "[3] STARTED Do an action",
        "[3] SUCCESS Do an action",
        "[2] STARTED Do a few more times",
        "[3] STARTED Do an action",
        "[3] SUCCESS Do an action",
        "[2] SUCCESS Do a few more times",
        "[1] SUCCESS Do a few times",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn repeater_sequence_success() {
    let behavior = r#"
    (
        "Do a few times",
        Repeater((repeat:Times(2))),
            [(
                "Sequencer of two actions",
                Sequencer(()),
                [
                    ("Do an action", Debug((message:"Hello, from DebugMessage0!"))),
                    ("Do another action", Debug((message:"Hello, from DebugMessage1!")))
                ]
        )]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Do a few times",
        "[2] STARTED Sequencer of two actions",
        "[3] STARTED Do an action",
        "[3] SUCCESS Do an action",
        "[4] STARTED Do another action",
        "[4] SUCCESS Do another action",
        "[2] SUCCESS Sequencer of two actions",
        "[1] STARTED Do a few times",
        "[2] STARTED Sequencer of two actions",
        "[3] STARTED Do an action",
        "[3] SUCCESS Do an action",
        "[4] STARTED Do another action",
        "[4] SUCCESS Do another action",
        "[2] SUCCESS Sequencer of two actions",
        "[1] SUCCESS Do a few times",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn repeater_sequence_failure() {
    let behavior = r#"
    (
        "Do a few times",
        Repeater((repeat:Times(2))),
            [(
                "Sequencer of a few actions",
                Sequencer(()),
                [
                    ("Do an action", Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    ("Do another action", Debug((message:"Hello, from DebugMessage1!")))
                ]
            )
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Do a few times",
        "[2] STARTED Sequencer of a few actions",
        "[3] STARTED Do an action",
        "[3] FAILURE Do an action",
        "[2] FAILURE Sequencer of a few actions",
        "[1] STARTED Do a few times",
        "[2] STARTED Sequencer of a few actions",
        "[3] STARTED Do an action",
        "[3] FAILURE Do an action",
        "[2] FAILURE Sequencer of a few actions",
        "[1] SUCCESS Do a few times",
    ]);
    assert_eq!(&trace, &expected_trace);
}
