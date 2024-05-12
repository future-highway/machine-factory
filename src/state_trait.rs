use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;
use syn::FnArg;
use syn::ItemTrait;
use syn::Path;
use syn::TraitItem;
use syn::Type;

pub fn state_trait(attr: TokenStream, trait_: TokenStream) -> TokenStream {
    let context_path = parse_macro_input!(attr as Path);
    let mut trait_ = parse_macro_input!(trait_ as syn::ItemTrait);
    ensure_on_enter_fn(&context_path, &mut trait_).expect("`on_enter` function error");
    ensure_on_exit_fn(&context_path, &mut trait_).expect("`on_exit` function error");
    trait_.to_token_stream().into()
}

/// We want the `on_enter` function to be mandatory.
/// If it is not present, we add.
/// If it is present but has a different signature, we return an error.
fn ensure_on_enter_fn(context_path: &Path, trait_: &mut ItemTrait) -> syn::Result<()> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let func = trait_.items.iter().find_map(|item| match item {
        TraitItem::Fn(f) if f.sig.ident == "on_enter" => Some(f),
        _ => None,
    });

    if let Some(func) = func {
        const FIRST_ARG_ERROR: &str = "must accept `&mut self` as the first argument";
        const SECOND_ARG_ERROR: &str = "must accept `&mut {Context}` as the second argument";

        let mut inputs = func.sig.inputs.iter();

        let first_input = inputs.next();

        let Some(first_input) = first_input else {
            return Err(syn::Error::new_spanned(func, FIRST_ARG_ERROR));
        };

        let FnArg::Receiver(first_input) = first_input else {
            return Err(syn::Error::new_spanned(func, FIRST_ARG_ERROR));
        };

        if first_input.mutability.is_none() || first_input.reference.is_none() {
            return Err(syn::Error::new_spanned(first_input, FIRST_ARG_ERROR));
        }

        let second_input = inputs.next();

        let Some(second_input) = second_input else {
            return Err(syn::Error::new_spanned(second_input, SECOND_ARG_ERROR));
        };

        let FnArg::Typed(second_input) = second_input else {
            return Err(syn::Error::new_spanned(second_input, SECOND_ARG_ERROR));
        };

        #[allow(clippy::wildcard_enum_match_arm)]
        let second_input_ty = match &*second_input.ty {
            Type::Reference(r) if r.mutability.is_some() => r.elem.as_ref(),
            _ => return Err(syn::Error::new_spanned(second_input, SECOND_ARG_ERROR)),
        };

        if !matches!(second_input_ty, Type::Path(p) if p.path.eq(context_path)) {
            return Err(syn::Error::new_spanned(second_input, SECOND_ARG_ERROR));
        }
    } else {
        let on_enter = syn::parse_quote! {
            fn on_enter(&mut self, context: &mut #context_path) {}
        };

        trait_.items.push(TraitItem::Fn(on_enter));
    }

    Ok(())
}

fn ensure_on_exit_fn(context_path: &Path, trait_: &mut ItemTrait) -> syn::Result<()> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let func = trait_.items.iter().find_map(|item| match item {
        TraitItem::Fn(f) if f.sig.ident == "on_exit" => Some(f),
        _ => None,
    });

    if let Some(func) = func {
        const FIRST_ARG_ERROR: &str = "must accept `self` as the first argument";
        const SECOND_ARG_ERROR: &str = "must accept `&mut {Context}` as the second argument";

        let mut inputs = func.sig.inputs.iter();

        let first_input = inputs.next();

        let Some(first_input) = first_input else {
            return Err(syn::Error::new_spanned(func, FIRST_ARG_ERROR));
        };

        let FnArg::Receiver(first_input) = first_input else {
            return Err(syn::Error::new_spanned(func, FIRST_ARG_ERROR));
        };

        if first_input.mutability.is_none() || first_input.reference.is_none() {
            return Err(syn::Error::new_spanned(first_input, FIRST_ARG_ERROR));
        }

        let second_input = inputs.next();

        let Some(second_input) = second_input else {
            return Err(syn::Error::new_spanned(second_input, SECOND_ARG_ERROR));
        };

        let FnArg::Typed(second_input) = second_input else {
            return Err(syn::Error::new_spanned(second_input, SECOND_ARG_ERROR));
        };

        #[allow(clippy::wildcard_enum_match_arm)]
        let second_input_ty = match &*second_input.ty {
            Type::Reference(r) if r.mutability.is_some() => r.elem.as_ref(),
            _ => return Err(syn::Error::new_spanned(second_input, SECOND_ARG_ERROR)),
        };

        if !matches!(second_input_ty, Type::Path(p) if p.path.eq(context_path)) {
            return Err(syn::Error::new_spanned(second_input, SECOND_ARG_ERROR));
        }
    } else {
        let on_exit = syn::parse_quote! {
            fn on_exit(&mut self, context: &mut #context_path) {}
        };

        trait_.items.push(TraitItem::Fn(on_exit));
    }

    Ok(())
}
