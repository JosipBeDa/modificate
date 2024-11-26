use crate::{
    fields::{FieldInfo, Fields},
    serde::{extract_custom_serde, quote_custom_serde_payload_field},
};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

pub mod r#impl;

/// Create the payload ident: MyStruct => MyStructPayload
pub(super) fn payload_ident(original: &syn::Ident) -> syn::Ident {
    format_ident!("{}Payload", original)
}

pub(super) fn generate_struct(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &input.ident;

    let payload_ident = payload_ident(ident);

    // Collect only serde attributes to propagate to the payload
    let attributes = input
        .attrs
        .iter()
        .filter(|attr| attr.meta.path().is_ident("serde"))
        .cloned()
        .collect::<Vec<_>>();

    let visibility = &input.vis;

    let syn::Data::Struct(ref strct) = input.data else {
        abort!(
            input.span(),
            "#[derive(Payload)] can only be used on structs with named fields"
        )
    };

    let fields = Fields::collect(&attributes, &strct.fields);

    let mut payload_fields = vec![];
    let mut custom_serdes = vec![];

    for field in fields.0.iter() {
        let (payload_tokens, custom_serde) = map_payload_fields(field);
        payload_fields.push(payload_tokens);
        if let Some(custom_de) = custom_serde {
            custom_serdes.push(custom_de);
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let into_fields = fields
        .0
        .iter()
        .map(map_into_fields)
        .collect::<Vec<proc_macro2::TokenStream>>();

    let from_fields = fields
        .0
        .iter()
        .map(map_from_fields)
        .collect::<Vec<proc_macro2::TokenStream>>();

    let quoted = quote!(
        #[derive(Debug, ::validify::Validate, serde::Deserialize)]
        #(#attributes)*
        #visibility struct #payload_ident #ty_generics #where_clause {
            #(#payload_fields)*
        }

        impl #impl_generics From<#ident> for #payload_ident {
            fn from(original: #ident) -> Self {
                Self {
                    #(#into_fields)*
                }
            }
        }

        impl #impl_generics From<#payload_ident> for #ident {
            fn from(payload: #payload_ident) -> Self {
                Self {
                    #(#from_fields)*
                }
            }
        }

        #(#custom_serdes)*
    );

    quoted
}

fn map_payload_fields(
    info: &FieldInfo,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let ident = info.field.ident.as_ref().unwrap();

    let is_list = info.is_list();
    let is_option = info.is_option();
    let is_nested = info.is_nested_validify();

    let ty = &info.field.ty;

    // Grab all serde attributes and attempt to find custom deserializations
    let serde_attrs = info.serde_attrs();
    let (custom_serde, serde_attrs) = extract_custom_serde(&serde_attrs);

    let mut custom_de_attr = None;
    let mut custom_de_tokens = None;

    if let Some(custom_de) = custom_serde {
        let (custom_de_id, custom_de_toks) =
            quote_custom_serde_payload_field(ident, ty, custom_de, is_option);
        let custom_de_id = custom_de_id.to_string();
        custom_de_attr = Some(quote!(#[serde(deserialize_with = #custom_de_id)]));
        custom_de_tokens = Some(custom_de_toks);
    }

    // Grab all remaining attributes
    let remaining_attrs = info.remaining_attrs();

    if !is_option && !is_nested {
        return (
            quote!(
                #custom_de_attr
                #(#serde_attrs)*
                #(#remaining_attrs)*
                #[validate(required)]
                #ident: Option<#ty>,
            ),
            custom_de_tokens,
        );
    }

    if !is_option {
        let ident = info.field.ident.as_ref().unwrap();
        let ty = &info.field.ty;

        let syn::Type::Path(mut path) = ty.clone() else {
            abort!(
                info.field.span(),
                "Nested validifes must be structs implementing Validify or collections/options of"
            )
        };

        if is_list {
            payload_path_angle_bracketed(&mut path);
        } else {
            let ty_ident = &path.path.segments.last().unwrap().ident;
            path.path.segments.last_mut().unwrap().ident = format_ident!("{ty_ident}Payload");
        }

        let payload_type = syn::Type::Path(path);

        return (
            quote!(
                #custom_de_attr
                #(#serde_attrs)*
                #(#remaining_attrs)*
                #[validate(required)]
                #[validate]
                #ident: Option<#payload_type>,
            ),
            custom_de_tokens,
        );
    }

    if !is_nested {
        return (
            quote!(
                #custom_de_attr
                #(#serde_attrs)*
                #(#remaining_attrs)*
                #ident: #ty,
            ),
            custom_de_tokens,
        );
    }

    let syn::Type::Path(mut path) = ty.clone() else {
        abort!(
            info.field.span(),
            "Nested validifes must be structs implementing Validify"
        )
    };

    let syn::PathArguments::AngleBracketed(ref mut args) =
        path.path.segments.last_mut().unwrap().arguments
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    let syn::GenericArgument::Type(syn::Type::Path(ref mut inner_path)) =
        args.args.last_mut().unwrap()
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    if is_list {
        payload_path_angle_bracketed(inner_path);
    } else {
        let type_ident = &inner_path.path.segments.last().unwrap().ident;
        inner_path.path.segments.last_mut().unwrap().ident = format_ident!("{type_ident}Payload");
    }

    let payload_type = syn::Type::Path(path);

    (
        quote!(
            #custom_de_attr
            #(#serde_attrs)*
            #(#remaining_attrs)*
            #[validate]
            #ident: #payload_type,
        ),
        custom_de_tokens,
    )
}

fn map_from_fields(info: &FieldInfo) -> proc_macro2::TokenStream {
    let ident = info.field.ident.as_ref().unwrap();

    if info.is_option() {
        if info.is_nested_validify() && info.is_list() {
            return quote!(
                #ident: payload.#ident.map(|v|v.into_iter().map(|el|el.into()).collect()),
            );
        }

        if info.is_nested_validify() {
            return quote!(
                #ident: payload.#ident.map(|o|o.into()),
            );
        }

        quote!(
            #ident: payload.#ident,
        )
    } else {
        if info.is_nested_validify() && info.is_list() {
            return quote!(#ident: payload.#ident.unwrap().into_iter().map(|el|el.into()).collect(),);
        }

        if info.is_nested_validify() {
            return quote!(#ident: payload.#ident.unwrap().into(),);
        }

        quote!(#ident: payload.#ident.unwrap(),)
    }
}

fn map_into_fields(info: &FieldInfo) -> proc_macro2::TokenStream {
    let ident = info.field.ident.as_ref().unwrap();

    if info.is_option() {
        if info.is_nested_validify() && info.is_list() {
            return quote!(#ident: original.#ident.map(|v| v.into_iter().map(|el|el.into()).collect()),);
        }

        if info.is_nested_validify() {
            return quote!(#ident: original.#ident.map(|o|o.into()),);
        }

        quote!(#ident: original.#ident,)
    } else {
        if info.is_nested_validify() && info.is_list() {
            return quote!(#ident: Some(original.#ident.into_iter().map(|el|el.into()).collect()),);
        }

        if info.is_nested_validify() {
            return quote!(#ident: Some(original.#ident.into()),);
        }

        quote!(#ident: Some(original.#ident),)
    }
}

fn payload_path_angle_bracketed(path: &mut syn::TypePath) {
    // Type is contained in a List<T>. It will always have angle args and will
    // always be the last segment of the path
    let syn::PathArguments::AngleBracketed(ref mut args) =
        path.path.segments.last_mut().unwrap().arguments
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    let syn::GenericArgument::Type(syn::Type::Path(ref mut p)) = args.args.last_mut().unwrap()
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    let Some(segment) = p.path.segments.last_mut() else {
        abort!(p.span(), "Invalid path provided for validify")
    };

    segment.ident = format_ident!("{}Payload", segment.ident);
}

#[allow(dead_code)]
/// Could come in handy, parses the inner contents of an angle bracketed path and outputs
/// the original and the payload paths in a tuple
fn get_inner_path(ty: syn::Type) -> (syn::TypePath, syn::TypePath) {
    let syn::Type::Path(mut path) = ty else {
        abort!(
            ty.span(),
            "Nested validifes must be structs implementing Validify"
        )
    };

    let syn::PathArguments::AngleBracketed(ref mut args) =
        path.path.segments.last_mut().unwrap().arguments
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    let syn::GenericArgument::Type(syn::Type::Path(ref mut inner_path)) =
        args.args.last_mut().unwrap()
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    let original = inner_path.clone();

    let type_ident = &inner_path.path.segments.last_mut().unwrap().ident;

    inner_path.path.segments.last_mut().unwrap().ident = format_ident!("{type_ident}Payload");

    (original, inner_path.clone())
}
