//! Helper macros for generating state machines.

use proc_macro::TokenStream;

mod deterministic_state_machine;
mod event_driven_state_machine;
mod event_enum;
mod event_trait;
mod state_enum;
mod state_trait;

/// Build an event driven finite state machine.
///
/// To raise an event, call the `handle_event` method on the
/// state machine with the event as an argument. When an
/// event is handled, a series of functions are called in
/// the following order:
///
/// ```ignore
/// CurrentState::should_exit(&self, &context, &event)
/// CurrentState::on_exit(&mut self, &mut context)
/// Event::pre_transition(&mut self, &mut context)
/// {
///     // TransitionBlock defined in the state machine macro
///     // that must return the new state.
///     // The following variables are available:
///     // - `mut state`: the current state
///     // - `&mut context`: the context
///     // - `&mut event`: the event
/// }
/// Event::post_transition(&mut self, &mut context)
/// NewState::on_enter(&mut self, &mut context)
/// ```
///
/// # Syntax
/// ```text
/// event_driven_state_machine! {
///     [ Attribute [ Attribute ]* ]
///     [ Visibility ] [ async ] [ struct ] Identifier {
///         context: Path,
///         state_enum: [ Attribute [ Attribute ]* ] Identifier,
///         state_trait: Trait,
///         event_enum: [ Attribute [ Attribute ]* ] Identifier,
///         event_trait: Trait,
///         states: LeftBracket
///             [ StateTransition [, StateTransition]* ]
///             [ _ { DefaultTransitionBlock } ]
///         RightBracket,
///         events: LeftBracket
///            [ Path [, Path]* ]
///         RightBracket,
///     }
/// }
///
/// Attribute = a valid Rust outer attribute (e.g., `#[derive(Debug)]`)
/// Visibility = a valid Rust visibility modifier (e.g., `pub`, `pub(crate)`, etc.)
/// Identifier = a valid Rust identifier (e.g., `MyStateMachine`)
/// Path = a valid Rust path (e.g., `crate::MyContext` or `MyContext`)
/// Trait = a valid Rust trait definition (e.g., `pub trait MyTrait { ... }`)
/// LeftBracket = [
/// RightBracket = ]
/// StateTransition = Path { [ DefaultTransition | TransitionBlock [, DefaultTransition | TransitionBlock ]* ] }
/// DefaultTransition = Path -> Path
/// TransitionBlock = Path { ... } (where `...` is a block of Rust code that returns a state)
/// DefaultTransitionBlock = 1 or more Rust statements that return a state
/// ```
///
/// # Example
/// The following example defines a traffic-light state
/// machine, slightly more advanced than the one defined in
/// the docs for the
/// [`deterministic_state_machine!`] macro. This state
/// machine will receive events to determine state
/// transitions.
///
/// *The point of the example is to show how to use the
/// `event_driven_state_machine!` macro, and isn't
/// the best or even a complete way to model a traffic
/// light.*
///
/// ```rust
/// # mod traffic_light {
/// use machine_factory::event_driven_state_machine;
///
/// event_driven_state_machine! {
///     #[derive(Debug)] // Attributes can optionally be added to the generated state machine's struct
///     #[derive(Clone)] // Multiple attributes are allowed
///     // Visibility, `async`, and `struct` are optional,
///     // and are followed by a required Identifier for the state machine.
///     // The async keyword makes the expanded methods async by default.
///     pub async struct TrafficLight {
///         // Path to the context for the state machine
///         context: TrafficLightContext,
///         // Identifier for the generated enum of states
///         // Attributes can also optionally be added to the generated enum
///         state_enum: #[derive(Debug, Clone)] TrafficLightState,
///         // Define a trait that all states must implement.
///         // The macro will automatically define the required methods
///         // (i.e., `on_enter`, `on_exit`, `should_exit`), with default implementations,
///         // if they are not provided.
///         state_trait: pub trait TrafficLightStateTrait {
///             // The traffic-light is defaulting to `async`, so we override the default
///             // `should_exit` method to not be async, and provide a default implementation.
///             fn should_exit(
///                 &self,
///                 context: &TrafficLightContext,
///                 event: &TrafficLightEvent,
///             ) -> bool {
///                 if context.in_emergency {
///                     // If we're in an emergency, we don't want to change the state,
///                     // unless the vehicle passing is an emergency vehicle.
///                     return matches!(event, TrafficLightEvent::CarPassed(CarPassed { emergency_vehicle: true }));
///                 }
///                 
///                 true
///             }
///
///             // We can also define additional methods on the trait.
///             fn color(&self) -> TrafficLightColor;
///         },
///         // Identifier for the generated enum of events
///         // Attributes can also optionally be added to the generated enum, like with `state_enum`
///         event_enum: TrafficLightEvent,
///         // Define a trait that all events must implement.
///         // The macro will automatically define the required methods
///         // (i.e., `pre_transition`, `post_transtion`), with default implementations,
///         // if they are not provided.
///         event_trait: pub trait TrafficLightEventTrait: Send {
///             // We've added a 'Send' bound to the trait, which is required because
///             // the `pre_transition` and `post_transition` methods are async,
///             // and the events are held across async boundaries.
///
///             // We can also define additional methods on the trait.
///             fn is_emergency(&self) -> bool {
///                 false
///             }
///         },
///         // List of transitions for the state machine
///         states: [
///             Red {
///                 // This is a transition from the `Red` state to the `Green` state,
///                 // triggered by the `Next` event.
///                 // Since a body is not provided, `Green::default()` is used to create the new state.
///                 Next -> Green,
///                 EmergencyVehicleApproaching {
///                     // We can use logic to decide which state to transition to.
///                     // We have access to the current state (`mut state`), the context (`&mut context`),
///                     // and the event (`&mut event`).
///                     // Here, we're accessing data on the event to make our decision.
///                     match event.requested_color {
///                         TrafficLightColor::Red => {
///                             // Since we may be returning any state, we need to convert it to the state enum.
///                             // The `from` trait is automatically implemented for each state for the state enum.
///                             TrafficLightState::from(state)
///                         },
///                         TrafficLightColor::Yellow => {
///                             Yellow {}.into()
///                         }
///                         TrafficLightColor::Green => {
///                             Green {}.into()
///                         }
///                     }
///                 },
///             },
///             Yellow {
///                 Next -> Red,
///             },
///             Green {
///                Next -> Yellow,
///             },
///             // You can optionally define a catch-all transition that applies to all state/event pairs not explicitly defined.
///             _ {
///                 if let TrafficLightEvent::EmergencyVehicleApproaching(EmergencyVehicleApproaching { requested_color }) = event {
///                     context.in_emergency = true;
///                     TrafficLightState::from(*requested_color)
///                 } else {
///                     // We are going to remain in the same state.
///                     // This doesn't short-circit the transition like `should_exit` can.
///                     // `on_exit` and `pre_transition` have already been called, and
///                     // `post_transition` and `on_enter` will be called
///                     state
///                 }
///             }
///         ],
///         // Optionally, you can add other events that can be raised on the state machine.
///         // If the event appears in the `states` block, it does not need to be defined here.
///         events: [
///            CarPassed, // No matter the state, the default transition block will be used for this event
///         ],
///     }
/// }
///
/// // Let's define a default for the traffic light.
/// impl Default for TrafficLight {
///     fn default() -> Self {
///         TrafficLight::new(Red, TrafficLightContext::default())
///     }
/// }
///
/// // The context for the state machine.
/// #[derive(Debug, Clone, Default)]
/// pub struct TrafficLightContext {
///    pub cars_count: u32,
///    pub in_emergency: bool,
/// }
///
/// // These are the states for the traffic light.
/// // They can have fields of their own, but, in this case, they don't.
///
/// #[derive(Debug, Clone, Default)]
/// struct Red;
///
/// #[derive(Debug, Clone, Default)]
/// struct Yellow;
///
/// #[derive(Debug, Clone, Default)]
/// struct Green;
///
/// // Implement the state trait for each state.
/// impl TrafficLightStateTrait for Red {
///    fn color(&self) -> TrafficLightColor {
///       TrafficLightColor::Red
///   }
/// }
///
/// impl TrafficLightStateTrait for Yellow {
///     fn color(&self) -> TrafficLightColor {
///         TrafficLightColor::Yellow
///     }
/// }
///
/// impl TrafficLightStateTrait for Green {
///     fn color(&self) -> TrafficLightColor {
///         TrafficLightColor::Green
///     }
/// }
///
/// // These are the events that can be raised on the traffic light.
/// pub struct Next;
///
/// pub struct CarPassed {
///     pub emergency_vehicle: bool
/// }
///
/// pub struct EmergencyVehicleApproaching {
///    pub requested_color: TrafficLightColor,
/// }
///
/// // Implement the event trait for each event.
/// impl TrafficLightEventTrait for Next {}
///
/// // Without `#[async_trait::async_trait]`, you'll see a weird lifetime error:
/// // "lifetimes do not match in trait"
/// #[async_trait::async_trait]
/// impl TrafficLightEventTrait for CarPassed {
///     async fn pre_transition(&mut self, context: &mut TrafficLightContext) {
///         context.cars_count += 1;
///         println!("Car passed: {}", context.cars_count);
///         // We could also await asynchronous work here
///     }
///
///     async fn post_transition(&mut self, context: &mut TrafficLightContext) {
///         if self.emergency_vehicle {
///             context.in_emergency = false;
///         }
///     }
/// }
///
/// impl TrafficLightEventTrait for EmergencyVehicleApproaching {
///     fn is_emergency(&self) -> bool {
///         true
///     }
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum TrafficLightColor {
///    Red,
///    Yellow,
///    Green,
/// }
///
/// impl From<TrafficLightColor> for TrafficLightState {
///     fn from(color: TrafficLightColor) -> Self {
///         match color {
///             TrafficLightColor::Red => Red {}.into(),
///             TrafficLightColor::Yellow => Yellow {}.into(),
///             TrafficLightColor::Green => Green {}.into(),
///         }    
///     }
/// }
///
/// # }
/// # use traffic_light::*;
///
/// #[tokio::main]
/// async fn main() {
///     // Create a new traffic light in the Red state, with a starting context.
///     let mut traffic_light = TrafficLight::default();
///
///     // We can access the context and state of the traffic light
///     // using the `context` and `state` methods, which return immutable references.
///     assert_eq!(traffic_light.context().cars_count, 0);
///     assert_eq!(traffic_light.context().in_emergency, false);
///     assert_eq!(traffic_light.state().color(), TrafficLightColor::Red);
///
///     // A car passes:
///     traffic_light.handle_event(CarPassed { emergency_vehicle: false }).await;
///     assert_eq!(traffic_light.context().cars_count, 1);
///
///     traffic_light.handle_event(Next).await;
///     assert_eq!(traffic_light.state().color(), TrafficLightColor::Green);
///     assert_eq!(traffic_light.context().cars_count, 1);
///
///     // Emergency event
///     traffic_light.handle_event(EmergencyVehicleApproaching { requested_color: TrafficLightColor::Red }).await;
///     assert_eq!(traffic_light.state().color(), TrafficLightColor::Red);
///     assert_eq!(traffic_light.context().in_emergency, true);
///
///     // Car passes
///     traffic_light.handle_event(CarPassed { emergency_vehicle: false }).await;
///
///     // Notice that cars_count is not incremented because the traffic light is in emergency,
///     // which makes `should_exit` return false, preventing the transition, and the event's pre/post_transition methods from being called.
///     assert_eq!(traffic_light.context().cars_count, 1);
///     assert_eq!(traffic_light.context().in_emergency, true);
///
///     // Emergency vehicle passes
///     traffic_light.handle_event(CarPassed { emergency_vehicle: true }).await;
///     assert_eq!(traffic_light.context().cars_count, 2);
///     assert_eq!(traffic_light.context().in_emergency, false);
/// }
/// ```
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
/// # Syntax
/// ```text
/// deterministic_state_machine! {
///     [ Attribute [ Attribute ]* ]
///     [ Visibility ] [ struct ] Identifier {
///         context: Path,
///       [ state_trait: [ Trait ], ]
///         states: LeftBracket
///            [ StateTransition [, StateTransition]* ]
///         RightBracket,
///     }
/// }
///
/// Attribute = a valid Rust outer attribute (e.g., `#[derive(Debug)]`)
/// Visibility = a valid Rust visibility modifier (e.g., `pub`, `pub(crate)`, etc.)
/// Identifier = a valid Rust identifier (e.g., `MyStateMachine`)
/// Path = a valid Rust path (e.g., `crate::MyContext` or `MyContext`)
/// Trait = a valid Rust trait definition (e.g., `pub trait MyTrait { ... }`)
/// LeftBracket = [
/// RightBracket = ]
/// StateTransition = Path { [ Function ]* }
/// Function = a valid Rust function definition (e.g., `pub fn my_transition(self) -> MyNextState  { ... }`)
/// ```
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
