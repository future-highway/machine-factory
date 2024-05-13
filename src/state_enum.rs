use quote::quote;
use syn::{Attribute, Ident, Path, Visibility};

pub struct StateEnumInput {
    pub attributes: Vec<Attribute>,
    pub visibility: Option<Visibility>,
    pub ident: Ident,
    pub state_paths: Vec<Path>,
}

pub fn state_enum(
    input: StateEnumInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let StateEnumInput {
        attributes,
        visibility,
        ident,
        state_paths,
    } = input;

    let variants = state_paths
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
