use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn repeater_simple() {
    let behavior = r#"
    (
        root:(
            Repeater((repeat:Times(2))),
                [
                    (Debug((message:"Hello, from DebugMessage0!"))),
                ]
            )
        )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::decorators::repeater::Repeater",
        "[1] STARTED simula_behavior::actions::debug::Debug",
        "[1] SUCCESS simula_behavior::actions::debug::Debug",
        "[0] STARTED simula_behavior::decorators::repeater::Repeater",
        "[1] STARTED simula_behavior::actions::debug::Debug",
        "[1] SUCCESS simula_behavior::actions::debug::Debug",
        "[0] SUCCESS simula_behavior::decorators::repeater::Repeater",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn repeater_nested() {
    let behavior = r#"
    (
        root:(
            Repeater((repeat:Times(2))),
                [(Repeater((repeat:Times(2))),[
                    (Debug((message:"Hello, from DebugMessage0!"))),
                ])]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::decorators::repeater::Repeater",
        "[1] STARTED simula_behavior::decorators::repeater::Repeater",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::actions::debug::Debug",
        "[1] STARTED simula_behavior::decorators::repeater::Repeater",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::actions::debug::Debug",
        "[1] SUCCESS simula_behavior::decorators::repeater::Repeater",
        "[0] STARTED simula_behavior::decorators::repeater::Repeater",
        "[1] STARTED simula_behavior::decorators::repeater::Repeater",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::actions::debug::Debug",
        "[1] STARTED simula_behavior::decorators::repeater::Repeater",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::actions::debug::Debug",
        "[1] SUCCESS simula_behavior::decorators::repeater::Repeater",
        "[0] SUCCESS simula_behavior::decorators::repeater::Repeater",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn repeater_sequence_success() {
    let behavior = r#"
    (
        root:(
            Repeater((repeat:Times(2))),
                [(Sequence(()),[
                    (Debug((message:"Hello, from DebugMessage0!"))),
                    (Debug((message:"Hello, from DebugMessage1!")))
                ])]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::decorators::repeater::Repeater",
        "[1] STARTED simula_behavior::composites::sequence::Sequence",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::actions::debug::Debug",
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] SUCCESS simula_behavior::actions::debug::Debug",
        "[1] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[0] STARTED simula_behavior::decorators::repeater::Repeater",
        "[1] STARTED simula_behavior::composites::sequence::Sequence",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::actions::debug::Debug",
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] SUCCESS simula_behavior::actions::debug::Debug",
        "[1] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[0] SUCCESS simula_behavior::decorators::repeater::Repeater",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn repeater_sequence_failure() {
    let behavior = r#"
    (
        root:(
            Repeater((repeat:Times(2))),
                [(Sequence(()),[
                    (Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    (Debug((message:"Hello, from DebugMessage1!")))
                ])]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::decorators::repeater::Repeater",
        "[1] STARTED simula_behavior::composites::sequence::Sequence",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] FAILURE simula_behavior::actions::debug::Debug",
        "[1] FAILURE simula_behavior::composites::sequence::Sequence",
        "[0] STARTED simula_behavior::decorators::repeater::Repeater",
        "[1] STARTED simula_behavior::composites::sequence::Sequence",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] FAILURE simula_behavior::actions::debug::Debug",
        "[1] FAILURE simula_behavior::composites::sequence::Sequence",
        "[0] SUCCESS simula_behavior::decorators::repeater::Repeater",
    ]);
    assert_eq!(&trace, &expected_trace);
}
