//! This example demonstrates a simple camera finite state
//! machine. The camera can be in one of two states: Standby
//! or Recording. The camera can receive two events:
//! `StartRecording` and `StopRecording`. When the camera is
//! in the Standby state, it can transition to the Recording
//! state by receiving the `StartRecording` event.
//! When the camera is in the Recording state, it can
//! transition to the Standby state by receiving the
//! `StopRecording` event. The camera records the total
//! number of seconds it has been in the Recording state.

#![allow(clippy::missing_trait_methods)]
#![allow(missing_docs)]
#![allow(clippy::print_stdout)]

use crate::state_machines::camera::{
    Camera, StartRecording, StopRecording,
};
use core::time::Duration;
use std::thread::sleep;

mod state_machines;

fn main() {
    let mut camera = Camera::default();

    _ = camera.handle_event(StartRecording {});
    sleep(Duration::from_secs(2));
    _ = camera.handle_event(StopRecording {});

    assert!(
        camera.context().total_recorded_seconds >= 2,
        "Expected at least 2 seconds of recording time, got {}",
        camera.context().total_recorded_seconds
    );

    let context = camera.into_context();

    println!(
        "Total recorded seconds: {}",
        context.total_recorded_seconds
    );
}
