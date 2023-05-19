use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn identity_sequence_success() {
    let behavior = r#"
    Behavior(
        "Identity results of sequence",
        Identity(()),
        [
            (
                "Sequencer of two actions",
                Sequencer(()),
                [
                    ("Do action", Debug((message:"Hello, from DebugMessage0!", fail:false))),
                    ("Do another action", Debug((message:"Hello, from DebugMessage1!")))
                ]
            )
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED Identity results of sequence",
        "[1] STARTED Sequencer of two actions",
        "[2] STARTED Do action",
        "[2] SUCCESS Do action",
        "[3] STARTED Do another action",
        "[3] SUCCESS Do another action",
        "[1] SUCCESS Sequencer of two actions",
        "[0] SUCCESS Identity results of sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn identity_sequence_failure() {
    let behavior = r#"
    Behavior(
        "Identity results of sequence",
        Identity(()),
        [
            (
                "Sequencer of two actions",
                Sequencer(()),
                [
                    ("Do action", Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    ("Do another action", Debug((message:"Hello, from DebugMessage1!")))
                ]
            )
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED Identity results of sequence",
        "[1] STARTED Sequencer of two actions",
        "[2] STARTED Do action",
        "[2] FAILURE Do action",
        "[1] FAILURE Sequencer of two actions",
        "[0] FAILURE Identity results of sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}
