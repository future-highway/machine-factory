#![allow(missing_docs)]

use crate::state_machines::traffic_light::TrafficLightColor;
use state_machines::traffic_light::{
    ChaosEvent, EmergencyEvent, TimeoutEvent, TrafficLight, TrafficLightMachineEvent,
};
use tap::Tap;

mod state_machines;

fn main() {
    let traffic_light = TrafficLight::default();

    _ = traffic_light
        .tap(|x| {
            assert_eq!(x.color(), TrafficLightColor::Red, "Color should be red");
        })
        .handle_event(TrafficLightMachineEvent::TimeoutEvent(TimeoutEvent {}))
        .tap(|x| {
            assert_eq!(x.color(), TrafficLightColor::Green, "Color should be green");
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
            assert_eq!(x.color(), TrafficLightColor::Red, "Color should be red");
        })
        .handle_event(TimeoutEvent {})
        .tap(|x| {
            assert_eq!(x.color(), TrafficLightColor::Green, "Color should be green");
        })
        .handle_event(ChaosEvent {})
        .tap(|x| {
            assert!(
                matches!(
                    x.color(),
                    TrafficLightColor::Red | TrafficLightColor::Yellow
                ),
                "Color should be red or yellow"
            );
        })
        .handle_event(EmergencyEvent {
            requested_color: TrafficLightColor::Red,
        })
        .tap(|x| {
            assert_eq!(x.color(), TrafficLightColor::Red, "Color should be red");
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
}
