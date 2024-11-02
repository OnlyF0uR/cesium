use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{parse::Parser, parse_macro_input};

// For structs this will be used as #[cesium(contract_state)] and for
// the actual module it will be used as #[cesium] so must have support for args
#[proc_macro_attribute]
pub fn cesium(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut expanded = quote! {};

    if let Ok(mut func) = syn::parse::<syn::ItemFn>(item.clone()) {
        let attr = parse_macro_input!(attr as syn::AttributeArgs);

        let mut external_func = false;
        for a in attr {
            if let syn::NestedMeta::Meta(syn::Meta::Path(p)) = a {
                if p.is_ident("external_func") {
                    external_func = true;
                }
            }
        }

        if external_func {
            // Make mod item extern "C"
            func.sig.abi = Some(syn::Abi {
                extern_token: syn::token::Extern {
                    span: Span::call_site(),
                },
                name: Some(syn::LitStr::new("C", Span::call_site())),
            });
            // We need to add #[no_mangle] to the function
            expanded = quote! {
                #[no_mangle]
                #func
            };
        }
    } else if let Ok(mut struct_item) = syn::parse::<syn::ItemStruct>(item.clone()) {
        let attr = parse_macro_input!(attr as syn::AttributeArgs);

        let mut contract_state = false;
        for a in attr {
            if let syn::NestedMeta::Meta(syn::Meta::Path(p)) = a {
                if p.is_ident("contract_state") {
                    contract_state = true;
                }
            }
        }
        if !contract_state {
            return TokenStream::from(
                syn::Error::new(
                    Span::call_site().into(),
                    "requires contract_state attribute for structs.",
                )
                .to_compile_error(),
            );
        }

        // Collect all the fields of the struct
        let struct_item_clone = struct_item.clone();
        let fields = match struct_item_clone.fields {
            syn::Fields::Named(fields) => fields.named,
            syn::Fields::Unnamed(fields) => fields.unnamed,
            syn::Fields::Unit => {
                return TokenStream::from(
                    syn::Error::new(
                        Span::call_site().into(),
                        "cesium macro can only be used on struct with fields.",
                    )
                    .to_compile_error(),
                )
            }
        };

        let mut new_block = quote! {};
        match &mut struct_item.fields {
            syn::Fields::Named(fields) => {
                let loop_fields = fields.clone();
                // loop through named fields
                for field in loop_fields.named.iter() {
                    // add a new field to the struct
                    let field_name = field.ident.clone().unwrap();
                    let new_field_name =
                        Ident::new(&format!("{}_cached", field_name), Span::call_site());
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! { #new_field_name: bool })
                            .unwrap(),
                    );

                    // for both the new and old field add default value
                    new_block = quote! {
                        #new_block
                        #field_name: Default::default(),
                        #new_field_name: Default::default(),
                    };
                }
            }
            _ => (),
        };

        // A function per field to convert value to bytes
        let mut to_bytes = quote! {};
        // Field of the define_all function

        // Use the fields without cached fields
        let mut field_index = 0;
        for field in fields {
            let field_ident = field.ident.unwrap();
            let field_name = field_ident.to_string();
            let field_name_cached =
                Ident::new(&format!("{}_cached", field_name), Span::call_site());
            let field_type = field.ty.clone();

            let get_indent = Ident::new(&format!("get_{}", field_name), Span::call_site());
            let set_indent = Ident::new(&format!("set_{}", field_name), Span::call_site());

            // println!("{}", field_type.to_token_stream().to_string());

            if field_type.to_token_stream().to_string() == "String" {
                to_bytes = quote! {
                    #to_bytes
                    pub fn #get_indent(&mut self) -> Result<String, selenide_sdk::data::StateError> {
                      if self.#field_name_cached {
                        return Ok(self.#field_ident.clone());
                      }

                      let result = State::get(#field_index);
                      if let Err(e) = result {
                        return Err(e);
                      }

                      let result = result.unwrap();
                      if result.is_none() {
                        return Err(selenide_sdk::data::StateError::NoReturnData);
                      }

                      let result_str = std::str::from_utf8(&result.unwrap()).unwrap().to_string();
                      self.#field_ident = result_str.clone();
                      self.#field_name_cached = true;

                      Ok(result_str)
                    }
                    pub fn #set_indent(&mut self, value: &str) -> Result<(), selenide_sdk::data::StateError> {
                      let result = State::set(#field_index, value.as_bytes());
                      if result.is_ok() {
                        self.#field_ident = value.to_owned();
                        self.#field_name_cached = true;
                      }
                      result
                    }
                };
            } else if field_type.to_token_stream().to_string() == "Vec < u8 >" {
                to_bytes = quote! {
                    #to_bytes
                    pub fn #get_indent(&mut self) -> Result<Vec<u8>, selenide_sdk::data::StateError> {
                      if self.#field_name_cached {
                        return Ok(self.#field_ident.clone());
                      }

                      let result = State::get(#field_index);
                      if let Err(e) = result {
                        return Err(e);
                      }

                      let result = result.unwrap();
                      if result.is_none() {
                        return Err(selenide_sdk::data::StateError::NoReturnData);
                      }

                      let result = result.unwrap();

                      self.#field_ident = result.clone();
                      self.#field_name_cached = true;

                      Ok(result)
                    }
                    pub fn #set_indent(&mut self, value: Vec<u8>) -> Result<(), selenide_sdk::data::StateError> {
                      let result = State::set(#field_index, &value);
                      if result.is_ok() {
                        self.#field_ident = value;
                        self.#field_name_cached = true;
                      }
                      result
                    }
                };
            } else if field_type.to_token_stream().to_string() == "bool" {
                to_bytes = quote! {
                    #to_bytes
                    pub fn #get_indent(&mut self) -> Result<bool, selenide_sdk::data::StateError> {
                      if self.#field_name_cached {
                        return Ok(self.#field_ident);
                      }

                      let result = State::get(#field_index);
                      if let Err(e) = result {
                        return Err(e);
                      }

                      let result = result.unwrap();
                      if result.is_none() {
                        return Err(selenide_sdk::data::StateError::NoReturnData);
                      }

                      let result = result.unwrap();

                      self.#field_ident = result[0] == 1;
                      self.#field_name_cached = true;

                      Ok(result[0] == 1)
                    }
                    pub fn #set_indent(&mut self, value: bool) -> Result<(), selenide_sdk::data::StateError> {
                      let byte_value: u8 = if value { 1 } else { 0 };
                      let byte_slice: &[u8] = &[byte_value];
                      let result = State::set(#field_index, byte_slice);
                      if result.is_ok() {
                        self.#field_ident = value;
                        self.#field_name_cached = true;
                      }
                      result
                    }
                };
            } else if field_type.to_token_stream().to_string() == "u256"
                || field_type.to_token_stream().to_string() == "u128"
                || field_type.to_token_stream().to_string() == "u64"
                || field_type.to_token_stream().to_string() == "u32"
                || field_type.to_token_stream().to_string() == "u16"
                || field_type.to_token_stream().to_string() == "u8"
                || field_type.to_token_stream().to_string() == "i256"
                || field_type.to_token_stream().to_string() == "i128"
                || field_type.to_token_stream().to_string() == "i64"
                || field_type.to_token_stream().to_string() == "i32"
                || field_type.to_token_stream().to_string() == "i16"
                || field_type.to_token_stream().to_string() == "i8"
            {
                to_bytes = quote! {
                    #to_bytes
                    pub fn #get_indent(&mut self) -> Result<#field_type, selenide_sdk::data::StateError> {
                      if self.#field_name_cached {
                        return Ok(self.#field_ident);
                      }

                      let result = State::get(#field_index);
                      if let Err(e) = result {
                        return Err(e);
                      }

                      let result = result.unwrap();
                      if result.is_none() {
                        return Err(selenide_sdk::data::StateError::NoReturnData);
                      }

                      let result = result.unwrap();

                      self.#field_ident = #field_type::from_le_bytes(result.as_slice().try_into().unwrap());
                      self.#field_name_cached = true;

                      Ok(self.#field_ident)
                    }
                    pub fn #set_indent(&mut self, value: #field_type) -> Result<(), selenide_sdk::data::StateError> {
                      let byte_value = value.to_le_bytes();
                      let result = State::set(#field_index, &byte_value);
                      if result.is_ok() {
                        self.#field_ident = value;
                        self.#field_name_cached = true;
                      }
                      result
                    }
                };
            } else {
                return TokenStream::from(
                    syn::Error::new(
                        Span::call_site().into(),
                        "cesium macro can only be used on struct with fields of type String, Vec<u8>, bool, u256, u128, u64, u32, u16, u8, i256, i128, i64, i32, i16, i8.",
                    )
                    .to_compile_error(),
                );
            }
            field_index = field_index + 1;
        }

        // Create the entire struct impl
        let struct_name = struct_item.ident.clone();

        // add fields to struct item
        let struct_item_clone = struct_item.clone();
        expanded = quote! {
            #struct_item_clone
            impl #struct_name {
                pub fn new() -> Self {
                    Self {
                        #new_block
                    }
                }
                pub fn define_all() {
                    selenide_sdk::data::State::define(#field_index + 1);
                }
                #to_bytes
            }
        };
    } else {
        return TokenStream::from(
            syn::Error::new(
                Span::call_site().into(),
                "cesium macro can only be used on struct or func.",
            )
            .to_compile_error(),
        );
    }

    TokenStream::from(expanded)
}
