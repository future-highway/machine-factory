use syn::{
    token::Async, FnArg, Ident, ItemTrait, Path,
    ReturnType, TraitItem, Type,
};

// NOTE: Haven't refactored out the fn check becuase the
// decision on the fn signature is still not firm.

pub fn ensure_state_trait(
    asyncness: Option<Async>,
    trait_: &mut ItemTrait,
    context_path: &Path,
    event_enum_iden: &Ident,
) -> syn::Result<()> {
    ensure_on_enter_fn(asyncness, trait_, context_path)?;
    ensure_on_exit_fn(asyncness, trait_, context_path)?;
    ensure_should_exit_fn(
        asyncness,
        trait_,
        context_path,
        event_enum_iden,
    )?;
    Ok(())
}

/// We want the `on_enter` function to be mandatory.
/// If it is not present, we add.
/// If it is present but has a different signature, we
/// return an error.
fn ensure_on_enter_fn(
    asyncness: Option<Async>,
    trait_: &mut ItemTrait,
    context_path: &Path,
) -> syn::Result<()> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let func =
        trait_.items.iter().find_map(|item| match item {
            TraitItem::Fn(f)
                if f.sig.ident == "on_enter" =>
            {
                Some(f)
            }
            _ => None,
        });

    if let Some(func) = func {
        const FIRST_ARG_ERROR: &str =
            "must accept `&mut self` as the first argument";
        const SECOND_ARG_ERROR: &str = "must accept `&mut {Context}` as the second argument";

        let mut inputs = func.sig.inputs.iter();

        let first_input = inputs.next();

        let Some(first_input) = first_input else {
            return Err(syn::Error::new_spanned(
                func,
                FIRST_ARG_ERROR,
            ));
        };

        let FnArg::Receiver(first_input) = first_input
        else {
            return Err(syn::Error::new_spanned(
                func,
                FIRST_ARG_ERROR,
            ));
        };

        if first_input.mutability.is_none()
            || first_input.reference.is_none()
        {
            return Err(syn::Error::new_spanned(
                first_input,
                FIRST_ARG_ERROR,
            ));
        }

        let second_input = inputs.next();

        let Some(second_input) = second_input else {
            return Err(syn::Error::new_spanned(
                second_input,
                SECOND_ARG_ERROR,
            ));
        };

        let FnArg::Typed(second_input) = second_input
        else {
            return Err(syn::Error::new_spanned(
                second_input,
                SECOND_ARG_ERROR,
            ));
        };

        #[allow(clippy::wildcard_enum_match_arm)]
        let second_input_ty = match &*second_input.ty {
            Type::Reference(r)
                if r.mutability.is_some() =>
            {
                r.elem.as_ref()
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    second_input,
                    SECOND_ARG_ERROR,
                ));
            }
        };

        if !matches!(second_input_ty, Type::Path(p) if p.path.eq(context_path))
        {
            return Err(syn::Error::new_spanned(
                second_input,
                SECOND_ARG_ERROR,
            ));
        }

        if inputs.next().is_some() {
            return Err(syn::Error::new_spanned(
                func,
                "must not have more than two arguments",
            ));
        }

        if !matches!(func.sig.output, ReturnType::Default) {
            return Err(syn::Error::new_spanned(
                func,
                "must not have a return type",
            ));
        }
    } else {
        let on_enter = syn::parse_quote! {
            #asyncness fn on_enter(&mut self, context: &mut #context_path) {}
        };

        trait_.items.push(TraitItem::Fn(on_enter));
    }

    Ok(())
}

fn ensure_on_exit_fn(
    asyncness: Option<Async>,
    trait_: &mut ItemTrait,
    context_path: &Path,
) -> syn::Result<()> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let func =
        trait_.items.iter().find_map(|item| match item {
            TraitItem::Fn(f)
                if f.sig.ident == "on_exit" =>
            {
                Some(f)
            }
            _ => None,
        });

    if let Some(func) = func {
        const FIRST_ARG_ERROR: &str =
            "must accept `&mut self` as the first argument";
        const SECOND_ARG_ERROR: &str = "must accept `&mut {Context}` as the second argument";

        let mut inputs = func.sig.inputs.iter();

        let first_input = inputs.next();

        let Some(first_input) = first_input else {
            return Err(syn::Error::new_spanned(
                func,
                FIRST_ARG_ERROR,
            ));
        };

        let FnArg::Receiver(first_input) = first_input
        else {
            return Err(syn::Error::new_spanned(
                func,
                FIRST_ARG_ERROR,
            ));
        };

        if first_input.mutability.is_none()
            || first_input.reference.is_none()
        {
            return Err(syn::Error::new_spanned(
                first_input,
                FIRST_ARG_ERROR,
            ));
        }

        let second_input = inputs.next();

        let Some(second_input) = second_input else {
            return Err(syn::Error::new_spanned(
                second_input,
                SECOND_ARG_ERROR,
            ));
        };

        let FnArg::Typed(second_input) = second_input
        else {
            return Err(syn::Error::new_spanned(
                second_input,
                SECOND_ARG_ERROR,
            ));
        };

        #[allow(clippy::wildcard_enum_match_arm)]
        let second_input_ty = match &*second_input.ty {
            Type::Reference(r)
                if r.mutability.is_some() =>
            {
                r.elem.as_ref()
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    second_input,
                    SECOND_ARG_ERROR,
                ));
            }
        };

        if !matches!(second_input_ty, Type::Path(p) if p.path.eq(context_path))
        {
            return Err(syn::Error::new_spanned(
                second_input,
                SECOND_ARG_ERROR,
            ));
        }

        if inputs.next().is_some() {
            return Err(syn::Error::new_spanned(
                func,
                "must not have more than two arguments",
            ));
        }

        if !matches!(func.sig.output, ReturnType::Default) {
            return Err(syn::Error::new_spanned(
                func,
                "must not have a return type",
            ));
        }
    } else {
        let on_exit = syn::parse_quote! {
            #asyncness fn on_exit(&mut self, context: &mut #context_path) {}
        };

        trait_.items.push(TraitItem::Fn(on_exit));
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn ensure_should_exit_fn(
    asyncness: Option<Async>,
    trait_: &mut ItemTrait,
    context_path: &Path,
    event_enum_ident: &Ident,
) -> syn::Result<()> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let func =
        trait_.items.iter().find_map(|item| match item {
            TraitItem::Fn(f)
                if f.sig.ident == "should_exit" =>
            {
                Some(f)
            }
            _ => None,
        });

    if let Some(func) = func {
        const FIRST_ARG_ERROR: &str =
            "must accept `&self` as the first argument";
        const SECOND_ARG_ERROR: &str = "must accept `&{Context}` as the second argument";
        const THRID_ARG_ERROR: &str = "must accept `&{EventEnum}` as the third argument";
        const RETURN_ERROR: &str = "must return a `bool`";

        let mut inputs = func.sig.inputs.iter();

        let first_input = inputs.next();

        let Some(first_input) = first_input else {
            return Err(syn::Error::new_spanned(
                func,
                FIRST_ARG_ERROR,
            ));
        };

        let FnArg::Receiver(first_input) = first_input
        else {
            return Err(syn::Error::new_spanned(
                func,
                FIRST_ARG_ERROR,
            ));
        };

        if first_input.mutability.is_some()
            || first_input.reference.is_none()
        {
            return Err(syn::Error::new_spanned(
                first_input,
                FIRST_ARG_ERROR,
            ));
        }

        let second_input = inputs.next();

        let Some(second_input) = second_input else {
            return Err(syn::Error::new_spanned(
                second_input,
                SECOND_ARG_ERROR,
            ));
        };

        let FnArg::Typed(second_input) = second_input
        else {
            return Err(syn::Error::new_spanned(
                second_input,
                SECOND_ARG_ERROR,
            ));
        };

        #[allow(clippy::wildcard_enum_match_arm)]
        let second_input_ty = match &*second_input.ty {
            Type::Reference(r)
                if r.mutability.is_none() =>
            {
                r.elem.as_ref()
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    second_input,
                    SECOND_ARG_ERROR,
                ));
            }
        };

        if !matches!(second_input_ty, Type::Path(p) if p.path.eq(context_path))
        {
            return Err(syn::Error::new_spanned(
                second_input,
                SECOND_ARG_ERROR,
            ));
        }

        let third_input = inputs.next();
        let Some(third_input) = third_input else {
            return Err(syn::Error::new_spanned(
                third_input,
                THRID_ARG_ERROR,
            ));
        };

        let FnArg::Typed(third_input) = third_input else {
            return Err(syn::Error::new_spanned(
                third_input,
                THRID_ARG_ERROR,
            ));
        };

        #[allow(clippy::wildcard_enum_match_arm)]
        let third_input_ty = match &*third_input.ty {
            Type::Reference(r)
                if r.mutability.is_none() =>
            {
                r.elem.as_ref()
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    third_input,
                    THRID_ARG_ERROR,
                ));
            }
        };

        if !matches!(third_input_ty, Type::Path(p) if p.path.is_ident(&event_enum_ident.to_string()))
        {
            return Err(syn::Error::new_spanned(
                third_input,
                THRID_ARG_ERROR,
            ));
        }

        if inputs.next().is_some() {
            return Err(syn::Error::new_spanned(
                func,
                "must not have more than three arguments",
            ));
        }

        let ReturnType::Type(_, return_ty) =
            &func.sig.output
        else {
            return Err(syn::Error::new_spanned(
                func,
                RETURN_ERROR,
            ));
        };

        if !matches!(return_ty.as_ref(), Type::Path(p) if p.path.is_ident("bool"))
        {
            return Err(syn::Error::new_spanned(
                func,
                RETURN_ERROR,
            ));
        }
    } else {
        let on_exit = syn::parse_quote! {
            #asyncness fn should_exit(&self, context: &#context_path, event: &#event_enum_ident) -> bool {
                true
            }
        };

        trait_.items.push(TraitItem::Fn(on_exit));
    }

    Ok(())
}
