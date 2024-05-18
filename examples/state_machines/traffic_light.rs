#![allow(clippy::print_stdout)]
#![allow(clippy::use_debug)]
#![allow(clippy::missing_trait_methods)]
#![allow(dead_code)] // there are false positives

use machine_factory::event_driven_state_machine;
use std::time::Instant;

// First, we define the context that the traffic light will
// use
#[derive(Default)]
pub struct TrafficLightContext {
    #[allow(dead_code)] // seems like a false positive
    last_change: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct TimeoutEvent;

impl TrafficLightEvent for TimeoutEvent {
    // Here we can override the default implementation of
    // the pre_transition and post_transition methods,
    // as well as any other methods on the trait.
}

#[derive(Debug, Clone)]
pub struct EmergencyEvent {
    pub requested_color: TrafficLightColor,
}

impl TrafficLightEvent for EmergencyEvent {}

#[derive(Debug, Clone)]
pub struct ChaosEvent;
impl TrafficLightEvent for ChaosEvent {}

#[derive(Default, Debug, Clone)]
pub struct Red;

impl TrafficLightState for Red {
    // We can override the default implementation of the
    // on_enter and on_exit methods, as well as any
    // other methods on the trait.
    fn on_enter(
        &mut self,
        context: &mut TrafficLightContext,
    ) {
        context.last_change = Some(Instant::now());
        println!("{:?}: Changed to Red", Instant::now());
    }

    // This is required, since we don't provide a default
    // implementation in the trait
    fn color(&self) -> TrafficLightColor {
        TrafficLightColor::Red
    }
}

#[derive(Default, Debug, Clone)]
pub struct Yellow;

impl TrafficLightState for Yellow {
    fn color(&self) -> TrafficLightColor {
        TrafficLightColor::Yellow
    }
}

#[derive(Default, Debug, Clone)]
pub struct Green;

impl TrafficLightState for Green {
    fn color(&self) -> TrafficLightColor {
        TrafficLightColor::Green
    }
}

// Finally, we define the state machine
event_driven_state_machine!(
    pub TrafficLight {
        context: TrafficLightContext,
        event_trait:  trait TrafficLightEvent {},
        event_enum: #[derive(Debug, Clone)] TrafficLightMachineEvent,
        state_trait: pub trait TrafficLightState {
            fn on_enter(&mut self, context: &mut TrafficLightContext) {
                context.last_change = Some(Instant::now());
            }

            fn color(&self) -> TrafficLightColor;
        },
        state_enum: #[derive(Debug)] #[derive(Clone)] TrafficLightMachineState,
        states: [
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
                    if context.last_change.as_ref().is_some_and(|c| c.elapsed().as_secs() % 2 == 0) {
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
            },
            // We can also define an unhandled_event block, which would be called when an event is not handled by the state
            _ {
                if let TrafficLightMachineEvent::EmergencyEvent(EmergencyEvent { requested_color }) = event {
                    println!("{:?}: Emergency event not handled. Requested color: {:?}", Instant::now(), requested_color);
                    TrafficLightMachineState::from(&*requested_color)
                } else {
                    println!("{:?}: Unhandled event: {:?}", Instant::now(), event);
                    state
                }
            },
        ],
        // If we had events that weren't in the state blocks...
        // events: [OtherEvent],
    }
);

impl TrafficLight {
    pub fn color(&self) -> TrafficLightColor {
        self.state().color()
    }
}

impl Default for TrafficLight {
    fn default() -> Self {
        Self::new(Red, TrafficLightContext::default())
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrafficLightColor {
    #[default]
    Red,
    Yellow,
    Green,
}

impl From<&TrafficLightColor> for TrafficLightMachineState {
    fn from(color: &TrafficLightColor) -> Self {
        match color {
            TrafficLightColor::Red => Red {}.into(),
            TrafficLightColor::Yellow => Yellow {}.into(),
            TrafficLightColor::Green => Green {}.into(),
        }
    }
}
