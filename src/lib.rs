use proc_macro::TokenStream;

mod event_enum;
mod event_trait;
mod state_enum;
mod state_machine;
mod state_trait;

/// For internal use only. Will be removed in the future.
#[doc(hidden)]
#[proc_macro_attribute]
pub fn event_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    event_trait::event_trait(attr, item)
}

#[proc_macro]
pub fn state_machine(input: TokenStream) -> TokenStream {
    state_machine::state_machine(input)
}

/// For internal use only. Will be removed in the future.
#[doc(hidden)]
#[proc_macro_attribute]
pub fn state_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    state_trait::state_trait(attr, item)
}
