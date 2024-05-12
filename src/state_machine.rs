use crate::{
    event_enum::{event_enum, EventEnumInput},
    state_enum::{state_enum, StateEnumInput},
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
    token::{Brace, Comma},
    Attribute, Block, FnArg, Ident, Path, Token, TraitItem, Visibility,
};

struct Machine {
    visibility: Option<Visibility>,
    name: Ident,
    context_path: Path,
    state_enum_attrs: Vec<Attribute>,
    state_enum_ident: Ident,
    state_trait: syn::ItemTrait,
    event_enum_attrs: Vec<Attribute>,
    event_enum_ident: Ident,
    event_trait: syn::ItemTrait,
    transitions: Vec<StateTransitions>,
    unhandled_event: Option<Block>,
}

impl Parse for Machine {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let visibility = input.peek(Token![pub]).then(|| input.parse()).transpose()?;
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
        let mut transitions = None;
        let mut unhandled_event = None;

        while content.peek(Ident) {
            let label: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;

            match label.to_string().as_str() {
                "context" => {
                    drop(context_path.replace(content.parse()?));
                }
                "state_enum" => {
                    if content.peek(Token![#]) {
                        drop(state_enum_attrs.replace(Attribute::parse_outer(&content)?));
                    }

                    drop(state_enum_ident.replace(content.parse()?));
                }
                "state_trait" => {
                    drop(state_trait.replace(content.parse()?));
                }
                "event_enum" => {
                    if content.peek(Token![#]) {
                        drop(event_enum_attrs.replace(Attribute::parse_outer(&content)?));
                    }
                    drop(event_enum_ident.replace(content.parse()?));
                }
                "event_trait" => {
                    drop(event_trait_path.replace(content.parse()?));
                }
                "transitions" => {
                    let content2;
                    let _ = bracketed!(content2 in content);
                    let parsed_transitions = Punctuated::<StateTransitions, Comma>::parse_terminated(&content2)?;
                    let parsed_transitions = parsed_transitions.into_iter().collect();
                    drop(transitions.replace(parsed_transitions));
                }
                "unhandled_event" => {
                    drop(unhandled_event.replace(content.parse()?));
                }
                _ => return Err(syn::Error::new(label.span(), "unrecognized label")),
            }

            if content.peek(Comma) {
                let _: Comma = content.parse()?;
            }
        }

        let context_path =
            context_path.ok_or_else(|| syn::Error::new(name.span(), "machine is missing context"))?;

        let state_enum_ident = state_enum_ident
            .ok_or_else(|| syn::Error::new(name.span(), "machine is missing state_enum"))?;

        let state_trait = state_trait
            .ok_or_else(|| syn::Error::new(name.span(), "machine is missing state_trait"))?;

        let event_enum_ident = event_enum_ident
            .ok_or_else(|| syn::Error::new(name.span(), "machine is missing event_enum"))?;
        
        let event_trait = event_trait_path
            .ok_or_else(|| syn::Error::new(name.span(), "machine is missing event_trait"))?;

        let transitions =
            transitions.ok_or_else(|| syn::Error::new(name.span(), "machine is missing transitions"))?;

        Ok(Self {
            visibility,
            name,
            context_path,
            state_enum_attrs: state_enum_attrs.unwrap_or_default(),
            state_enum_ident,
            state_trait,
            event_enum_attrs: event_enum_attrs.unwrap_or_default(),
            event_enum_ident,
            event_trait,
            transitions,
            unhandled_event,
        })
    }
}

struct StateTransitions {
    state_path: Path,
    transitions: Vec<Transition>,
}

impl Parse for StateTransitions {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let state_path = input.parse()?;

        let content;
        _ = braced!(content in input);
        let transitions = Punctuated::<Transition, Comma>::parse_terminated(&content)?;
        let transitions = transitions.into_iter().collect();

        Ok(Self {
            state_path,
            transitions,
        })
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
            return Err(syn::Error::new(input.span(), "expected -> or {"));
        };


        Ok(Self {
            event_path: event,
            block,
        })
    }
}

struct StateEvent {
    state_path: Path,
    state_ident: Ident,
    event_path: Path,
    event_ident: Ident,
    block: Block,
}

#[allow(clippy::too_many_lines)]
pub(super) fn state_machine(input: TokenStream) -> TokenStream {
    let Machine {
        visibility,
        name,
        context_path,
        state_enum_attrs,
        state_enum_ident,
        state_trait,
        event_enum_attrs,
        event_enum_ident,
        event_trait,
        transitions,
        unhandled_event,
    } = parse_macro_input!(input as Machine);

    let state_trait_path= &state_trait.ident;
    let event_trait_path = &event_trait.ident;

    let state_events = transitions.into_iter().map(|StateTransitions { state_path, transitions }| {
        let Some(state_ident) = state_path.segments.last().map(|s| s.ident.clone()) else {
            return Err(syn::Error::new(state_path.span(), "state path is empty"));
        };

        transitions.into_iter().map(|Transition { event_path, block }| {
            let Some(event_ident) = event_path.segments.last().map(|s| s.ident.clone()) else {
                return Err(syn::Error::new(event_path.span(), "event path is empty"));
            };

            let block = match block {
                TransitionBlock::Block(block) => block,
                TransitionBlock::Default(target) => {
                    syn::parse_quote! {{
                        #target::default()
                    }}
                }
            };

            Ok(StateEvent {
                state_ident: state_ident.clone(),
                state_path: state_path.clone(),
                event_path,
                event_ident,
                block,
            })
        })
        .collect::<syn::Result<Vec<_>>>()
    })
    .collect::<syn::Result<Vec<_>>>();

    let state_events = match state_events {
        Ok(x) => x.into_iter().flatten().collect::<Vec<_>>(),
        Err(e) => return e.to_compile_error().into(),
    };

    let handle_event_match_arms = state_events.iter()
        .map(|StateEvent { state_path, state_ident, event_path, event_ident, block }| {
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
                    fn #function_ident(
                        mut state: #state_path,
                        mut event: #event_path,
                        context: &mut #context_path,
                    ) -> #state_enum_ident {
                        #state_trait_path::on_exit(&mut state, context);
                        #event_trait_path::pre_transition(&mut event, context);
                        let mut state: #state_enum_ident = #block.into();
                        #event_trait_path::post_transition(&mut event, context);
                        #state_trait_path::on_enter(&mut state, context);
                        state
                    }
                    
                    let context = &mut self.context;
                    let state = #function_ident(state, event, &mut self.context);
                    state
                }
            }
        })
        .collect::<Vec<_>>();

    let event_enum_trait_variants = state_events.iter()
        .map(|StateEvent { event_path, .. }| event_path.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let event_path_ident =state_events
        .iter()
        .map(|StateEvent { event_path, event_ident, .. }| (event_path.clone(), event_ident.clone()))
        .collect::<HashMap<_, _>>();

    let event_from_impls = event_path_ident
        .iter()
        .map(|(event_path, event_ident)| {
            quote! {
                impl From<#event_path> for #event_enum_ident {
                    fn from(event: #event_path) -> Self {
                        #event_enum_ident::#event_ident(event)
                    }
                }
            }
        });

    #[allow(clippy::wildcard_enum_match_arm)]
    let event_trait_function_sigs = event_trait.items.iter().filter_map(|item| match item {
        TraitItem::Fn(f) => Some((f.sig.ident.to_string(), f.sig.clone())),
        _ => None,
    })
    .chain([
        ("pre_transition".to_owned(), parse_quote!(fn pre_transition(&mut self, context: &mut #context_path))),
        ("post_transition".to_owned(), parse_quote!(fn post_transition(&mut self, context: &mut #context_path))),
    ])
    .collect::<HashMap<_, _>>();

    let event_enum_trait_functions = event_trait_function_sigs.values().map(|sig| {
        let ident = &sig.ident;
        let args = sig.inputs.iter().skip(1).map(|input| {
            let FnArg::Typed(input) = input else {
                return Err(syn::Error::new(input.span(), "expected typed input"));
            };

            let pat = &input.pat;

            Ok(quote!(#pat))
        })
        .collect::<syn::Result<Vec<_>>>()?;

        let args = once(quote!(event)).chain(args).collect::<Vec<_>>();
        let arms = event_path_ident.iter().map(|(event_path, event_ident)| {
            quote! {
                Self::#event_ident(event) => #event_path::#ident(#(#args),*),
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

    let event_enum_trait_functions = match event_enum_trait_functions {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into(),
    };

     let event_enum_trait_impl = quote! {
        impl #event_trait_path for #event_enum_ident {
            #(#event_enum_trait_functions)*
        }
    };

    let state_enum_trait_variants = state_events.iter()
        .map(|StateEvent { state_path, .. }| state_path.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let state_path_ident = state_events
        .iter()
        .map(|StateEvent { state_path, state_ident, .. }| (state_path.clone(), state_ident.clone()))
        .collect::<HashMap<_, _>>();

    let state_from_impls = state_path_ident
        .iter()
        .map(|(state_path, state_ident)| {
            quote! {
                impl From<#state_path> for #state_enum_ident {
                    fn from(state: #state_path) -> Self {
                        #state_enum_ident::#state_ident(state)
                    }
                }
            }
        });

    #[allow(clippy::wildcard_enum_match_arm)]
    let state_trait_function_sigs = state_trait.items.iter().filter_map(|item| match item {
        TraitItem::Fn(f) => Some((f.sig.ident.to_string(), f.sig.clone())),
        _ => None,
    })
    .chain([
        ("on_enter".to_owned(), parse_quote!(fn on_enter(&mut self, context: &mut #context_path))),
        ("on_exit".to_owned(), parse_quote!(fn on_exit(&mut self, context: &mut #context_path))),
    ])
    .collect::<HashMap<_, _>>();

    let state_enum_trait_functions = state_trait_function_sigs.values().map(|sig| {
        let ident = &sig.ident;
        let args = sig.inputs.iter().skip(1).map(|input| {
            let FnArg::Typed(input) = input else {
                return Err(syn::Error::new(input.span(), "expected typed input"));
            };

            let pat = &input.pat;

            Ok(quote!(#pat))
        })
        .collect::<syn::Result<Vec<_>>>()?;

        let args = once(quote!(state)).chain(args).collect::<Vec<_>>();
        let arms = state_path_ident.iter().map(|(state_path, state_ident)| {
            quote! {
                Self::#state_ident(state) => #state_path::#ident(#(#args),*),
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

    let state_enum_trait_functions = match state_enum_trait_functions {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into(),
    };

    let state_enum_trait_impl = quote! {
        impl #state_trait_path for #state_enum_ident {
            #(#state_enum_trait_functions)*
        }
    };

    let unhandled_event = unhandled_event.map(|block| {
        quote! {
            (state, event) => #block
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
        visibility,
        ident: state_enum_ident.clone(),
        state_paths: state_enum_trait_variants,
    }) {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = quote! {
        #[::machine_factory::event_trait(#context_path)]
        #event_trait
        #event_enum
        #(#event_from_impls)*
        #event_enum_trait_impl

        #[::machine_factory::state_trait(#context_path)]
        #state_trait
        #state_enum
        #(#state_from_impls)*
        #state_enum_trait_impl

        pub struct #name {
            context: #context_path,
            state: ::std::option::Option<#state_enum_ident>,
        }

        impl #name {
            pub fn init<State: Into<#state_enum_ident> + #state_trait_path>(state: State, context: #context_path) -> Self {
                Self { context, state: ::std::option::Option::Some(state.into()) }
            }

            pub fn context(&self) -> &#context_path {
                &self.context
            }

            pub fn state(&self) -> &dyn #state_trait_path {
                self.state.as_ref().expect("state is missing")
            }

            pub fn handle_event<Event: Into<#event_enum_ident> + #event_trait_path>(&mut self, event: Event) -> &mut Self {
                let state = self.state.take().expect("state is missing");
                
                let state: #state_enum_ident = match (state, event.into()) {
                    #(#handle_event_match_arms)*
                    #unhandled_event
                };

                _ = self.state.replace(state);
                self
            }
        }
    };

    expanded.into()
}
