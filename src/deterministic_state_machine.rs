use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    FnArg, Ident, ItemTrait, Path, Token, Visibility,
};

struct Machine {
    visibility: Option<Visibility>,
    name: Ident,
    context: Path,
    state_trait: ItemTrait,
    acceptor_trait: Option<Ident>,
    acceptor_struct: Path,
    error: Option<Path>,
    transitions: Vec<Transition>,
}

impl Parse for Machine {
    #[allow(clippy::too_many_lines)]
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let visibility = input
            .peek(Token![pub])
            .then(|| input.parse())
            .transpose()?;

        let name = input.parse()?;

        let content;
        _ = braced!(content in input);

        let mut context = None;
        let mut state_trait = None;
        let mut acceptor_trait = None;
        let mut acceptor_struct = None;
        let mut error = None;
        let mut transitions = None;

        'outer: while content.peek(Ident) {
            let label: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;

            match label.to_string().as_str() {
                "context" => {
                    context = Some(content.parse()?);
                }
                "state_trait" => {
                    state_trait = Some(content.parse()?);
                }
                "acceptor_trait" => {
                    acceptor_trait = Some(content.parse()?);
                }
                "acceptor_struct" => {
                    acceptor_struct =
                        Some(content.parse()?);
                }
                "error" => {
                    error = Some(content.parse()?);
                }
                "transitions" => {
                    let transitions_content;
                    _ = bracketed!(transitions_content in content);

                    let mut state: Path =
                        transitions_content.parse()?;

                    let mut parsed = vec![];
                    '_inner: loop {
                        let mut data =
                            Transition::new(state);

                        let _: Token![.] =
                            transitions_content.parse()?;

                        data.fn_ident = Some(
                            transitions_content.parse()?,
                        );

                        let fn_args_content;
                        _ = parenthesized!(fn_args_content in transitions_content);
                        data.fn_input = Some(
                            fn_args_content
                                .parse_terminated(
                                    FnArg::parse,
                                    Comma,
                                )?,
                        );

                        if transitions_content
                            .peek(Token![.])
                        {
                            let _: Token![.] =
                                transitions_content
                                    .parse()?;

                            let _: Token![await] =
                                transitions_content
                                    .parse()?;

                            data.is_async = true;
                        }

                        if transitions_content
                            .peek(Token![?])
                        {
                            let _: Token![?] =
                                transitions_content
                                    .parse()?;

                            data.is_result = true;

                            // Hack: How do I check if the next token is a path?
                            if !transitions_content.peek(Token![->]) {
                                data.error =
                                    Some(transitions_content.parse()?);
                            }
                        }

                        let _: Token![->] =
                            transitions_content.parse()?;

                        state =
                            transitions_content.parse()?;

                        data.next_state =
                            Some(state.clone());

                        parsed.push(data);

                        if transitions_content.is_empty() {
                            transitions = Some(parsed);
                            continue 'outer;
                        }
                    }
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

        let state_trait = state_trait.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "missing `state_trait`",
            )
        })?;

        let acceptor_struct =
            acceptor_struct.ok_or_else(|| {
                syn::Error::new(
                    input.span(),
                    "missing `acceptor_struct`",
                )
            })?;

        let transitions = transitions.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "missing `transitions`",
            )
        })?;

        if error.is_none()
            && transitions
                .iter()
                .any(|t| t.is_result && t.error.is_none())
        {
            return Err(syn::Error::new(
                input.span(),
                "missing `error`",
            ));
        }

        Ok(Self {
            visibility,
            name,
            context,
            state_trait,
            acceptor_trait,
            acceptor_struct,
            error,
            transitions,
        })
    }
}

struct Transition {
    state: Path,
    fn_ident: Option<Ident>,
    fn_input: Option<Punctuated<FnArg, Comma>>,
    is_async: bool,
    is_result: bool,
    error: Option<Path>,
    next_state: Option<Path>,
}

impl Transition {
    const fn new(state: Path) -> Self {
        Self {
            state,
            fn_ident: None,
            fn_input: None,
            is_async: false,
            is_result: false,
            error: None,
            next_state: None,
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn deterministic_state_machine(
    input: TokenStream,
) -> TokenStream {
    let Machine {
        visibility,
        name,
        context,
        state_trait,
        acceptor_trait: acceptor_trait_ident,
        acceptor_struct,
        error,
        transitions,
    } = parse_macro_input!(input as Machine);

    let state_trait_ident = &state_trait.ident;
    let acceptor_struct_ident = &acceptor_struct;

    let acceptor_trait = acceptor_trait_ident.as_ref().map_or_else(|| None, |acceptor_trait_ident| {
        let fns = transitions.iter().map(
            |Transition {
                state,
                fn_ident,
                fn_input,
                is_async,
                is_result,
                error: transition_error,
                next_state,
            }| {
                let async_token =
                    is_async.then(|| quote! { async });

                let result = if *is_result {
                    let error = transition_error.as_ref().unwrap_or_else(|| error.as_ref().expect("prevented in parsing"));
                    quote! { Result<#name<#next_state>, #error> }
                } else {
                    quote! { #name<#next_state> }
                };

                quote! {
                    #async_token fn #fn_ident(
                        state: #name<#state>,
                        #fn_input
                    ) -> #result;
                }
            },
        );

        Some(quote! {
            #visibility trait #acceptor_trait_ident {
                #(#fns)*
            }
        })
    });

    let next_impls = transitions.into_iter().map(
        |Transition {
             state,
             fn_ident,
             fn_input,
             is_async,
             is_result,
             error: transition_error,
             next_state,
         }| {
            let async_token =
                is_async.then(|| quote! { async });

            let async_postfix =
                is_async.then(|| quote! { .await });

            let result = if is_result {
                let error = transition_error.as_ref().unwrap_or_else(|| error.as_ref().expect("prevented in parsing"));
                quote! { Result<#name<#next_state>, #error> }
            } else {
                quote! { #name<#next_state> }
            };

            let params = fn_input.as_ref().map(|fn_input| {
                fn_input.iter().map(|fn_arg| {
                    match fn_arg {
                        FnArg::Receiver(_) => {
                            Err(syn::Error::new(fn_arg.span(), "unexpected receiver (the current state is automatically inserted as the first argument)"))
                        }
                        FnArg::Typed(pat_type) => {
                            let pat = &pat_type.pat;
                            Ok(quote! { #pat })
                        }
                    }
                }).collect::<syn::Result<Vec<_>>>()
            }).transpose()?.unwrap_or_default();

            let acceptor_type = acceptor_trait_ident.as_ref().map_or_else(
                || quote! {#acceptor_struct_ident}, 
                |acceptor_trait_ident| quote! {
                    <#acceptor_struct_ident as #acceptor_trait_ident>
                },
            );

            Ok(quote! {
                impl #name<#state> {
                    pub #async_token fn #fn_ident(
                        self,
                        #fn_input
                    ) -> #result {
                        #acceptor_type::#fn_ident(self, #(#params),*)#async_postfix
                    }
                }
            })
        },
    )
    .collect::<syn::Result<Vec<_>>>();

    let next_impls = match next_impls {
        Ok(next_impls) => next_impls,
        Err(err) => return err.to_compile_error().into(),
    };

    let expanded = quote! {
        #visibility struct #name<State: #state_trait_ident> {
            context: #context,
            state: State,
        }

        impl<State: #state_trait_ident> #name<State> {
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

        #acceptor_trait

        #state_trait

        #(#next_impls)*
    };

    expanded.into()
}
