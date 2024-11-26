use crate::{
    serde::RenameRule,
    validate::{r#impl::collect_validation, validation::Validator},
    validify::{modifier::Modifier, r#impl::collect_modifiers},
};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Ident};

/// Holds variants of an enum and their respective fields.
#[derive(Debug)]
pub struct Variants(Vec<VariantInfo>);

impl Variants {
    /// Returns the variants of an enum, its fields, and whether the variant is named.
    ///
    /// In enums, each variant's attributes are used to rename fields as opposed to the
    /// top level ones, since in enums the top level attributes rename the variants and we
    /// don't care about those.
    pub fn collect(input: &syn::DataEnum) -> Self {
        let mut variants = Vec::new();

        for variant in input.variants.iter() {
            let mut fields = Fields::collect(&variant.attrs, &variant.fields);

            fields.0.iter_mut().for_each(|field| {
                let ident = match field.name_or_index {
                    NameOrIndex::Name(ref n) => format_ident!("{n}"),
                    NameOrIndex::Index(i) => format_ident!("arg_{i}"),
                };
                field.ident_override = Some(ident.clone());
            });

            variants.push(VariantInfo::new(
                variant.ident.clone(),
                fields,
                matches!(variant.fields, syn::Fields::Named(_)),
            ));
        }

        Self(variants)
    }

    /// Output the necessary tokens for variant, and in turn field
    /// validation when implementing `Validate`.
    pub fn to_validate_tokens(&self) -> proc_macro2::TokenStream {
        let field_validation = self.0.iter().fold(
            quote!(),
            |mut tokens,
             VariantInfo {
                 ident: ref variant,
                 ref fields,
                 ref named,
             }| {
                let variant_fields = fields.0.iter().map(|field| match field.name_or_index {
                    NameOrIndex::Name(ref n) => format_ident!("{n}"),
                    NameOrIndex::Index(i) => format_ident!("arg_{i}"),
                });

                let variant_field_tokens = quote!(#(#variant_fields),*);

                let field_validation = fields.to_validate_tokens();

                if *named {
                    tokens.extend(quote!(
                            Self::#variant { #variant_field_tokens } => { #(#field_validation)* }
                    ));
                } else {
                    tokens.extend(quote!(
                        Self::#variant(#variant_field_tokens) => { #(#field_validation)* }
                    ));
                }

                tokens
            },
        );

        quote!(match self { #field_validation })
    }

    pub fn to_modify_tokens(&self) -> Vec<proc_macro2::TokenStream> {
        let mut modifiers = vec![];

        for variant in self.0.iter() {
            let VariantInfo {
                ident: ref variant,
                ref fields,
                ref named,
            } = variant;

            let variant_fields = fields.0.iter().map(|field| match field.name_or_index {
                NameOrIndex::Name(ref n) => format_ident!("{n}"),
                NameOrIndex::Index(i) => format_ident!("arg_{i}"),
            });

            let variant_field_tokens = quote!(#(ref mut #variant_fields),*);

            let field_modifiers = fields.to_modify_tokens();

            if *named {
                let tokens =
                    quote!( Self::#variant { #variant_field_tokens } => { #(#field_modifiers)* });
                modifiers.push(tokens);
            } else {
                let tokens =
                    quote!( Self::#variant ( #variant_field_tokens ) => { #(#field_modifiers)* });
                modifiers.push(tokens);
            }
        }

        modifiers
    }
}

/// Holds the combined validations and modifiers for one enum variant.
#[derive(Debug)]
pub struct VariantInfo {
    pub ident: syn::Ident,
    pub fields: Fields,
    pub named: bool,
}

impl VariantInfo {
    fn new(ident: syn::Ident, fields: Fields, named: bool) -> Self {
        Self {
            ident,
            fields,
            named,
        }
    }
}

#[derive(Debug)]
pub struct Fields(pub Vec<FieldInfo>);

impl Fields {
    /// Used by both the `Validate` and `Validify` implementations. Validate ignores the modifiers.
    pub fn collect(attributes: &[syn::Attribute], input: &syn::Fields) -> Self {
        let rename_rule = crate::serde::find_rename_all(attributes);

        let fields = input
            .iter()
            .enumerate()
            .map(|(i, field)| {
                {
                    let name_or_index = field
                        .ident
                        .as_ref()
                        .map(|i| NameOrIndex::Name(i.to_string()))
                        .unwrap_or(NameOrIndex::Index(i));

                    let validations = collect_validation(field);
                    let modifiers = collect_modifiers(field);

                    // The original name refers to the field name set with serde rename.
                    let original_name = crate::serde::find_rename(field);

                    FieldInfo::new(
                        field.clone(),
                        name_or_index,
                        original_name,
                        validations,
                        modifiers,
                        rename_rule,
                    )
                }
            })
            .collect::<Vec<_>>();

        Self(fields)
    }

    /// Output the necessary tokens for field validation when implementing `Validate`.
    pub fn to_validate_tokens(&self) -> Vec<proc_macro2::TokenStream> {
        let mut validations = vec![];

        for field_info in self.0.iter() {
            let tokens = field_info.to_validate_tokens();
            validations.extend(tokens);
        }

        validations
    }

    /// Creates a token stream applying the modifiers based on the field annotations.
    pub fn to_modify_tokens(&self) -> Vec<proc_macro2::TokenStream> {
        let mut modifications = vec![];

        for field_info in self.0.iter() {
            let modification = field_info.to_modify_tokens();
            modifications.extend(modification);
        }

        modifications
    }
}

/// Holds the combined validations and modifiers for one field.
#[derive(Debug)]
pub struct FieldInfo {
    /// The original field
    pub field: syn::Field,

    /// The index of the tuple value if this is an unnamed field.
    /// The field's name in string form if named.,
    pub name_or_index: NameOrIndex,

    /// The field's original name if annotated with `serde(rename)``
    pub original_name: Option<String>,

    /// Validation annotations
    pub validations: Vec<Validator>,

    /// Modifier annotations
    pub modifiers: Vec<Modifier>,

    /// Obtained from `serde(rename_all)`
    pub rename_rule: Option<RenameRule>,

    /// Used when in enum and the field has to be pattern matched.
    pub ident_override: Option<Ident>,
}

impl FieldInfo {
    pub fn new(
        field: syn::Field,
        name_or_index: NameOrIndex,
        original_name: Option<String>,
        validations: Vec<Validator>,
        modifiers: Vec<Modifier>,
        rename_rule: Option<RenameRule>,
    ) -> Self {
        FieldInfo {
            field,
            name_or_index,
            original_name,
            validations,
            modifiers,
            rename_rule,
            ident_override: None,
        }
    }

    /// Returns the field name or the name from serde rename in case of named field.
    /// Returns the index if the field is unnamed.
    /// Used for errors.
    pub fn name(&self) -> String {
        if let Some(ref original_name) = self.original_name {
            return original_name.clone();
        }

        match (self.name_or_index.clone(), self.rename_rule) {
            (NameOrIndex::Name(name), None) => name,
            (NameOrIndex::Index(index), None) => index.to_string(),
            (NameOrIndex::Name(name), Some(ref rule)) => rule.apply_to_field(&name),
            (NameOrIndex::Index(index), Some(_)) => index.to_string(),
        }
    }

    // QUOTING

    /// Returns tokens for the `impl Validate` block.
    /// Child validation are always at the start of the token stream.
    pub fn to_validate_tokens(&self) -> Vec<proc_macro2::TokenStream> {
        let mut child_validation = vec![];
        let mut validation = vec![];

        for validator in self.validations.iter() {
            let validator_param = self.validator_param_tokens();

            let tokens = validator.to_validate_tokens(self, validator_param);

            match tokens {
                crate::tokens::ValidationTokens::Normal(v) => validation.push(v),
                crate::tokens::ValidationTokens::Nested(v) => child_validation.push(v),
            }
        }

        child_validation.extend(validation);
        child_validation
    }

    /// Returns the modification tokens as the first element and any nested validifes as the second.
    pub fn to_modify_tokens(&self) -> Vec<proc_macro2::TokenStream> {
        let mut modifications = vec![];

        for modifier in self.modifiers.iter() {
            let tokens = modifier.to_validify_tokens(self);
            modifications.push(tokens);
        }

        modifications
    }

    /// Generates the tokens that get passed to validation functions.
    ///
    /// If the field has an ident override in case of enums, quote it directly.
    /// All enum variants use the ident override.
    ///
    /// If the field is an `Option`, quote the field ident as we always
    /// wrap optional fields in an `if let Some(ref _)`.
    ///
    /// If the field is a reference the returned tokens are `self.field`.
    ///
    /// If the field is owned, the tokens are `&self.field`.
    pub fn validator_param_tokens(&self) -> proc_macro2::TokenStream {
        if let Some(ref ident) = self.ident_override {
            return quote!(#ident);
        }

        let ident = &self.field.ident;

        if self.is_option() {
            return quote!(#ident);
        }

        match self.field.ty {
            syn::Type::Reference(_) => {
                quote!(self.#ident)
            }
            syn::Type::Array(_)
            | syn::Type::Path(_)
            | syn::Type::Paren(_)
            | syn::Type::Slice(_)
            | syn::Type::Tuple(_)
            | syn::Type::Group(_) => quote!(&self.#ident),
            _ => abort!(self.field.ty.span(), "unsupported type"),
        }
    }

    /// Returns `self.#ident`, unless the field is an option in which case it just
    /// returns an `#ident` as we always do a `if let` check on Option fields
    pub fn modifier_param_tokens(&self) -> proc_macro2::TokenStream {
        if let Some(ident) = &self.ident_override {
            return quote!(#ident);
        }

        let ident = &self.field.ident;

        if self.is_reference() {
            abort!(
                ident.span(),
                "Fields containing modifiers must contain owned data"
            )
        }

        if self.is_option() {
            quote!(#ident)
        } else {
            quote!(self.#ident)
        }
    }

    /// Wrap the provided tokens in an `if let Some` block if the field is an option.
    pub fn wrap_tokens_if_option(
        &self,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        if !self.is_option() {
            return tokens;
        }

        let field_ident = self
            .ident_override
            .as_ref()
            .map(|i| quote!(#i))
            .unwrap_or_else(|| {
                let i = self.field.ident.as_ref().unwrap();
                quote!(#i)
            });

        let mut this = if self.is_reference() {
            quote!(#field_ident)
        } else {
            quote!(ref #field_ident)
        };

        let field_ident = if self.ident_override.is_some() {
            field_ident
        } else {
            quote!(self.#field_ident)
        };

        let mut ty = &self.field.ty;

        while let Some(typ) = try_extract_option(ty) {
            this = quote!(Some(#this));
            ty = typ;
        }

        quote!(
            if let #this = #field_ident {
                #tokens
            }
        )
    }

    /// Wrap the quoted output of a validation with a for loop if
    /// the field type is a collection.
    pub fn wrap_validator_if_collection(
        &self,
        param: proc_macro2::TokenStream,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let field_name = self.name();

        // When we're using an option, we'll have the field unwrapped, so we should not access it
        // through `self`.
        let prefix = (!self.is_option() && self.ident_override.is_none()).then(|| quote! { self. });

        // When iterating over a list, the iterator has Item=T, while a map yields Item=(K, V), and
        // we're only interested in V.
        if self.is_list() {
            quote!(
                for (i, item) in #prefix #param.iter().enumerate() {
                    if let Err(mut errs) = item.validate() {
                        errs.errors_mut().iter_mut().for_each(|err| err.set_location_idx(i, #field_name));
                        errors.merge(errs);
                    }
                }
            )
        } else if self.is_map() {
            quote!(
                for (key, item) in #prefix #param.iter() {
                    if let Err(mut errs) = item.validate() {
                        errs.errors_mut().iter_mut().for_each(|err| err.set_location_idx(key, #field_name));
                        errors.merge(errs);
                    }
                }
            )
        } else {
            tokens
        }
    }

    pub fn wrap_modifier_if_option(
        &self,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let field_ident = &self.field.ident;

        if self.is_option() {
            let this = self.option_self_tokens_modifications();
            return quote!(
                if let #this = self.#field_ident.as_mut() {
                    #tokens
                }
            );
        }

        tokens
    }

    /// Wrap the quoted output of a modification in a for loop if
    /// the field type is a collection.
    pub fn wrap_modifier_if_collection(
        &self,
        param: proc_macro2::TokenStream,
        tokens: proc_macro2::TokenStream,
        modifier: &Modifier,
    ) -> proc_macro2::TokenStream {
        if !self.is_list() {
            return tokens;
        }

        let modified = match modifier {
            Modifier::Trim => quote!(el.trim().to_string()),
            Modifier::Uppercase => quote!(el.to_uppercase()),
            Modifier::Lowercase => quote!(el.to_lowercase()),
            Modifier::Capitalize => {
                quote!(::std::format!("{}{}", &el[0..1].to_uppercase(), &el[1..]))
            }
            _ => unreachable!("modifier is never wrapped"),
        };

        quote!(
            for el in #param.iter_mut() {
                *el = #modified
            }
        )
    }

    /// Return all the field's attributes that are unrelated to validify and serde
    pub fn remaining_attrs(&self) -> Vec<&syn::Attribute> {
        self.field
            .attrs
            .iter()
            .filter(|attr| !validify_attr_check(attr) && !attr.path().is_ident("serde"))
            .collect()
    }

    /// Return all the field's attributes related to `serde`
    pub fn serde_attrs(&self) -> Vec<&syn::Attribute> {
        self.field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
            .collect()
    }

    // ASSERTION

    /// Returns true if the field is an option.
    pub fn is_option(&self) -> bool {
        let syn::Type::Path(ref p) = self.field.ty else {
            return false;
        };

        p.path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "Option")
    }

    /// Returns true if the field is &'_ T, or Option<&'_ T>.
    pub fn is_reference(&self) -> bool {
        is_reference(&self.field.ty)
    }

    pub fn is_list(&self) -> bool {
        is_list(&self.field.ty)
    }

    pub fn is_map(&self) -> bool {
        is_map(&self.field.ty)
    }

    /// Returns true if the field is annotated with `#[validify]`
    pub fn is_nested_validify(&self) -> bool {
        self.field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("validify") && attr.meta.require_path_only().is_ok())
    }

    fn option_self_tokens_modifications(&self) -> proc_macro2::TokenStream {
        let ident = &self.field.ident;
        let mut tokens = quote!(#ident);
        let mut ty = &self.field.ty;

        while let Some(typ) = try_extract_option(ty) {
            tokens = quote!(Some(#tokens));
            ty = typ;
        }
        tokens
    }
}

/// Check whether the attribute belongs to validify, i.e. is it
/// `validate`, `modify`, or `validify`.
pub fn validify_attr_check(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("validify")
        || attr.path().is_ident("validate")
        || attr.path().is_ident("modify")
}

fn is_reference(ty: &syn::Type) -> bool {
    // Strip any `Option`s
    if let Some(ty) = try_extract_option(ty) {
        return is_reference(ty);
    }

    matches!(ty, syn::Type::Reference(_))
}

fn is_list(ty: &syn::Type) -> bool {
    if let Some(ty) = try_extract_option(ty) {
        return is_list(ty);
    }

    // We consider arrays lists
    if let syn::Type::Array(_) = ty {
        return true;
    }

    // If it's not a path, it's not a list
    let syn::Type::Path(p) = ty else {
        return false;
    };

    // Always check the last arg such as in `std::vec::Vec`
    let Some(seg) = p.path.segments.last() else {
        return false;
    };

    seg.ident == "Vec"
        || seg.ident == "HashSet"
        || seg.ident == "BTreeSet"
        || seg.ident == "IndexSet"
}

/// Used in [FieldInfo].
#[derive(Debug, Clone)]
pub enum NameOrIndex {
    Name(String),
    Index(usize),
}

fn is_map(ty: &syn::Type) -> bool {
    if let Some(ty) = try_extract_option(ty) {
        return is_map(ty);
    }

    let syn::Type::Path(p) = ty else {
        return false;
    };

    // Always check the last arg such as in `std::vec::Vec`
    let Some(seg) = p.path.segments.last() else {
        return false;
    };

    seg.ident == "HashMap" || seg.ident == "BTreeMap" || seg.ident == "IndexMap"
}

fn try_extract_option(ty: &syn::Type) -> Option<&syn::Type> {
    // Struct definitions always contain paths if they're options
    let syn::Type::Path(p) = ty else {
        return None;
    };

    // Always check the last arg such as in `std::option::Option`
    let seg = p.path.segments.last()?;

    if &seg.ident != "Option" {
        return None;
    }

    // Option<T> always has arguments in angle brackets
    let syn::PathArguments::AngleBracketed(ref ab) = seg.arguments else {
        return None;
    };

    // Option always contains a single generic arg
    let arg = ab.args.last()?;

    match arg {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}
