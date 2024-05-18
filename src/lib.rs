//! Helper macros for generating state machines.

use proc_macro::TokenStream;

mod deterministic_state_machine;
mod event_driven_state_machine;
mod event_enum;
mod event_trait;
mod state_enum;
mod state_trait;

/// Build an event driven finite state machine.
#[proc_macro]
pub fn event_driven_state_machine(
    input: TokenStream,
) -> TokenStream {
    event_driven_state_machine::event_driven_state_machine(
        input,
    )
}

/// Build a deterministic finite state machine.
///
/// Since the state machine is deterministic, transitions
/// are enforced at compile time.
///
/// # Example
/// The following example defines a simple traffic-light
/// state machine. In each state, you can call the `change`
/// method to transition to the next state. The context
/// is updated with the number of times the light has
/// changed.
///
/// ```rust
/// # mod traffic_light {
/// use machine_factory::deterministic_state_machine;
///
/// deterministic_state_machine! {
///     #[derive(Debug)] // Attributes can optionally be added to the generated state machine's struct
///     #[derive(Clone)] // Multiple attributes are allowed
///     // Visibility and the struct keywords are also optional,
///     // and are followed by a required Identifier for the state machine
///     pub struct TrafficLight {
///         // Path to the context for the state machine
///         context: TrafficLightContext,
///         // Optionally specify a trait that all states must implement
///         state_trait: pub trait TrafficLightStateTrait {
///             fn color(&self) -> String;
///         },
///         // List of states and their transitions
///         states: [
///             // Each state's transitions are defined as functions in a block,
///             // similar to an impl block (which is what they expand to).
///             Red {
///                 pub fn change(self) -> TrafficLight<Green> {
///                    let Self { mut context, .. } = self;
///                    context.changes_count += 1;
///                    TrafficLight { context, state: Green }
///                 }
///             },
///             Yellow {
///                 pub fn change(self) -> TrafficLight<Red> {
///                     let Self { mut context, .. } = self;
///                     context.changes_count += 1;
///                     TrafficLight { context, state: Red }
///                 }
///
///                 // You can transition to multiple states from a single state
///                 // by defining multiple functions with different return types.
///                 pub fn back_to_green(self) -> TrafficLight<Green> {
///                     let Self { mut context, .. } = self;
///                     context.changes_count += 1;
///                     TrafficLight { context, state: Green }
///                 }
///             },
///             Green {
///                 pub fn change(self) -> TrafficLight<Yellow> {
///                     let Self { mut context, .. } = self;
///                     context.changes_count += 1;
///                     TrafficLight { context, state: Yellow }
///                 }
///
///                 // You can also define state-specific methods;
///                 // it isn't required that a transition occurs.
///                 // This function is only available when the
///                 // traffic-light is in the Green state.
///                 pub fn car_passed(&mut self) {
///                     self.context.car_count += 1;
///                 }
///             },
///         ],
///     }
/// }
///
/// // The context for the state machine.
/// #[derive(Debug, Clone)]
/// pub struct TrafficLightContext {
///     pub changes_count: u32,
///     pub car_count: u32,
/// }
///
/// // These are the states for the traffic light.
/// // They can have fields of their own, but, in this case, they don't.
///
/// pub struct Red;
/// pub struct Yellow;
/// pub struct Green;
///
/// // Since we specified the `TrafficLightStateTrait` trait, we must implement it for each state.
///
/// impl TrafficLightStateTrait for Red {
///     fn color(&self) -> String {
///         "Red".to_owned()
///     }
/// }
///
/// impl TrafficLightStateTrait for Yellow {
///     fn color(&self) -> String {
///         "Yellow".to_owned()
///     }
/// }
///
/// impl TrafficLightStateTrait for Green {
///     fn color(&self) -> String {
///         "Green".to_owned()
///     }
/// }
/// # }
///
/// // Now we can use the state machine.
/// # use traffic_light::*;
///
/// // Create a new traffic light in the Red state, with a starting context.
/// let traffic_light = TrafficLight::new(Red, TrafficLightContext {
///     changes_count: 0,
///     car_count: 0,
/// });
///
/// // We can access the context and state of the traffic light
/// // using the `context` and `state` methods, which return immutable references.
/// assert_eq!(traffic_light.context().changes_count, 0);
/// assert_eq!(traffic_light.context().car_count, 0);
/// assert_eq!(traffic_light.state().color(), "Red");
///
/// // Change the light to Green.
/// let mut traffic_light = traffic_light.change();
/// assert_eq!(traffic_light.context().changes_count, 1);
/// assert_eq!(traffic_light.context().car_count, 0);
/// assert_eq!(traffic_light.state().color(), "Green");
///
/// // A car passes, incrementing the car count.
/// traffic_light.car_passed();
/// assert_eq!(traffic_light.context().car_count, 1);
///
/// // Change the light to Yellow.
/// let traffic_light = traffic_light.change();
/// assert_eq!(traffic_light.context().changes_count, 2);
/// assert_eq!(traffic_light.context().car_count, 1);
/// assert_eq!(traffic_light.state().color(), "Yellow");
///
/// // This is a magic traffic-light where cars only pass when it's green,
/// // so we can't call `car_passed` here.
/// // traffic_light.car_passed();
///
/// // Change the light back to Green.
/// let traffic_light = traffic_light.back_to_green();
/// assert_eq!(traffic_light.context().changes_count, 3);
/// assert_eq!(traffic_light.context().car_count, 1);
/// assert_eq!(traffic_light.state().color(), "Green");
/// ```
#[proc_macro]
pub fn deterministic_state_machine(
    input: TokenStream,
) -> TokenStream {
    deterministic_state_machine::deterministic_state_machine(
        input,
    )
}
