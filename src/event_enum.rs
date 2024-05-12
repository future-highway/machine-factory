use quote::quote;
use syn::{Attribute, Ident, Path, Visibility};

pub struct EventEnumInput {
    pub attributes: Vec<Attribute>,
    pub visibility: Option<Visibility>,
    pub ident: Ident,
    pub event_paths: Vec<Path>,
}

pub fn event_enum(input: EventEnumInput) -> syn::Result<proc_macro2::TokenStream> {
    let EventEnumInput {
        attributes,
        visibility,
        ident,
        event_paths,
    } = input;

    let variants = event_paths
        .iter()
        .map(|path| {
            let Some(name) = path.segments.last().map(|s| &s.ident) else {
                return Err(syn::Error::new_spanned(
                    path,
                    "expected path to have at least one segment",
                ));
            };

            Ok(quote! {
                #name(#path)
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        #(#attributes)*
        #visibility enum #ident {
            #(#variants),*
        }
    })
}
