/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn;

// ModuleArgs contain the parsing logic. Macro parameters should look like this:
// #[connection_module(
//     name="name",
//     version="1.0",
//     description="description",
//     settings={
//         parameter1_key => "parameter1_description",
//         parameter2_key => "parameter2_description"
//     }
// )]
struct ModuleArgs {
    name: String,
    version: String,
    description: String,
    parent_module_name: Option<String>,
    parent_module_version: Option<String>,
    cache_scope: String,
    settings: HashMap<String, String>,
}

impl syn::parse::Parse for ModuleArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut version = None;
        let mut description = None;
        let mut parent_module_name = None;
        let mut parent_module_version = None;
        let mut cache_scope = String::from("Host");
        let mut settings = HashMap::new();

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            match key.to_string().as_str() {
                "name" => {
                    name = Some(input.parse::<syn::LitStr>()?.value());
                }
                "version" => {
                    version = Some(input.parse::<syn::LitStr>()?.value());
                }
                "description" => {
                    description = Some(input.parse::<syn::LitStr>()?.value());
                }
                "parent_module_name" => {
                    parent_module_name = Some(input.parse::<syn::LitStr>()?.value());
                }
                "parent_module_version" => {
                    parent_module_version = Some(input.parse::<syn::LitStr>()?.value());
                }
                "cache_scope" => {
                    cache_scope = input.parse::<syn::LitStr>()?.value();
                }
                "settings" => {
                    let content;
                    syn::braced!(content in input);

                    while !content.is_empty() {
                        let key: syn::Ident = content.parse()?;
                        content.parse::<syn::Token![=>]>()?;
                        let value: syn::LitStr = content.parse()?;
                        settings.insert(key.to_string(), value.value());
                        if !content.is_empty() {
                            content.parse::<syn::Token![,]>()?;
                        }
                    }
                },
                _ => return Err(syn::Error::new(key.span(), format!("Unknown key: {}", key))),
            }
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(ModuleArgs {
            name: name.unwrap(),
            version: version.unwrap(),
            description: description.unwrap(),
            parent_module_name: parent_module_name,
            parent_module_version: parent_module_version,
            cache_scope: cache_scope,
            settings: settings,
        })
    }
}


#[proc_macro_attribute]
pub fn monitoring_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_parsed = syn::parse_macro_input!(args as ModuleArgs);
    let module_name = args_parsed.name;
    let module_version = args_parsed.version;
    let module_description = args_parsed.description;
    let settings = args_parsed.settings.iter().map(|(key, value)| {
        quote! {
            (#key.to_string(), #value.to_string())
        }
    });

    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let original = ast.clone();
    let struct_name = &ast.ident;

    // Works only for structs.
    if let syn::Data::Struct(_data) = ast.data {
        quote! {
            #[derive(Clone)]
            #original

            impl MetadataSupport for #struct_name {
                fn get_metadata() -> Metadata {
                    Metadata {
                        module_spec: ModuleSpecification::monitor(#module_name, #module_version),
                        description: String::from(#module_description),
                        settings: HashMap::from([
                            #(#settings),*
                        ]),
                        parent_module: None,
                        is_stateless: true,
                        cache_scope: crate::cache::CacheScope::Host,
                    }
                }

                fn get_metadata_self(&self) -> Metadata {
                    Self::get_metadata()
                }

                fn get_module_spec(&self) -> ModuleSpecification {
                    Self::get_metadata().module_spec
                }
            }

            impl BoxCloneableMonitor for #struct_name {
                fn box_clone(&self) -> Box<dyn MonitoringModule + Send + Sync> {
                    Box::new(self.clone())
                }
            }
        }.into()
    }
    else {
        TokenStream::new()
    }
}

/// Extension modules enrich or modify the original data and are processed after parent module.
#[proc_macro_attribute]
pub fn monitoring_extension_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_parsed = syn::parse_macro_input!(args as ModuleArgs);
    let module_name = args_parsed.name;
    let module_version = args_parsed.version;
    let module_description = args_parsed.description;
    let parent_module_name = args_parsed.parent_module_name;
    let parent_module_version = args_parsed.parent_module_version;
    let settings = args_parsed.settings.iter().map(|(key, value)| {
        quote! {
            (#key.to_string(), #value.to_string())
        }
    });

    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let original = ast.clone();
    let struct_name = &ast.ident;

    // Works only for structs.
    if let syn::Data::Struct(_data) = ast.data {
        quote! {
            #[derive(Clone)]
            #original

            impl MetadataSupport for #struct_name {
                fn get_metadata() -> Metadata {
                    Metadata {
                        module_spec: ModuleSpecification::monitor(#module_name, #module_version),
                        description: String::from(#module_description),
                        settings: HashMap::from([
                            #(#settings),*
                        ]),
                        parent_module: Some(ModuleSpecification::monitor(#parent_module_name, #parent_module_version)),
                        is_stateless: true,
                        cache_scope: crate::cache::CacheScope::Host,
                    }
                }

                fn get_metadata_self(&self) -> Metadata {
                    Self::get_metadata()
                }

                fn get_module_spec(&self) -> ModuleSpecification {
                    Self::get_metadata().module_spec
                }
            }

             impl BoxCloneableMonitor for #struct_name {
                 fn box_clone(&self) -> Box<dyn MonitoringModule + Send + Sync> {
                     Box::new(self.clone())
                 }
            }
        }.into()
    }
    else {
        TokenStream::new()
    }
}

#[proc_macro_attribute]
pub fn command_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_parsed = syn::parse_macro_input!(args as ModuleArgs);
    let module_name = args_parsed.name;
    let module_version = args_parsed.version;
    let module_description = args_parsed.description;
    let settings = args_parsed.settings.iter().map(|(key, value)| {
        quote! {
            (#key.to_string(), #value.to_string())
        }
    });

    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let original = ast.clone();
    let struct_name = &ast.ident;

    quote! {
        #[derive(Clone)]
        #original

        impl MetadataSupport for #struct_name {
            fn get_metadata() -> Metadata {
                Metadata {
                    module_spec: ModuleSpecification::command(#module_name, #module_version),
                    description: String::from(#module_description),
                    settings: HashMap::from([
                        #(#settings),*
                    ]),
                    parent_module: None,
                    is_stateless: true,
                    cache_scope: crate::cache::CacheScope::Host,
                }
            }

            fn get_metadata_self(&self) -> Metadata {
                Self::get_metadata()
            }

            fn get_module_spec(&self) -> ModuleSpecification {
                Self::get_metadata().module_spec
            }
        }

        impl BoxCloneableCommand for #struct_name {
            fn box_clone(&self) -> Box<dyn CommandModule + Send + Sync> {
                Box::new(self.clone())
            }
        }
    }.into()

}

#[proc_macro_attribute]
pub fn connection_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_parsed = syn::parse_macro_input!(args as ModuleArgs);
    let module_name = args_parsed.name;
    let module_version = args_parsed.version;
    let module_description = args_parsed.description;
    let settings = args_parsed.settings.iter().map(|(key, value)| {
        quote! {
            (#key.to_string(), #value.to_string())
        }
    });

    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let original = ast.clone();
    let struct_name = &ast.ident;

    quote! {
        #original

        impl MetadataSupport for #struct_name {
            fn get_metadata() -> Metadata {
                Metadata {
                    module_spec: ModuleSpecification::connector(#module_name, #module_version),
                    description: String::from(#module_description),
                    settings: HashMap::from([
                        #(#settings),*
                    ]),
                    parent_module: None,
                    is_stateless: false,
                    cache_scope: crate::cache::CacheScope::Host,
                }
            }

            fn get_metadata_self(&self) -> Metadata {
                Self::get_metadata()
            }

            fn get_module_spec(&self) -> ModuleSpecification {
                Self::get_metadata().module_spec
            }
        }
    }.into()
}


#[proc_macro_attribute]
pub fn stateless_connection_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_parsed = syn::parse_macro_input!(args as ModuleArgs);
    let module_name = args_parsed.name;
    let module_version = args_parsed.version;
    let module_description = args_parsed.description;
    let cache_scope = args_parsed.cache_scope;
    let settings = args_parsed.settings.iter().map(|(key, value)| {
        quote! {
            (#key.to_string(), #value.to_string())
        }
    });

    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let original = ast.clone();
    let struct_name = &ast.ident;

    // Works only for structs.
    if let syn::Data::Struct(_data) = ast.data {
        quote! {
            #original

            impl MetadataSupport for #struct_name {
                fn get_metadata() -> Metadata {
                    Metadata {
                        module_spec: ModuleSpecification::connector(#module_name, #module_version),
                        description: String::from(#module_description),
                        settings: HashMap::from([
                            #(#settings),*
                        ]),
                        parent_module: None,
                        is_stateless: true,
                        cache_scope: #cache_scope.parse::<crate::cache::CacheScope>().unwrap(),
                    }
                }

                fn get_metadata_self(&self) -> Metadata {
                    Self::get_metadata()
                }

                fn get_module_spec(&self) -> ModuleSpecification {
                    Self::get_metadata().module_spec
                }
            }
        }.into()
    }
    else {
        TokenStream::new()
    }
}