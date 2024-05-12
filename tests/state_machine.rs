#![allow(clippy::print_stdout)]
#![allow(clippy::tests_outside_test_module)]
#![allow(clippy::use_debug)]
#![allow(clippy::missing_trait_methods)]
#![allow(clippy::single_match_else)]
#![allow(clippy::wildcard_enum_match_arm)]

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
struct EmergencyEvent {
    requested_color: TrafficLightColor,
}

impl TrafficLightEvent for EmergencyEvent {}

#[derive(Debug, Clone)]
struct ChaosEvent;
impl TrafficLightEvent for ChaosEvent {}

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
                TimeoutEvent {
                    println!("{:?}: Changing to Green", Instant::now());
                    Green {} // Any state can be returned here
                },
                EmergencyEvent {
                    match event.requested_color {
                        TrafficLightColor::Red => {
                            println!("{:?}: Changing to Red", Instant::now());
                            TrafficLightMachineState::from(state)
                            // Note: even though we don't change the state, the lifecycle methods
                            // (i.e., on_exit, pre_transition, post_transition, on_enter) will still be called
                        }
                        TrafficLightColor::Yellow => {
                            println!("{:?}: Changing to Yellow", Instant::now());
                            Yellow {}.into()
                        }
                        TrafficLightColor::Green => {
                            println!("{:?}: Changing to Green", Instant::now());
                            Green {}.into()
                        }
                    }
                },
                ChaosEvent {
                    println!("{:?}: ChaosEvent", Instant::now());

                    // We can't return the state object because they may be different types.
                    // Instead, we return the state enum variant.
                    if context.last_change.elapsed().as_secs() % 2 == 0 {
                        println!("{:?}: Changing to Green", Instant::now());
                        TrafficLightMachineState::Green(Green {})
                    } else {
                        println!("{:?}: Changing to Yellow", Instant::now());
                        Yellow {}.into()
                    }
                }
            },
            Yellow {
                // No transition block is provided, so the target state needs to implement Default
                TimeoutEvent -> Red,
                EmergencyEvent {
                    let res: TrafficLightMachineState = match event.requested_color {
                        TrafficLightColor::Red => Red {}.into(),
                        TrafficLightColor::Yellow => Yellow {}.into(),
                        TrafficLightColor::Green => Green {}.into(),
                    };

                    res
                },
                ChaosEvent -> Green,
            },
            Green {
                TimeoutEvent -> Yellow,
                ChaosEvent -> Red,
            }
        ],
        // We can also define an unhandled_event block, which would be called when an event is not handled by the state
        unhandled_event: {
            match event {
                TrafficLightMachineEvent::EmergencyEvent(EmergencyEvent { requested_color }) => {
                    println!("{:?}: Emergency event not handled. Requested color: {:?}", Instant::now(), requested_color);
                    TrafficLightMachineState::from(requested_color)
                }
                _ => {
                    // Here, we've decided that all other unhanded events are the same as changing to the current state.
                    println!("{:?}: Unhandled event: {:?}", Instant::now(), event);
                    state
                }
            }
        }
    }
);

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum TrafficLightColor {
    #[default]
    Red,
    Yellow,
    Green,
}

impl From<TrafficLightColor> for TrafficLightMachineState {
    fn from(color: TrafficLightColor) -> Self {
        match color {
            TrafficLightColor::Red => Red {}.into(),
            TrafficLightColor::Yellow => Yellow {}.into(),
            TrafficLightColor::Green => Green {}.into(),
        }
    }
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
        .handle_event(ChaosEvent {}) // Color can Yellow or Red, since we're in Green
        .handle_event(EmergencyEvent {
            requested_color: TrafficLightColor::Red,
        })
        .tap(|x| {
            assert_eq!(
                x.state().color(),
                TrafficLightColor::Red,
                "Color should be red"
            );
        })
        .handle_event(EmergencyEvent {
            requested_color: TrafficLightColor::Yellow,
        })
        .tap(|x| {
            assert_eq!(
                x.state().color(),
                TrafficLightColor::Yellow,
                "Color should be yellow"
            );
        });
}
