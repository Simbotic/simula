use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn sequence_single_success() {
    let behavior = r#"
    (
        root:(
            "Sequence of a few actions",
            Sequence(()),
            [
                ("Do action 0", Debug((message:"Hello, from DebugMessage0!"))),
                ("Do action 1", Debug((message:"Hello, from DebugMessage1!"))),
                ("Do action 2", Debug((message:"Hello, from DebugMessage2!"))),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED Sequence of a few actions",
        "[1] STARTED Do action 0",
        "[1] SUCCESS Do action 0",
        "[2] STARTED Do action 1",
        "[2] SUCCESS Do action 1",
        "[3] STARTED Do action 2",
        "[3] SUCCESS Do action 2",
        "[0] SUCCESS Sequence of a few actions",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_single_failure() {
    let behavior = r#"
    (
        root:(
            "Sequence of a few actions",
            Sequence(()), 
            [
                ("Do action 0", Debug((message:"Hello, from DebugMessage0!"))),
                ("Do action 1", Debug((message:"Hello, from DebugMessage1!", fail:true))),
                ("Do action 2", Debug((message:"Hello, from DebugMessage2!"))),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED Sequence of a few actions",
        "[1] STARTED Do action 0",
        "[1] SUCCESS Do action 0",
        "[2] STARTED Do action 1",
        "[2] FAILURE Do action 1",
        "[0] FAILURE Sequence of a few actions",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_nested_success() {
    let behavior = r#"
    (
        root:(
            "Sequence of sequence",
            Sequence(()),
            [
                (
                    "Depth 1 sequence",
                    Sequence(()),
                    [
                        (
                            "Depth 2 sequence",
                            Sequence(()),
                            [
                                ("Deep nested action 1", Debug((message:"Hello, from DebugMessage0!"))),
                            ]
                        ),
                        (
                            "Another depth 2 sequence",
                            Sequence(()),
                            [
                                ("Deep nested action 2", Debug((message:"Hello, from DebugMessage1!"))),
                                ("Deep nested action 3", Debug((message:"Hello, from DebugMessage2!"))),
                            ]
                        )
                    ]
                ),
            ]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED Sequence of sequence",
        "[1] STARTED Depth 1 sequence",
        "[2] STARTED Depth 2 sequence",
        "[3] STARTED Deep nested action 1",
        "[3] SUCCESS Deep nested action 1",
        "[2] SUCCESS Depth 2 sequence",
        "[4] STARTED Another depth 2 sequence",
        "[5] STARTED Deep nested action 2",
        "[5] SUCCESS Deep nested action 2",
        "[6] STARTED Deep nested action 3",
        "[6] SUCCESS Deep nested action 3",
        "[4] SUCCESS Another depth 2 sequence",
        "[1] SUCCESS Depth 1 sequence",
        "[0] SUCCESS Sequence of sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_nested_failure() {
    let behavior = r#"
    (
        root:(
            "Sequence of sequence",
            Sequence(()),
            [
                (
                    "Nested sequence of sequences",
                    Sequence(()),
                    [
                        (
                            "First depth 2 sequence",
                            Sequence(()),
                            [
                                ("Deep nested action 1", Debug((message:"Hello, from DebugMessage0!"))),
                            ]
                        ),
                        (
                            "Second depth 2 sequence",
                            Sequence(()),
                            [
                                ("Deep nested action 2", Debug((message:"Hello, from DebugMessage1!", fail:true))),
                                ("Deep nested action 3", Debug((message:"Hello, from DebugMessage2!"))),
                            ]
                        )
                    ]
                ),
            ]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED Sequence of sequence",
        "[1] STARTED Nested sequence of sequences",
        "[2] STARTED First depth 2 sequence",
        "[3] STARTED Deep nested action 1",
        "[3] SUCCESS Deep nested action 1",
        "[2] SUCCESS First depth 2 sequence",
        "[4] STARTED Second depth 2 sequence",
        "[5] STARTED Deep nested action 2",
        "[5] FAILURE Deep nested action 2",
        "[4] FAILURE Second depth 2 sequence",
        "[1] FAILURE Nested sequence of sequences",
        "[0] FAILURE Sequence of sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_nested_selector_success() {
    let behavior = r#"
    (
        root:(
            "Sequence of sequence",
            Sequence(()),
            [
                (
                    "Nested sequence of sequences",
                    Sequence(()),
                    [
                        (
                            "First depth 2 sequence",
                            Selector(()),
                            [
                                ("Unlock doors", Debug((message:"Unlocked the doors!"))),
                            ]
                        ),
                        (
                            "Second depth 2 sequence",
                            Sequence(()),
                            [
                                ("Close doors",Debug((message:"Closed doors!"))),
                                ("Enter door",Debug((message:"Go to selected door!"))),
                            ]
                        )
                    ]
                ),
            ]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED Sequence of sequence",
        "[1] STARTED Nested sequence of sequences",
        "[2] STARTED First depth 2 sequence",
        "[3] STARTED Unlock doors",
        "[3] SUCCESS Unlock doors",
        "[2] SUCCESS First depth 2 sequence",
        "[4] STARTED Second depth 2 sequence",
        "[5] STARTED Close doors",
        "[5] SUCCESS Close doors",
        "[6] STARTED Enter door",
        "[6] SUCCESS Enter door",
        "[4] SUCCESS Second depth 2 sequence",
        "[1] SUCCESS Nested sequence of sequences",
        "[0] SUCCESS Sequence of sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}
