use std::collections::HashMap;

use proc_macro_error::abort;
use quote::ToTokens;
use syn::spanned::Spanned;

use crate::{
    types::{Modifier, Validator},
    validate::r#impl::collect_validations,
    validify::r#impl::collect_modifiers,
};

/// Holds the combined validations and modifiers for one field
#[derive(Debug)]
pub struct FieldInformation {
    pub field: syn::Field,
    pub field_type: String,
    pub name: String,
    pub validations: Vec<Validator>,
    pub modifiers: Vec<Modifier>,
}

impl FieldInformation {
    pub fn new(
        field: syn::Field,
        field_type: String,
        name: String,
        validations: Vec<Validator>,
        modifiers: Vec<Modifier>,
    ) -> Self {
        FieldInformation {
            field,
            field_type,
            name,
            validations,
            modifiers,
        }
    }
}

/// Used by both the `Validate` and `Validify` implementations. Validate ignores the modifiers.
pub fn collect_field_info(
    input: &syn::DeriveInput,
    allow_refs: bool,
) -> Result<Vec<FieldInformation>, syn::Error> {
    let mut fields = collect_fields(input);

    let field_types = map_field_types(&fields, allow_refs);

    let mut final_validations = vec![];

    for field in fields.drain(..) {
        let field_ident = field
            .ident
            .as_ref()
            .expect("Found unnamed field")
            .to_string();

        let (validations, modifiers) = collect_field_attributes(&field, &field_types)?;

        final_validations.push(FieldInformation::new(
            field,
            field_types.get(&field_ident).unwrap().clone(),
            field_ident,
            validations,
            modifiers,
        ));
    }

    Ok(final_validations)
}

/// Find the types (as string) for each field of the struct. The `allow_refs`, if false, will error if
/// the field is a reference. This is needed for modifiers as we do not allow references when deriving
/// `Validifty`. References in `Validate` are OK.
pub fn map_field_types(fields: &[syn::Field], allow_refs: bool) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for field in fields {
        let field_ident = field
            .ident
            .clone()
            .expect("Found unnamed field")
            .to_string();

        let field_type = match field.ty {
            syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                path.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            syn::Type::Reference(syn::TypeReference {
                ref lifetime,
                ref elem,
                ..
            }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                let mut name = tokens.to_string().replace(' ', "");
                if lifetime.is_some() {
                    name.insert(0, '&')
                }
                name
            }
            syn::Type::Group(syn::TypeGroup { ref elem, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            ref ty => {
                let mut field_type = proc_macro2::TokenStream::new();
                ty.to_tokens(&mut field_type);
                field_type.to_string().replace(' ', "")
            }
        };
        if field_type.contains('&') && !allow_refs {
            abort!(
                field.span(),
                "Validify must be implemented for structs with owned data, if you just need validation and not modification, use Validate instead"
            )
        }
        types.insert(field_ident, field_type);
    }

    types
}

pub fn collect_fields(input: &syn::DeriveInput) -> Vec<syn::Field> {
    match input.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                abort!(
                    fields.span(),
                    "#[derive(Validate/Validify)] can only be used on structs with named fields"
                );
            }

            fields.iter().cloned().collect::<Vec<syn::Field>>()
        }
        _ => abort!(
            input.span(),
            "#[derive(Validate/Validify)] can only be used on structs with named fields"
        ),
    }
}

/// Find everything we need to know about a field: its real name if it's changed from the serialization
/// and the list of validators to run on it
pub fn collect_field_attributes(
    field: &syn::Field,
    field_types: &HashMap<String, String>,
) -> Result<(Vec<Validator>, Vec<Modifier>), syn::Error> {
    let field_ident = field.ident.clone().unwrap().to_string();
    let field_type = field_types.get(&field_ident).unwrap();

    let mut validators = vec![];
    let mut modifiers = vec![];

    collect_validations(&mut validators, field, field_type);
    collect_modifiers(&mut modifiers, field);

    Ok((validators, modifiers))
}