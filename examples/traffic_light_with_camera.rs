//! This example demonstrates a traffic light that records
//! when in the `Red` state. It uses a nested state machine
//! to control a camera that starts recording when the
//! traffic light is red. See the [`camera`] module for the
//! camera state machine.
#![allow(clippy::missing_trait_methods)]
#![allow(missing_docs)]
#![allow(clippy::print_stdout)]

use core::time::Duration;
use machine_factory::event_driven_finite_state_machine;
use state_machines::camera::{
    Camera, StartRecording, StopRecording,
};
use std::thread::sleep;

mod state_machines;

struct Context {
    camera: Camera,
}

// States
#[derive(Default)]
struct Red;
impl TrafficLightStateTrait for Red {
    fn on_enter(&mut self, context: &mut Context) {
        _ = context.camera.handle_event(StartRecording {});
    }

    fn on_exit(&mut self, context: &mut Context) {
        _ = context.camera.handle_event(StopRecording {});
    }
}

#[derive(Default)]
struct Yellow;
impl TrafficLightStateTrait for Yellow {
    fn should_exit(
        &self,
        _context: &Context,
        event: &TrafficLightEvent,
    ) -> bool {
        !matches!(
            event,
            TrafficLightEvent::StopRecording(_)
        )
    }
}

#[derive(Default)]
struct Green;
impl TrafficLightStateTrait for Green {
    fn should_exit(
        &self,
        _context: &Context,
        event: &TrafficLightEvent,
    ) -> bool {
        !matches!(
            event,
            TrafficLightEvent::StopRecording(_)
        )
    }
}

// Events
struct Next;
impl TrafficLightEventTrait for Next {}

impl TrafficLightEventTrait for StopRecording {}

event_driven_finite_state_machine!(TrafficLight {
    context: Context ,
    state_enum: TrafficLightState,
    state_trait: trait TrafficLightStateTrait {},
    event_enum: TrafficLightEvent,
    event_trait: trait TrafficLightEventTrait {},
    transitions: [
        Red {
            Next -> Green,
            StopRecording {
                _ = context.camera.handle_event(event.clone());
                state
            }
        },
        Yellow {
            Next -> Red,
        },
        Green {
            Next -> Yellow,
        },
        _ {
            state
        },
    ],
});

fn main() {
    let mut context = Context { camera: Camera::default() };

    // `on_enter` is not automatically called when the state
    // machine is initialized, so we need to call it
    // manually.
    let mut state = Red {};
    state.on_enter(&mut context);

    let mut traffic_light =
        TrafficLight::init(Red, context);

    let mut count = 0_i32;
    while count < 10_i32 {
        _ = traffic_light.handle_event(Next {});
        count = count.saturating_add(1);
        sleep(Duration::from_secs(1));
    }

    _ = traffic_light.handle_event(StopRecording {});

    println!("Traffic light stopped recording.");
    println!(
        "Recorded {} seconds.",
        traffic_light
            .context()
            .camera
            .context()
            .total_recorded_seconds
    );
}
