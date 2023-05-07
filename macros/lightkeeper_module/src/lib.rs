use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::parse::Parser;

#[proc_macro_attribute]
pub fn monitoring_module(args: TokenStream, input: TokenStream) -> TokenStream {
    // TODO: Add compile errors?
    let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
    let args_parsed = parser.parse(args).unwrap();
    let mut args_iter = args_parsed.iter();
    let module_name = args_iter.next().unwrap();
    let module_version = args_iter.next().unwrap();

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
                        // module_spec: ModuleSpecification::new(stringify!(#module_name), stringify!(#module_version)),
                        module_spec: ModuleSpecification::new(#module_name, #module_version),
                        description: String::from(""),
                        url: String::from(""),
                        parent_module: None,
                        is_stateless: true,
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
    // TODO: Add compile errors?
    let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
    let args_parsed = parser.parse(args).unwrap();
    let mut args_iter = args_parsed.iter();
    let module_name = args_iter.next().unwrap();
    let module_version = args_iter.next().unwrap();
    let parent_module_name = args_iter.next().unwrap();
    let parent_module_version = args_iter.next().unwrap();

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
                        // module_spec: ModuleSpecification::new(stringify!(#module_name), stringify!(#module_version)),
                        module_spec: ModuleSpecification::new(#module_name, #module_version),
                        description: String::from(""),
                        url: String::from(""),
                        parent_module: Some(ModuleSpecification::new(#parent_module_name, #parent_module_version)),
                        is_stateless: true,
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
    // TODO: Add compile errors?
    let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
    let args_parsed = parser.parse(args).unwrap();
    let mut args_iter = args_parsed.iter();
    let module_name = args_iter.next().unwrap();
    let module_version = args_iter.next().unwrap();

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
                        // module_spec: ModuleSpecification::new(stringify!(#module_name), stringify!(#module_version)),
                        module_spec: ModuleSpecification::new(#module_name, #module_version),
                        description: String::from(""),
                        url: String::from(""),
                        parent_module: None,
                        is_stateless: true,
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
    else {
        TokenStream::new()
    }
}

#[proc_macro_attribute]
pub fn connection_module(args: TokenStream, input: TokenStream) -> TokenStream {
    // TODO: Add compile errors?
    let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
    let args_parsed = parser.parse(args).unwrap();
    let mut args_iter = args_parsed.iter();
    let module_name = args_iter.next().unwrap();
    let module_version = args_iter.next().unwrap();

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
                        // module_spec: ModuleSpecification::new(stringify!(#module_name), stringify!(#module_version)),
                        module_spec: ModuleSpecification::new(#module_name, #module_version),
                        description: String::from(""),
                        url: String::from(""),
                        parent_module: None,
                        is_stateless: false,
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

#[proc_macro_attribute]
pub fn stateless_connection_module(args: TokenStream, input: TokenStream) -> TokenStream {
    // TODO: Add compile errors?
    let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
    let args_parsed = parser.parse(args).unwrap();
    let mut args_iter = args_parsed.iter();
    let module_name = args_iter.next().unwrap();
    let module_version = args_iter.next().unwrap();

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
                        // module_spec: ModuleSpecification::new(stringify!(#module_name), stringify!(#module_version)),
                        module_spec: ModuleSpecification::new(#module_name, #module_version),
                        description: String::from(""),
                        url: String::from(""),
                        parent_module: None,
                        is_stateless: true,
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