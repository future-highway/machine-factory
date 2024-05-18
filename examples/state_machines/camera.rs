#![allow(clippy::missing_trait_methods)]
#![allow(dead_code)] // there are false positives

use async_trait::async_trait;
use machine_factory::event_driven_state_machine;
use std::time::Instant;

#[derive(Debug)]
pub struct Storage {
    pub total_recorded_seconds: u64,
}

#[derive(Default, Debug)]
pub struct Standby;

// We are going to make `Camera` async, so we need to add
// the `#[async_trait]` attribute
#[async_trait]
impl CameraStateTrait for Standby {
    fn should_exit(
        &self,
        _context: &Storage,
        event: &CameraEvent,
    ) -> bool {
        matches!(event, CameraEvent::StartRecording(_))
    }
}

#[derive(Debug)]
pub struct Recording {
    started_recording_at: Instant,
}

impl Default for Recording {
    fn default() -> Self {
        Self { started_recording_at: Instant::now() }
    }
}

#[async_trait]
impl CameraStateTrait for Recording {
    async fn on_exit(&mut self, context: &mut Storage) {
        context.total_recorded_seconds =
            context.total_recorded_seconds.saturating_add(
                self.started_recording_at
                    .elapsed()
                    .as_secs(),
            );
    }

    fn should_exit(
        &self,
        _context: &Storage,
        event: &CameraEvent,
    ) -> bool {
        matches!(event, CameraEvent::StopRecording(_))
    }
}

#[derive(Debug, Clone)]
pub struct StartRecording;
impl CameraEventTrait for StartRecording {}

#[derive(Debug, Clone)]
pub struct StopRecording;
impl CameraEventTrait for StopRecording {}

event_driven_state_machine!(
    #[derive(Debug)]
    pub async Camera {
        context: Storage,
        state_enum: #[derive(Debug)] CameraState,
        state_trait: trait CameraStateTrait {
            // The camera is defaulting to `async`, so we override the default
            // `should_exit` method to not be async, as it's not needed.
            fn should_exit(
                &self,
                context: &Storage,
                event: &CameraEvent,
            ) -> bool;
        },
        event_enum: CameraEvent,
        event_trait: trait CameraEventTrait: Send {},
        states: [
            Standby {
                StartRecording -> Recording,
            },
            Recording {
                StopRecording -> Standby,
            },
            _ { state }
        ]
    }
);

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            Standby {},
            Storage { total_recorded_seconds: 0 },
        )
    }
}
