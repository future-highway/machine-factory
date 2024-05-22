Helper macros for generating state machines.

# Examples

See examples folder for more complete implementations and usage.

## Deterministic State Machine

```rust
deterministic_state_machine! {
    #[derive(Debug)] // Attributes can optionally be added to the generated state machine's struct
    #[derive(Clone)] // Multiple attributes are allowed
    // Visibility and the struct keywords are also optional,
    // and are followed by a required Identifier for the state machine
    pub struct TrafficLight {
        // Path to the context for the state machine
        context: TrafficLightContext,
        // Optionally specify a trait that all states must implement
        state_trait: pub trait TrafficLightStateTrait {
            fn color(&self) -> String;
        },
        // List of states and their transitions
        states: [
            // Each state's transitions are defined as functions in a block,
            // similar to an impl block (which is what they expand to).
            Red {
                pub fn change(self) -> TrafficLight<Green> {
                   let Self { mut context, .. } = self;
                   context.changes_count += 1;
                   TrafficLight { context, state: Green }
                }
            },
            Yellow {
                pub fn change(self) -> TrafficLight<Red> {
                    let Self { mut context, .. } = self;
                    context.changes_count += 1;
                    TrafficLight { context, state: Red }
                }

                // You can transition to multiple states from a single state
                // by defining multiple functions with different return types.
                pub fn back_to_green(self) -> TrafficLight<Green> {
                    let Self { mut context, .. } = self;
                    context.changes_count += 1;
                    TrafficLight { context, state: Green }
                }
            },
            Green {
                pub fn change(self) -> TrafficLight<Yellow> {
                    let Self { mut context, .. } = self;
                    context.changes_count += 1;
                    TrafficLight { context, state: Yellow }
                }

                // You can also define state-specific methods;
                // it isn't required that a transition occurs.
                // This function is only available when the
                // traffic-light is in the Green state.
                pub fn car_passed(&mut self) {
                    self.context.car_count += 1;
                }
            },
        ],
    }
}
```

## Event Driven State Machine

```rust
event_driven_state_machine! {
    #[derive(Debug)] // Attributes can optionally be added to the generated state machine's struct
    #[derive(Clone)] // Multiple attributes are allowed
    // Visibility, `async`, and `struct` are optional,
    // and are followed by a required Identifier for the state machine.
    // The async keyword makes the expanded methods async by default.
    pub async struct TrafficLight {
        // Path to the context for the state machine
        context: TrafficLightContext,
        // Identifier for the generated enum of states
        // Attributes can also optionally be added to the generated enum
        state_enum: #[derive(Debug, Clone)] TrafficLightState,
        // Define a trait that all states must implement.
        // The macro will automatically define the required methods
        // (i.e., `on_enter`, `on_exit`, `should_exit`), with default implementations,
        // if they are not provided.
        state_trait: pub trait TrafficLightStateTrait {
            // The traffic-light is defaulting to `async`, so we override the default
            // `should_exit` method to not be async, and provide a default implementation.
            fn should_exit(
                &self,
                context: &TrafficLightContext,
                event: &TrafficLightEvent,
            ) -> bool {
                if context.in_emergency {
                    // If we're in an emergency, we don't want to change the state,
                    // unless the vehicle passing is an emergency vehicle.
                    return matches!(event, TrafficLightEvent::CarPassed(CarPassed { emergency_vehicle: true }));
                }
                
                true
            }

            // We can also define additional methods on the trait.
            fn color(&self) -> TrafficLightColor;
        },
        // Identifier for the generated enum of events
        // Attributes can also optionally be added to the generated enum, like with `state_enum`
        event_enum: TrafficLightEvent,
        // Define a trait that all events must implement.
        // The macro will automatically define the required methods
        // (i.e., `pre_transition`, `post_transtion`), with default implementations,
        // if they are not provided.
        event_trait: pub trait TrafficLightEventTrait: Send {
            // We've added a 'Send' bound to the trait, which is required because
            // the `pre_transition` and `post_transition` methods are async,
            // and the events are held across async boundaries.

            // We can also define additional methods on the trait.
            fn is_emergency(&self) -> bool {
                false
            }
        },
        // List of transitions for the state machine
        states: [
            Red {
                // This is a transition from the `Red` state to the `Green` state,
                // triggered by the `Next` event.
                // Since a body is not provided, `Green::default()` is used to create the new state.
                Next -> Green,
                EmergencyVehicleApproaching {
                    // We can use logic to decide which state to transition to.
                    // We have access to the current state (`mut state`), the context (`&mut context`),
                    // and the event (`&mut event`).
                    // Here, we're accessing data on the event to make our decision.
                    match event.requested_color {
                        TrafficLightColor::Red => {
                            // Since we may be returning any state, we need to convert it to the state enum.
                            // The `from` trait is automatically implemented for each state for the state enum.
                            TrafficLightState::from(state)
                        },
                        TrafficLightColor::Yellow => {
                            Yellow {}.into()
                        }
                        TrafficLightColor::Green => {
                            Green {}.into()
                        }
                    }
                },
            },
            Yellow {
                Next -> Red,
            },
            Green {
               Next -> Yellow,
            },
            // You can optionally define a catch-all transition that applies to all state/event pairs not explicitly defined.
            _ {
                if let TrafficLightEvent::EmergencyVehicleApproaching(EmergencyVehicleApproaching { requested_color }) = event {
                    context.in_emergency = true;
                    TrafficLightState::from(*requested_color)
                } else {
                    // We are going to remain in the same state.
                    // This doesn't short-circit the transition like `should_exit` can.
                    // `on_exit` and `pre_transition` have already been called, and
                    // `post_transition` and `on_enter` will be called
                    state
                }
            }
        ],
        // Optionally, you can add other events that can be raised on the state machine.
        // If the event appears in the `states` block, it does not need to be defined here.
        events: [
           CarPassed, // No matter the state, the default transition block will be used for this event
        ],
    }
}
```