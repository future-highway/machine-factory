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
/// Since the state machine is deterministic (i.e. there is
/// only one possible transition for each state),
/// the transitions are enforced at compile time.
#[proc_macro]
pub fn deterministic_state_machine(
    input: TokenStream,
) -> TokenStream {
    deterministic_state_machine::deterministic_state_machine(
        input,
    )
}
