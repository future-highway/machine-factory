#![allow(clippy::missing_trait_methods)]
#![allow(dead_code)] // there are false positives

use machine_factory::event_driven_finite_state_machine;
use std::time::Instant;

pub struct Storage {
    pub total_recorded_seconds: u64,
}

#[derive(Default)]
struct Standby;
impl CameraStateTrait for Standby {
    fn should_exit(
        &self,
        _context: &Storage,
        event: &CameraEvent,
    ) -> bool {
        matches!(event, CameraEvent::StartRecording(_))
    }
}

struct Recording {
    started_recording_at: Instant,
}

impl Default for Recording {
    fn default() -> Self {
        Self { started_recording_at: Instant::now() }
    }
}

impl CameraStateTrait for Recording {
    fn on_exit(&mut self, context: &mut Storage) {
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

event_driven_finite_state_machine!(Camera {
    context: Storage,
    state_enum: CameraState,
    state_trait: trait CameraStateTrait {},
    event_enum: CameraEvent,
    event_trait: #[allow(dead_code)]trait CameraEventTrait {},
    transitions: [
        Standby {
            StartRecording -> Recording,
        },
        Recording {
            StopRecording -> Standby,
        },
        _ { state }
    ]
});

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            Standby {},
            Storage { total_recorded_seconds: 0 },
        )
    }
}
