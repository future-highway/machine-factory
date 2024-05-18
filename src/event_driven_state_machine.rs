use crate::{
    event_enum::{event_enum, EventEnumInput},
    event_trait::ensure_event_trait,
    state_enum::{state_enum, StateEnumInput},
    state_trait::ensure_state_trait,
};
use core::iter::once;
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Async, Brace, Comma},
    Attribute, Block, FnArg, Ident, Path, Token, TraitItem,
    Visibility,
};

struct Machine {
    attributes: Vec<Attribute>,
    visibility: Option<Visibility>,
    asyncness: Option<Async>,
    name: Ident,
    context_path: Path,
    state_enum_attrs: Vec<Attribute>,
    state_enum_ident: Ident,
    state_trait: syn::ItemTrait,
    event_enum_attrs: Vec<Attribute>,
    event_enum_ident: Ident,
    event_trait: syn::ItemTrait,
    state_transitions: Vec<StateTransitions>,
    other_events: Vec<Path>,
}

impl Parse for Machine {
    #[allow(clippy::too_many_lines)]
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attributes = input
            .peek(Token![#])
            .then(|| Attribute::parse_outer(input))
            .transpose()?
            .unwrap_or_default();

        let visibility = input
            .peek(Token![pub])
            .then(|| input.parse())
            .transpose()?;

        let asyncness = input
            .peek(Token![async])
            .then(|| input.parse::<Token![async]>().expect("peeked async"));

        _ = input.peek(Token![struct]).then(|| input.parse::<Token![struct]>()).transpose()?;

        let name: Ident = input.parse()?;

        let content;
        _ = braced!(content in input);

        let mut context_path = None;
        let mut state_enum_attrs = None;
        let mut state_enum_ident = None;
        let mut state_trait = None;
        let mut event_enum_attrs = None;
        let mut event_enum_ident = None;
        let mut event_trait_path = None;
        let mut state_transitions = None;
        let mut other_events = None;

        while content.peek(Ident) {
            let label: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;

            match label.to_string().as_str() {
                "context" => {
                    context_path = Some(content.parse()?);
                }
                "state_enum" => {
                    if content.peek(Token![#]) {
                        state_enum_attrs = Some(Attribute::parse_outer(&content)?);
                    }

                    state_enum_ident = Some(content.parse()?);
                }
                "state_trait" => {
                    state_trait = Some(content.parse()?);
                }
                "event_enum" => {
                    if content.peek(Token![#]) {
                        event_enum_attrs = Some(Attribute::parse_outer(&content)?);
                    }

                    event_enum_ident = Some(content.parse()?);
                }
                "event_trait" => {
                    event_trait_path = Some(content.parse()?);
                }
                "states" => {
                    let content2;
                    let _ = bracketed!(content2 in content);
                    let parsed_transitions =
                        Punctuated::<
                            StateTransitions,
                            Comma,
                        >::parse_terminated(
                            &content2
                        )?;

                    let parsed_transitions =
                        parsed_transitions
                            .into_iter()
                            .collect();

                    state_transitions = Some(parsed_transitions);
                }
                "events" => {
                    let content2;
                    let _ = bracketed!(content2 in content);
                    let parsed_events =
                        Punctuated::<
                            Path,
                            Comma,
                        >::parse_terminated(
                            &content2
                        )?;

                    let parsed_events =
                        parsed_events
                            .into_iter()
                            .collect();

                    other_events = Some(parsed_events);
                }
                _ => {
                    return Err(syn::Error::new(
                        label.span(),
                        "unrecognized label",
                    ));
                }
            }

            if content.peek(Comma) {
                let _: Comma = content.parse()?;
            }
        }

        let context_path =
            context_path.ok_or_else(|| {
                syn::Error::new(
                    name.span(),
                    "machine is missing context",
                )
            })?;

        let state_enum_ident = state_enum_ident
            .ok_or_else(|| {
                syn::Error::new(
                    name.span(),
                    "machine is missing state_enum",
                )
            })?;

        let state_trait = state_trait.ok_or_else(|| {
            syn::Error::new(
                name.span(),
                "machine is missing state_trait",
            )
        })?;

        let event_enum_ident = event_enum_ident
            .ok_or_else(|| {
                syn::Error::new(
                    name.span(),
                    "machine is missing event_enum",
                )
            })?;

        let event_trait =
            event_trait_path.ok_or_else(|| {
                syn::Error::new(
                    name.span(),
                    "machine is missing event_trait",
                )
            })?;

        let state_transitions = state_transitions.ok_or_else(|| {
            syn::Error::new(
                name.span(),
                "machine is missing states",
            )
        })?;

        Ok(Self {
            attributes,
            visibility,
            asyncness,
            name,
            context_path,
            state_enum_attrs: state_enum_attrs
                .unwrap_or_default(),
            state_enum_ident,
            state_trait,
            event_enum_attrs: event_enum_attrs
                .unwrap_or_default(),
            event_enum_ident,
            event_trait,
            state_transitions,
            other_events: other_events.unwrap_or_default(),
        })
    }
}

enum StateTransitions {
    Default(Block),
    State(StateStateTransitions),
}

struct StateStateTransitions {
    state_path: Path,
    transitions: Vec<Transition>,
}

impl Parse for StateTransitions {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Token![_]) {
            let _: Token![_] = input.parse()?;
            let block = input.parse()?;
            Ok(Self::Default(block))
        } else {
            let state_path = input.parse()?;

            let content;
            _ = braced!(content in input);
            let transitions = Punctuated::<
                Transition,
                Comma,
            >::parse_terminated(
                &content
            )?;
            let transitions =
                transitions.into_iter().collect();

            Ok(Self::State(StateStateTransitions {
                state_path,
                transitions,
            }))
        }
    }
}

struct Transition {
    event_path: Path,
    block: TransitionBlock,
}

enum TransitionBlock {
    Default(Path),
    Block(Block),
}

impl Parse for Transition {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let event = input.parse()?;

        let block = if input.peek(Token![->]) {
            let _: Token![->] = input.parse()?;
            let target = input.parse()?;
            TransitionBlock::Default(target)
        } else if input.peek(Brace) {
            let block = input.parse()?;
            TransitionBlock::Block(block)
        } else {
            return Err(syn::Error::new(
                input.span(),
                "expected -> or {",
            ));
        };

        Ok(Self { event_path: event, block })
    }
}

struct StateEvent {
    state_path: Path,
    state_ident: Ident,
    event_path: Path,
    event_ident: Ident,
    block: Block,
    is_default: bool,
}

#[allow(clippy::too_many_lines)]
pub(super) fn event_driven_state_machine(
    input: TokenStream,
) -> TokenStream {
    let Machine {
        attributes,
        visibility,
        asyncness,
        name,
        context_path,
        state_enum_attrs,
        state_enum_ident,
        mut state_trait,
        event_enum_attrs,
        event_enum_ident,
        mut event_trait,
        state_transitions,
        other_events,
    } = parse_macro_input!(input as Machine);

    let async_postfix = asyncness.is_some().then(|| quote!(.await));

    if let Err(e) = ensure_state_trait(
        asyncness,
        &mut state_trait,
        &context_path,
        &event_enum_ident,
    ) {
        return e.to_compile_error().into();
    }

    let state_trait_path = &state_trait.ident;

    if let Err(e) =
        ensure_event_trait(asyncness, &mut event_trait, &context_path)
    {
        return e.to_compile_error().into();
    }

    let async_trait_attr: Attribute = parse_quote!(#[::async_trait::async_trait]);
    let maybe_async_trait_attr = asyncness.is_some().then_some(&async_trait_attr);

    if asyncness.is_some() {
        state_trait.attrs.push(async_trait_attr.clone());
        event_trait.attrs.push(async_trait_attr.clone());
    }

    let event_trait_path = &event_trait.ident;

    let unhandled_event = {
        let mut unhandled_event = state_transitions
            .iter()
            .filter_map(|transition| {
                if let StateTransitions::Default(block) =
                    transition
                {
                    Some(block)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if unhandled_event.len() > 1 {
            let extra = unhandled_event
                .get(1)
                .expect("length check done above");
            return syn::Error::new(
                extra.span(),
                "multiple unhandled_event blocks",
            )
            .to_compile_error()
            .into();
        }

        unhandled_event.pop().cloned()
    };

    let state_transitions =
        state_transitions.into_iter().filter_map(|transition| {
            if let StateTransitions::State(x) = transition {
                Some(x)
            } else {
                None
            }
        });

    let state_events = state_transitions.into_iter().map(|StateStateTransitions { state_path, transitions }| {
        let Some(state_ident) = state_path.segments.last().map(|s| s.ident.clone()) else {
            return Err(syn::Error::new(state_path.span(), "state path is empty"));
        };

        transitions.into_iter().map(|Transition { event_path, block }| {
            let Some(event_ident) = event_path.segments.last().map(|s| s.ident.clone()) else {
                return Err(syn::Error::new(event_path.span(), "event path is empty"));
            };

            let (block, is_default) = match block {
                TransitionBlock::Block(block) => (block, false),
                TransitionBlock::Default(target) => {
                    (syn::parse_quote! {{
                        #target::default()
                    }}, true)
                }
            };

            Ok(StateEvent {
                state_ident: state_ident.clone(),
                state_path: state_path.clone(),
                event_path,
                event_ident,
                block,
                is_default,
            })
        })
        .collect::<syn::Result<Vec<_>>>()
    })
    .collect::<syn::Result<Vec<_>>>();

    let state_events = match state_events {
        Ok(x) => {
            x.into_iter().flatten().collect::<Vec<_>>()
        }
        Err(e) => return e.to_compile_error().into(),
    };

    let handle_event_match_arms = state_events.iter()
        .map(|StateEvent { state_path, state_ident, event_path, event_ident, block, is_default }| {
            if *is_default {
                quote! {
                    (#state_enum_ident::#state_ident(state), #event_enum_ident::#event_ident(event)) => {
                        #block.into()
                    }
                }
            } else {
                let function_ident = Ident::new(
                    &format!("handle__{}__{}", 
                        state_ident.to_string().to_snake_case(),
                        event_ident.to_string().to_snake_case()
                    ),
                    state_ident.span(),
                );

                quote! {
                    (#state_enum_ident::#state_ident(state), #event_enum_ident::#event_ident(event)) => {
                        #[allow(non_snake_case)]
                        #asyncness fn #function_ident(
                            mut state: #state_path,
                            event: &mut #event_path,
                            context: &mut #context_path,
                        ) -> impl Into<#state_enum_ident> #block
                    
                        #function_ident(state, event, &mut self.context)#async_postfix.into()
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let event_enum_trait_variants = state_events
        .iter()
        .map(|StateEvent { event_path, .. }| {
            event_path.clone()
        })
        .chain(other_events.iter().cloned())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let other_event_path_ident = other_events.iter().map(|event_path| {
        let Some(event_ident) = event_path.segments.last().map(|s| s.ident.clone()) else {
            return Err(syn::Error::new(event_path.span(), "event path is empty"));
        };

        Ok((event_path.clone(), event_ident))
    })
    .collect::<syn::Result<Vec<_>>>();

    let other_event_path_ident = match other_event_path_ident {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into(),
    };

    let event_path_ident = state_events
        .iter()
        .map(
            |StateEvent {
                 event_path, event_ident, ..
             }| {
                (event_path.clone(), event_ident.clone())
            },
        )
        .chain(other_event_path_ident)
        .collect::<HashMap<_, _>>();

    let event_from_impls = event_path_ident
        .iter()
        .map(|(event_path, event_ident)| {
            quote! {
                impl From<#event_path> for #event_enum_ident {
                    fn from(event: #event_path) -> Self {
                        Self::#event_ident(event)
                    }
                }
            }
        });

    #[allow(clippy::wildcard_enum_match_arm)]
    let event_trait_function_sigs = event_trait.items.iter().filter_map(|item| match item {
        TraitItem::Fn(f) => Some(f.sig.clone()),
        _ => None,
    })
    .collect::<Vec<_>>();

    let pre_transition_postfix = event_trait_function_sigs
        .iter()
        .find(|sig| sig.ident == "pre_transition")
        .and_then(|sig| sig.asyncness.and_then(|_| async_postfix.clone()));

    let post_transition_postfix = event_trait_function_sigs
        .iter()
        .find(|sig| sig.ident == "post_transition")
        .and_then(|sig| sig.asyncness.and_then(|_| async_postfix.clone()));

    let event_enum_trait_functions = event_trait_function_sigs.iter().map(|sig| {
        let ident = &sig.ident;
        let args = sig.inputs.iter().skip(1).map(|input| {
            let FnArg::Typed(input) = input else {
                return Err(syn::Error::new(input.span(), "expected typed input"));
            };

            let pat = &input.pat;

            Ok(quote!(#pat))
        })
        .collect::<syn::Result<Vec<_>>>()?;

        let async_postfix = sig.asyncness.as_ref().map(|_| quote!(.await));
        let args = once(quote!(event)).chain(args).collect::<Vec<_>>();
        let arms = event_path_ident.iter().map(|(event_path, event_ident)| {
            quote! {
                Self::#event_ident(event) => #event_path::#ident(#(#args),*)#async_postfix,
            }
        });

        Ok(quote! {
            #sig {
                match self {
                    #(#arms)*
                }
            }
        })
    })
    .collect::<syn::Result<Vec<_>>>();

    let event_enum_trait_functions =
        match event_enum_trait_functions {
            Ok(x) => x,
            Err(e) => return e.to_compile_error().into(),
        };

    let event_enum_trait_impl = quote! {
        #maybe_async_trait_attr
        impl #event_trait_path for #event_enum_ident {
            #(#event_enum_trait_functions)*
        }
    };

    let state_enum_trait_variants = state_events
        .iter()
        .map(|StateEvent { state_path, .. }| {
            state_path.clone()
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let state_path_ident = state_events
        .iter()
        .map(
            |StateEvent {
                 state_path, state_ident, ..
             }| {
                (state_path.clone(), state_ident.clone())
            },
        )
        .collect::<HashMap<_, _>>();

    let state_from_impls = state_path_ident
        .iter()
        .map(|(state_path, state_ident)| {
            quote! {
                impl From<#state_path> for #state_enum_ident {
                    fn from(state: #state_path) -> Self {
                        Self::#state_ident(state)
                    }
                }
            }
        });

    #[allow(clippy::wildcard_enum_match_arm)]
    let state_trait_function_sigs = state_trait.items.iter().filter_map(|item| match item {
        TraitItem::Fn(f) => Some(f.sig.clone()),
        _ => None,
    })
    .collect::<Vec<_>>();

    let on_enter_postfix = state_trait_function_sigs
        .iter()
        .find(|sig| sig.ident == "on_enter")
        .and_then(|sig| sig.asyncness.and_then(|_| async_postfix.clone()));

    let on_exit_postfix = state_trait_function_sigs
        .iter()
        .find(|sig| sig.ident == "on_exit")
        .and_then(|sig| sig.asyncness.and_then(|_| async_postfix.clone()));

    let should_exit_postfix = state_trait_function_sigs
        .iter()
        .find(|sig| sig.ident == "should_exit")
        .and_then(|sig| sig.asyncness.and_then(|_| async_postfix.clone()));

    let state_enum_trait_functions = state_trait_function_sigs.iter().map(|sig| {
        let ident = &sig.ident;
        let args = sig.inputs.iter().skip(1).map(|input| {
            let FnArg::Typed(input) = input else {
                return Err(syn::Error::new(input.span(), "expected typed input"));
            };

            let pat = &input.pat;

            Ok(quote!(#pat))
        })
        .collect::<syn::Result<Vec<_>>>()?;

        let async_postfix = sig.asyncness.as_ref().map(|_| quote!(.await));
        let args = once(quote!(state)).chain(args).collect::<Vec<_>>();
        let arms = state_path_ident.iter().map(|(state_path, state_ident)| {
            quote! {
                Self::#state_ident(state) => #state_path::#ident(#(#args),*)#async_postfix,
            }
        });

        Ok(quote! {
            #sig {
                match self {
                    #(#arms)*
                }
            }
        })
    })
    .collect::<syn::Result<Vec<_>>>();

    let state_enum_trait_functions =
        match state_enum_trait_functions {
            Ok(x) => x,
            Err(e) => return e.to_compile_error().into(),
        };

    let state_enum_trait_impl = quote! {
        #maybe_async_trait_attr
        impl #state_trait_path for #state_enum_ident {
            #(#[allow(clippy::used_underscore_binding)] #state_enum_trait_functions)*
        }
    };

    let unhandled_event = unhandled_event.map(|block| {
        let function_ident = Ident::new(
            "handle__unhandled_event",
            block.span(),
        );

        quote! {
            (state, event) => {
                #[allow(non_snake_case)]
                #asyncness fn #function_ident(
                    mut state: #state_enum_ident,
                    event: &mut #event_enum_ident,
                    context: &mut #context_path,
                ) -> impl Into<#state_enum_ident> #block
                    
                #function_ident(state, event, &mut self.context)#async_postfix.into()
            }
        }
    });

    let event_enum = match event_enum(EventEnumInput {
        attributes: event_enum_attrs,
        visibility: visibility.clone(),
        ident: event_enum_ident.clone(),
        event_paths: event_enum_trait_variants,
    }) {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into(),
    };

    let state_enum = match state_enum(StateEnumInput {
        attributes: state_enum_attrs,
        visibility: visibility.clone(),
        ident: state_enum_ident.clone(),
        state_paths: state_enum_trait_variants,
    }) {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = quote! {
        #event_trait
        #event_enum
        #(#event_from_impls)*
        #event_enum_trait_impl

        #state_trait
        #state_enum
        #(#state_from_impls)*
        #state_enum_trait_impl

        #(#attributes)*
        #visibility struct #name {
            context: #context_path,
            state: ::core::option::Option<#state_enum_ident>,
        }

        impl #name {
            pub fn new<State: Into<#state_enum_ident> + #state_trait_path>(state: State, context: #context_path) -> Self {
                Self { context, state: ::core::option::Option::Some(state.into()) }
            }

            pub fn context(&self) -> &#context_path {
                &self.context
            }

            pub fn state(&self) -> &#state_enum_ident {
                self.state.as_ref().expect("state is missing")
            }

            pub fn into_context(self) -> #context_path {
                self.context
            }

            pub fn into_state(self) -> #state_enum_ident {
                self.state.expect("state is missing")
            }

            pub fn into_parts(self) -> (#state_enum_ident, #context_path) {
                let Self { context, state } = self;
                let state = state.expect("state is missing");
                (state, context)
            }

            pub #asyncness fn handle_event<Event: Into<#event_enum_ident> + #event_trait_path>(&mut self, event: Event) -> &mut Self {
                let mut event = event.into();
                let mut state = self.state.take().expect("state is missing");

                if !#state_enum_ident::should_exit(&state, &self.context, &event)#should_exit_postfix {
                    self.state = ::core::option::Option::Some(state);
                    return self;
                }

                #state_trait_path::on_exit(&mut state, &mut self.context)#on_exit_postfix;
                #event_trait_path::pre_transition(&mut event, &mut self.context)#pre_transition_postfix;

                let mut state: #state_enum_ident = match (state, &mut event) {
                    #(#handle_event_match_arms)*
                    #unhandled_event
                };

                #event_trait_path::post_transition(&mut event, &mut self.context)#post_transition_postfix;
                #state_trait_path::on_enter(&mut state, &mut self.context)#on_enter_postfix;

                self.state = ::core::option::Option::Some(state);
                self
            }
        }
    };

    expanded.into()
}
