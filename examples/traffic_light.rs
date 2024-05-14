#![allow(missing_docs)]
#![allow(clippy::print_stdout)]
#![allow(clippy::use_debug)]

use crate::state_machines::traffic_light::{
    ChaosEvent, EmergencyEvent, TimeoutEvent, TrafficLight,
    TrafficLightColor, TrafficLightMachineEvent,
    TrafficLightState,
};
use tap::Tap;

mod state_machines;

fn main() {
    let mut traffic_light = TrafficLight::default();

    _ = (&mut traffic_light)
        .tap(|x| {
            assert_eq!(
                x.color(),
                TrafficLightColor::Red,
                "Color should be red"
            );
        })
        .handle_event(
            TrafficLightMachineEvent::TimeoutEvent(
                TimeoutEvent {},
            ),
        )
        .tap(|x| {
            assert_eq!(
                x.color(),
                TrafficLightColor::Green,
                "Color should be green"
            );
        })
        .handle_event(TimeoutEvent {})
        .tap(|x| {
            assert_eq!(
                x.color(),
                TrafficLightColor::Yellow,
                "Color should be yellow"
            );
        })
        .handle_event(TimeoutEvent {})
        .tap(|x| {
            assert_eq!(
                x.color(),
                TrafficLightColor::Red,
                "Color should be red"
            );
        })
        .handle_event(TimeoutEvent {})
        .tap(|x| {
            assert_eq!(
                x.color(),
                TrafficLightColor::Green,
                "Color should be green"
            );
        })
        .handle_event(ChaosEvent {})
        .tap(|x| {
            assert!(
                matches!(
                    x.color(),
                    TrafficLightColor::Red
                        | TrafficLightColor::Yellow
                ),
                "Color should be red or yellow"
            );
        })
        .handle_event(EmergencyEvent {
            requested_color: TrafficLightColor::Red,
        })
        .tap(|x| {
            assert_eq!(
                x.color(),
                TrafficLightColor::Red,
                "Color should be red"
            );
        })
        .handle_event(EmergencyEvent {
            requested_color: TrafficLightColor::Yellow,
        })
        .tap(|x| {
            assert_eq!(
                x.color(),
                TrafficLightColor::Yellow,
                "Color should be yellow"
            );
        });

    let state = traffic_light.into_state();
    println!("Final state: {:?}", state.color());
}
