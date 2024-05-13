//! Helper macros for generating state machines.

use proc_macro::TokenStream;

mod event_driven_finite_state_machine;
mod event_enum;
mod event_trait;
mod state_enum;
mod state_trait;

/// Define an event driven finite state machine.
#[proc_macro]
pub fn event_driven_finite_state_machine(input: TokenStream) -> TokenStream {
    event_driven_finite_state_machine::event_driven_finite_state_machine(input)
}
