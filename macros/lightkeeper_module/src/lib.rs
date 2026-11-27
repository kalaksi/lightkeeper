/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
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
// Monitors and commands also require: access="read" | access="write" | access="delete"
struct ModuleArgs {
    name: String,
    version: String,
    description: String,
    parent_module_name: Option<String>,
    parent_module_version: Option<String>,
    settings: HashMap<String, String>,
    secrets: HashMap<String, String>,
    uses_sudo: Option<bool>,
    access: Option<String>,
}

impl syn::parse::Parse for ModuleArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut version = None;
        let mut description = None;
        let mut parent_module_name = None;
        let mut parent_module_version = None;
        let mut settings = HashMap::new();
        let mut secrets = HashMap::new();
        let mut uses_sudo = None;
        let mut access = None;

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
                "uses_sudo" => {
                    uses_sudo = Some(input.parse::<syn::LitBool>()?.value);
                }
                "access" => {
                    let value = input.parse::<syn::LitStr>()?;
                    let access_value = value.value();
                    if access_value != "read" && access_value != "write" && access_value != "delete" {
                        return Err(syn::Error::new(
                            value.span(),
                            "access must be \"read\", \"write\", or \"delete\"",
                        ));
                    }
                    access = Some(access_value);
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
                "secrets" => {
                    let content;
                    syn::braced!(content in input);

                    while !content.is_empty() {
                        let key: syn::Ident = content.parse()?;
                        content.parse::<syn::Token![=>]>()?;
                        let value: syn::LitStr = content.parse()?;
                        secrets.insert(key.to_string(), value.value());
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
            name: name.expect("Module name is required"),
            version: version.expect("Module version is required"),
            description: description.expect("Module description is required"),
            parent_module_name: parent_module_name,
            parent_module_version: parent_module_version,
            settings: settings,
            secrets: secrets,
            uses_sudo: uses_sudo,
            access: access,
        })
    }
}

macro_rules! module_access_tokens {
    ($access:expr) => {{
        match $access.as_deref() {
            Some("read") => quote! { Some(crate::module::ModuleAccess::Read) },
            Some("write") => quote! { Some(crate::module::ModuleAccess::Write) },
            Some("delete") => quote! { Some(crate::module::ModuleAccess::Delete) },
            Some(_) => unreachable!("access validated during parse"),
            None => {
                return quote! {
                    compile_error!("access is required and must be \"read\", \"write\", or \"delete\"");
                }
                .into();
            }
        }
    }};
}

#[proc_macro_attribute]
pub fn monitoring_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_parsed = syn::parse_macro_input!(args as ModuleArgs);
    let module_name = args_parsed.name;
    let module_version = args_parsed.version;
    let module_description = args_parsed.description;
    let uses_sudo = args_parsed.uses_sudo.unwrap_or(false);
    let access = module_access_tokens!(args_parsed.access);
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
                        secrets: HashMap::new(),
                        parent_module: None,
                        is_stateless: true,
                        uses_sudo: #uses_sudo,
                        access: #access,
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
    let uses_sudo = args_parsed.uses_sudo.unwrap_or(false);
    let access = module_access_tokens!(args_parsed.access);
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
                        secrets: HashMap::new(),
                        parent_module: Some(ModuleSpecification::monitor(#parent_module_name, #parent_module_version)),
                        is_stateless: true,
                        uses_sudo: #uses_sudo,
                        access: #access,
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
    let uses_sudo = args_parsed.uses_sudo.unwrap_or(false);
    let access = module_access_tokens!(args_parsed.access);
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
                    secrets: HashMap::new(),
                    parent_module: None,
                    is_stateless: true,
                    uses_sudo: #uses_sudo,
                    access: #access,
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
/// Stateful connection modules.
pub fn connection_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_parsed = syn::parse_macro_input!(args as ModuleArgs);
    let module_name = args_parsed.name;
    let module_version = args_parsed.version;
    let module_description = args_parsed.description;
    let uses_sudo = args_parsed.uses_sudo.unwrap_or(false);
    let settings = args_parsed.settings.iter().map(|(key, value)| {
        quote! {
            (#key.to_string(), #value.to_string())
        }
    });
    let secrets = args_parsed.secrets.iter().map(|(key, value)| {
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
                    module_spec: ModuleSpecification::connector(#module_name, #module_version),
                    description: String::from(#module_description),
                    settings: HashMap::from([
                        #(#settings),*
                    ]),
                    secrets: HashMap::from([
                        #(#secrets),*
                    ]),
                    parent_module: None,
                    is_stateless: false,
                    uses_sudo: #uses_sudo,
                    access: None,
                }
            }

            fn get_metadata_self(&self) -> Metadata {
                Self::get_metadata()
            }

            fn get_module_spec(&self) -> ModuleSpecification {
                Self::get_metadata().module_spec
            }
        }

        impl BoxCloneableConnector for #struct_name {
            fn box_clone(&self) -> Box<dyn ConnectionModule + Send + Sync> {
                Box::new(self.clone())
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
    let uses_sudo = args_parsed.uses_sudo.unwrap_or(false);
    let settings = args_parsed.settings.iter().map(|(key, value)| {
        quote! {
            (#key.to_string(), #value.to_string())
        }
    });
    let secrets = args_parsed.secrets.iter().map(|(key, value)| {
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
                        module_spec: ModuleSpecification::connector(#module_name, #module_version),
                        description: String::from(#module_description),
                        settings: HashMap::from([
                            #(#settings),*
                        ]),
                        secrets: HashMap::from([
                            #(#secrets),*
                        ]),
                        parent_module: None,
                        is_stateless: true,
                        uses_sudo: #uses_sudo,
                        access: None,
                    }
                }

                fn get_metadata_self(&self) -> Metadata {
                    Self::get_metadata()
                }

                fn get_module_spec(&self) -> ModuleSpecification {
                    Self::get_metadata().module_spec
                }
            }

            impl BoxCloneableConnector for #struct_name {
                fn box_clone(&self) -> Box<dyn ConnectionModule + Send + Sync> {
                    Box::new(self.clone())
                }
            }
        }.into()
    }
    else {
        TokenStream::new()
    }
}
