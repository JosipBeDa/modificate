use crate::fields::FieldInfo;
use quote::quote;

#[derive(Debug, PartialEq, Eq)]
pub enum Modifier {
    Trim,
    Uppercase,
    Lowercase,
    Capitalize,
    Custom { function: syn::Path },
    Nested,
}

impl Modifier {
    /// Returns direct modification tokens as the first element and any nested validify tokens as the second element.
    /// Necessary because we need both in case a nested validify occurs. In that case, the first element will have the
    /// necessary modification tokens for nested elements in the `Modify` impl while the second will have the tokens
    /// for the `Validify` impl.
    pub fn to_validify_tokens(&self, field_info: &FieldInfo) -> proc_macro2::TokenStream {
        let param = field_info.modifier_param_tokens();

        match self {
            Modifier::Trim => {
                let tokens = if field_info.ident_override.is_some() || field_info.is_option() {
                    quote!(
                        *#param = #param.trim().to_string();
                    )
                } else {
                    quote!(
                        #param = #param.trim().to_string();
                    )
                };
                field_info.wrap_modifier_if_option(
                    field_info.wrap_modifier_if_collection(param, tokens, self),
                )
            }
            Modifier::Uppercase => {
                let tokens = if field_info.ident_override.is_some() || field_info.is_option() {
                    quote!(
                        *#param = #param.to_uppercase();
                    )
                } else {
                    quote!(
                        #param = #param.to_uppercase();
                    )
                };
                field_info.wrap_modifier_if_option(
                    field_info.wrap_modifier_if_collection(param, tokens, self),
                )
            }
            Modifier::Lowercase => {
                let tokens = if field_info.ident_override.is_some() || field_info.is_option() {
                    quote!(
                        *#param = #param.to_lowercase();
                    )
                } else {
                    quote!(
                        #param = #param.to_lowercase();
                    )
                };
                field_info.wrap_modifier_if_option(
                    field_info.wrap_modifier_if_collection(param, tokens, self),
                )
            }
            Modifier::Capitalize => {
                let tokens = if field_info.ident_override.is_some() || field_info.is_option() {
                    quote!(
                      *#param = ::std::format!("{}{}", &#param[0..1].to_uppercase(), &#param[1..]);
                    )
                } else {
                    quote!(
                      #param = ::std::format!("{}{}", &#param[0..1].to_uppercase(), &#param[1..]);
                    )
                };
                field_info.wrap_modifier_if_option(
                    field_info.wrap_modifier_if_collection(param, tokens, self),
                )
            }
            Modifier::Custom { function } => {
                let tokens = if field_info.ident_override.is_some() || field_info.is_option() {
                    quote!(
                        #function(#param);
                    )
                } else {
                    quote!(
                        #function(&mut #param);
                    )
                };
                field_info.wrap_modifier_if_option(tokens)
            }
            Modifier::Nested => {
                let modifications = if field_info.is_list() {
                    quote!(
                        for el in #param.iter_mut() {
                            el.modify();
                        }
                    )
                } else {
                    quote!(#param.modify();)
                };

                field_info.wrap_modifier_if_option(modifications)
            }
        }
    }
}
