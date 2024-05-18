use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    Attribute, Ident, ImplItemFn, ItemTrait, Path, Token,
    Visibility,
};

struct Machine {
    attributes: Vec<Attribute>,
    visibility: Option<Visibility>,
    name: Ident,
    context: Path,
    state_trait: Option<ItemTrait>,
    state_transitions: Vec<StateTransitions>,
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

        _ = input
            .peek(Token![struct])
            .then(|| input.parse::<Token![struct]>())
            .transpose()?;

        let name = input.parse()?;

        let content;
        _ = braced!(content in input);

        let mut context = None;
        let mut state_trait = None;
        let mut state_transitions = None;

        while content.peek(Ident) {
            let label: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;

            match label.to_string().as_str() {
                "context" => {
                    context = Some(content.parse()?);
                }
                "state_trait" => {
                    state_trait = Some(content.parse()?);
                }
                "states" => {
                    let transitions_content;
                    _ = bracketed!(transitions_content in content);
                    let parsed_transitions =
                        Punctuated::<
                            StateTransitions,
                            Token![,],
                        >::parse_terminated(
                            &transitions_content,
                        )?;
                    state_transitions = Some(
                        parsed_transitions
                            .into_iter()
                            .collect::<Vec<_>>(),
                    );
                }
                _ => {
                    return Err(syn::Error::new(
                        label.span(),
                        "unexpected label",
                    ));
                }
            }

            if content.peek(Comma) {
                let _: Comma = content.parse()?;
            }
        }

        if content.peek(Comma) {
            let _: Comma = content.parse()?;
        }

        let context = context.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "missing `context`",
            )
        })?;

        let state_transitions = state_transitions
            .ok_or_else(|| {
                syn::Error::new(
                    input.span(),
                    "missing `states`",
                )
            })?;

        Ok(Self {
            attributes,
            visibility,
            name,
            context,
            state_trait,
            state_transitions,
        })
    }
}

struct StateTransitions {
    state: Path,
    transitions: Vec<ImplItemFn>,
}

impl Parse for StateTransitions {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let state = input.parse()?;

        let content;
        _ = braced!(content in input);
        let mut transitions = vec![];
        while !content.is_empty() {
            let transition =
                content.parse::<ImplItemFn>()?;
            transitions.push(transition);
        }

        Ok(Self { state, transitions })
    }
}

#[allow(clippy::too_many_lines)]
pub fn deterministic_state_machine(
    input: TokenStream,
) -> TokenStream {
    let Machine {
        attributes,
        visibility,
        name,
        context,
        state_trait,
        state_transitions,
    } = parse_macro_input!(input as Machine);

    let next_impls = state_transitions
        .into_iter()
        .map(|StateTransitions { state, transitions }| {
            quote! {
                impl #name<#state> {
                    #(#transitions)*
                }
            }
        })
        .collect::<Vec<_>>();

    let state_trait_where_clause =
        state_trait.as_ref().map(|state_trait| {
            let state_trait_ident = &state_trait.ident;
            quote! { where State: #state_trait_ident }
        });

    let expanded = quote! {
        #(#attributes)*
        #visibility struct #name<State>
        #state_trait_where_clause
        {
            context: #context,
            state: State,
        }

        impl<State> #name<State>
        #state_trait_where_clause
        {
            pub fn new(intial_state: State, context: #context) -> Self {
                Self {
                    context,
                    state: intial_state,
                }
            }

            pub fn context(&self) -> &#context {
                &self.context
            }

            pub fn state(&self) -> &State {
                &self.state
            }

            pub fn into_context(self) -> #context {
                self.context
            }

            pub fn into_state(self) -> State {
                self.state
            }

            pub fn into_parts(self) -> (State, #context) {
                (self.state, self.context)
            }
        }

        #state_trait

        #(#next_impls)*
    };

    expanded.into()
}
