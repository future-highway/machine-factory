//! This example demonstrates a traffic light that records
//! when in the `Red` state. It uses a nested state machine
//! to control a camera that starts recording when the
//! traffic light is red. See the [`camera`] module for the
//! camera state machine.
#![allow(clippy::missing_trait_methods)]
#![allow(missing_docs)]
#![allow(clippy::print_stdout)]

use async_trait::async_trait;
use core::time::Duration;
use machine_factory::event_driven_state_machine;
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

#[async_trait]
impl TrafficLightStateTrait for Red {
    async fn on_enter(&mut self, context: &mut Context) {
        _ = context
            .camera
            .handle_event(StartRecording {})
            .await;
    }

    async fn on_exit(&mut self, context: &mut Context) {
        _ = context
            .camera
            .handle_event(StopRecording {})
            .await;
    }
}

#[derive(Default)]
struct Yellow;

#[async_trait]
impl TrafficLightStateTrait for Yellow {
    async fn should_exit(
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

#[async_trait]
impl TrafficLightStateTrait for Green {
    async fn should_exit(
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

event_driven_state_machine!(async TrafficLight {
    context: Context ,
    state_enum: TrafficLightState,
    state_trait: trait TrafficLightStateTrait {},
    event_enum: TrafficLightEvent,
    event_trait: trait TrafficLightEventTrait: Send {},
    transitions: [
        Red {
            Next -> Green,
            StopRecording {
                _ = context.camera.handle_event(event.clone()).await;
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

#[tokio::main]
async fn main() {
    let mut context = Context { camera: Camera::default() };

    // `on_enter` is not automatically called when the state
    // machine is initialized, so we need to call it
    // manually.
    let mut state = Red {};
    state.on_enter(&mut context).await;

    let mut traffic_light = TrafficLight::new(Red, context);

    let mut count = 0_i32;
    while count < 10_i32 {
        _ = traffic_light.handle_event(Next {}).await;
        count = count.saturating_add(1);
        sleep(Duration::from_secs(1));
    }

    _ = traffic_light.handle_event(StopRecording {}).await;

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
