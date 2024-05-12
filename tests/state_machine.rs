#![allow(clippy::print_stdout)]
#![allow(clippy::tests_outside_test_module)]
#![allow(clippy::use_debug)]
#![allow(clippy::missing_trait_methods)]

use machine_factory::state_machine;
use std::time::Instant;
use tap::Tap;

// First, we define the context that the traffic light will use
struct TrafficLightContext {
    last_change: Instant,
}

#[derive(Debug, Clone)]
struct TimeoutEvent;

impl TrafficLightEvent for TimeoutEvent {
    // Here we can override the default implementation of the pre_transition and post_transition methods,
    // as well as any other methods on the trait.
}

#[derive(Debug, Clone)]
struct EmergencyEvent;

impl TrafficLightEvent for EmergencyEvent {}

#[derive(Default, Debug, Clone)]
struct Red;

impl TrafficLightState for Red {
    // We can override the default implementation of the on_enter and on_exit methods,
    // as well as any other methods on the trait.
    fn on_enter(&mut self, context: &mut TrafficLightContext) {
        context.last_change = Instant::now();
        println!("{:?}: Changed to Red", Instant::now());
    }

    // This is required, since we don't provide a default implementation in the trait
    fn color(&self) -> TrafficLightColor {
        TrafficLightColor::Red
    }
}

#[derive(Default, Debug, Clone)]
struct Yellow;

impl TrafficLightState for Yellow {
    fn color(&self) -> TrafficLightColor {
        TrafficLightColor::Yellow
    }
}

#[derive(Default, Debug, Clone)]
struct Green;

impl TrafficLightState for Green {
    fn color(&self) -> TrafficLightColor {
        TrafficLightColor::Green
    }
}

// Finally, we define the state machine
state_machine!(
    TrafficLightMachine {
        context: TrafficLightContext,
        event_trait: trait TrafficLightEvent {},
        event_enum: #[derive(Debug, Clone)] TrafficLightMachineEvent,
        state_trait: trait TrafficLightState {
            fn on_enter(&mut self, context: &mut TrafficLightContext) {
                context.last_change = Instant::now();
            }

            fn color(&self) -> TrafficLightColor;
        },
        state_enum: #[derive(Debug)] #[derive(Clone)] TrafficLightMachineState,
        transitions: [
            Red {
                // From Red to Green when a TimeoutEvent occurs
                TimeoutEvent -> Green {
                    println!("{:?}: Changing to Green", Instant::now());
                    Green {}
                },
                EmergencyEvent -> Red {
                    println!("{:?}: Alredy red. No state change needed", Instant::now());
                    state
                },
            },
            Yellow {
                // No transition block is provided, so the target state needs to implement Default
                TimeoutEvent -> Red,
                EmergencyEvent -> Red
            },
            Green {
                TimeoutEvent -> Yellow,
                EmergencyEvent -> Red
            }
        ],
        // We could also define an unhandled_event block, which would be called when an event is not handled by the state
        // unhandled_event: {
        //     println!("{:?}: Unhandled event: {:?}", Instant::now(), event);
        //     state
        // }
    }
);

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum TrafficLightColor {
    #[default]
    Red,
    Yellow,
    Green,
}

#[test]
fn test() {
    let context = TrafficLightContext {
        last_change: Instant::now(),
    };

    let mut traffic_light = TrafficLightMachine::init(Red, context);

    assert_eq!(
        traffic_light.state().color(),
        TrafficLightColor::Red,
        "Color should be red"
    );

    _ = traffic_light.handle_event(TrafficLightMachineEvent::TimeoutEvent(TimeoutEvent {}));

    assert_eq!(
        traffic_light.state().color(),
        TrafficLightColor::Green,
        "Color should be green"
    );

    _ = traffic_light.handle_event(TimeoutEvent {});

    assert_eq!(
        traffic_light.state().color(),
        TrafficLightColor::Yellow,
        "Color should be yellow"
    );

    // Run test with --nocapture to see the output
    _ = traffic_light
        .handle_event(TimeoutEvent {}) // Red
        .tap(|x| {
            assert_eq!(
                x.state().color(),
                TrafficLightColor::Red,
                "Color should be red"
            );
        })
        .handle_event(TimeoutEvent {}) // Green
        .tap(|x| {
            assert_eq!(
                x.state().color(),
                TrafficLightColor::Green,
                "Color should be green"
            );
        })
        .handle_event(EmergencyEvent {});

    assert_eq!(
        traffic_light.state().color(),
        TrafficLightColor::Red,
        "Color should have changed from green to red, because of EmergencyEvent"
    );
}
