use syn::{
    FnArg, ItemTrait, Path, TraitItem, TraitItemFn, Type,
};

pub fn ensure_event_trait(
    trait_: &mut ItemTrait,
    context_path: &Path,
) -> syn::Result<()> {
    ensure_pre_transition_fn(trait_, context_path)?;
    ensure_post_transition_fn(trait_, context_path)?;
    Ok(())
}

fn ensure_pre_transition_fn(
    trait_: &mut ItemTrait,
    context_path: &Path,
) -> syn::Result<()> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let func =
        trait_.items.iter().find_map(|item| match item {
            TraitItem::Fn(f)
                if f.sig.ident == "pre_transition" =>
            {
                Some(f)
            }
            _ => None,
        });

    if let Some(func) = func {
        check_transition_fn(func, context_path)?;
    } else {
        let pre_transition = syn::parse_quote! {
            fn pre_transition(&mut self, _context: &mut #context_path) {}
        };

        trait_.items.push(TraitItem::Fn(pre_transition));
    }

    Ok(())
}

fn ensure_post_transition_fn(
    trait_: &mut ItemTrait,
    context_path: &Path,
) -> syn::Result<()> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let func =
        trait_.items.iter().find_map(|item| match item {
            TraitItem::Fn(f)
                if f.sig.ident == "post_transition" =>
            {
                Some(f)
            }
            _ => None,
        });

    if let Some(func) = func {
        check_transition_fn(func, context_path)?;
    } else {
        let post_transition = syn::parse_quote! {
            fn post_transition(&mut self, _context: &mut #context_path) {}
        };

        trait_.items.push(TraitItem::Fn(post_transition));
    }

    Ok(())
}

fn check_transition_fn(
    func: &TraitItemFn,
    context_path: &Path,
) -> syn::Result<()> {
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

    let FnArg::Receiver(first_input) = first_input else {
        return Err(syn::Error::new_spanned(
            func,
            FIRST_ARG_ERROR,
        ));
    };

    if first_input.mutability.is_none() {
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

    let FnArg::Typed(second_input) = second_input else {
        return Err(syn::Error::new_spanned(
            second_input,
            SECOND_ARG_ERROR,
        ));
    };

    #[allow(clippy::wildcard_enum_match_arm)]
    let second_input_ty = match &*second_input.ty {
        Type::Reference(r) if r.mutability.is_some() => {
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
            "must accept exactly two arguments",
        ));
    }

    Ok(())
}
