use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn inverter_sequence_success() {
    let behavior = r#"
    (
        root:(
            Inverter(()),
                [(Sequence(()),[
                    (DebugAction((message:"Hello, from DebugMessage0!", fail:true, repeat:0)),[]),
                    (DebugAction((message:"Hello, from DebugMessage1!", fail:false, repeat:2)),[])
                ])]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::decorators::inverter::Inverter",
        "[1] STARTED simula_behavior::composites::sequence::Sequence",
        "[2] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[2] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[1] FAILURE simula_behavior::composites::sequence::Sequence",
        "[0] SUCCESS simula_behavior::decorators::inverter::Inverter",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn inverter_sequence_failure() {
    let behavior = r#"
    (
        root:(
            Inverter(()),
                [(Sequence(()),[
                    (DebugAction((message:"Hello, from DebugMessage0!", fail:false, repeat:0)),[]),
                    (DebugAction((message:"Hello, from DebugMessage1!", fail:false, repeat:2)),[])
                ])]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::decorators::inverter::Inverter",
        "[1] STARTED simula_behavior::composites::sequence::Sequence",
        "[2] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[2] SUCCESS simula_behavior::actions::debug_action::DebugAction",
        "[3] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[3] SUCCESS simula_behavior::actions::debug_action::DebugAction",
        "[1] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[0] FAILURE simula_behavior::decorators::inverter::Inverter",
    ]);
    assert_eq!(&trace, &expected_trace);
}
