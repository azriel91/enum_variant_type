#![deny(missing_debug_implementations, missing_docs)] // kcov-ignore
#![recursion_limit = "128"]

//! Provides a proc macro derive to produce structs from enum variants.
//!
//! This is a poor-man's implementation of <https://github.com/rust-lang/rfcs/pull/2593>.
//!
//! # Examples
//!
//! ```rust,edition2018
//! use enum_variant_type::EnumVariantType;
//!
//! #[derive(EnumVariantType)]
//! pub enum MyEnum {
//!     /// Unit variant.
//!     Unit,
//!     /// Tuple variant.
//!     Tuple(u32, u64),
//!     /// Struct variant.
//!     Struct {
//!         field_0: u32,
//!         field_1: u64,
//!     },
//! }
//! ```
//!
//! Generates:
//!
//! ```rust,edition2018
//! use std::convert::TryFrom;
//!
//! # pub enum MyEnum {
//! #     /// Unit variant.
//! #     Unit,
//! #     /// Tuple variant.
//! #     Tuple(u32, u64),
//! #     /// Struct variant.
//! #     Struct {
//! #         field_0: u32,
//! #         field_1: u64,
//! #     },
//! # }
//! #
//! /// Unit variant.
//! #[derive(Debug)]
//! pub struct Unit;
//!
//! /// Tuple variant.
//! #[derive(Debug)]
//! pub struct Tuple(pub u32, pub u64);
//!
//! /// Struct variant.
//! #[derive(Debug)]
//! pub struct Struct {
//!     pub field_0: u32,
//!     pub field_1: u64,
//! }
//!
//! impl From<Unit> for MyEnum {
//!     fn from(enum_variant_type: Unit) -> Self {
//!         MyEnum::Unit
//!     }
//! }
//!
//! impl From<Tuple> for MyEnum {
//!     fn from(enum_variant_type: Tuple) -> Self {
//!         let Tuple(_0, _1) = enum_variant_type;
//!         MyEnum::Tuple(_0, _1)
//!     }
//! }
//!
//! impl From<Struct> for MyEnum {
//!     fn from(enum_variant_type: Struct) -> Self {
//!         let Struct { field_0, field_1 } = enum_variant_type;
//!         MyEnum::Struct { field_0, field_1 }
//!     }
//! }
//!
//! impl TryFrom<MyEnum> for Unit {
//!     type Error = MyEnum;
//!     fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
//!         if let MyEnum::Unit = enum_variant {
//!             Ok(Unit)
//!         } else {
//!             Err(enum_variant)
//!         }
//!     }
//! }
//!
//! impl TryFrom<MyEnum> for Tuple {
//!     type Error = MyEnum;
//!     fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
//!         if let MyEnum::Tuple(_0, _1) = enum_variant {
//!             Ok(Tuple(_0, _1))
//!         } else {
//!             Err(enum_variant)
//!         }
//!     }
//! }
//!
//! impl TryFrom<MyEnum> for Struct {
//!     type Error = MyEnum;
//!     fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
//!         if let MyEnum::Struct { field_0, field_1 } = enum_variant {
//!             Ok(Struct { field_0, field_1 })
//!         } else {
//!             Err(enum_variant)
//!         }
//!     }
//! }
//! ```

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_roids::FieldsExt;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DataEnum, DeriveInput, Field, Fields, Meta};

/// Attributes that should be copied across.
const ATTRIBUTES_TO_COPY: &[&str] = &["doc", "cfg", "allow", "deny"];

/// Derives a struct for each enum variant.
///
/// Struct fields including their attributes are copied over.
#[proc_macro_derive(EnumVariantType, attributes(evt_attrs))]
pub fn enum_variant_type(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let enum_name = &ast.ident;
    let vis = &ast.vis;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let data_enum = data_enum(&ast);
    let variants = &data_enum.variants;

    let mut struct_declarations = proc_macro2::TokenStream::new();
    let struct_declarations_iter = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let attrs_to_copy = variant
            .attrs
            .iter()
            .filter(|attribute| {
                ATTRIBUTES_TO_COPY
                    .iter()
                    .any(|attr_to_copy| attribute.path.is_ident(attr_to_copy))
            })
            .collect::<Vec<&Attribute>>();
        let variant_struct_attrs = variant.attrs.iter().fold(
            proc_macro2::TokenStream::new(),
            |mut attrs_tokens, attribute| {
                if attribute.path.is_ident("evt_attrs") {
                    let variant_struct_attrs = attribute.parse_meta().ok().and_then(|meta| {
                        if let Meta::List(meta_list) = meta {
                            Some(meta_list.nested)
                        } else {
                            None
                        }
                    });
                    if let Some(variant_struct_attrs) = variant_struct_attrs {
                        attrs_tokens.extend(quote!(#[#variant_struct_attrs]));
                    }
                }

                attrs_tokens
            },
        );
        let variant_fields = &variant.fields;

        // Need to attach visibility modifier to fields.
        let fields_with_vis = variant_fields
            .iter()
            .cloned()
            .map(|mut field| {
                field.vis = vis.clone();
                field
            })
            .collect::<Vec<Field>>();

        let data_struct = match variant_fields {
            Fields::Unit => quote! {
                struct #variant_name;
            },
            Fields::Unnamed(..) => {
                quote! {
                    struct #variant_name #ty_generics (#(#fields_with_vis,)*) #where_clause;
                }
            }
            Fields::Named(..) => quote! {
                struct #variant_name #ty_generics #where_clause {
                    #(#fields_with_vis,)*
                }
            },
        };

        // TODO: This generates invalid code if the type parameter is not used by this variant.
        let construction_form = variant_fields.construction_form();
        let deconstruct_variant_struct = if variant_fields.is_unit() {
            proc_macro2::TokenStream::new()
        } else {
            quote! {
                let #variant_name #construction_form = variant_struct;
            }
        };
        let impl_from_variant_for_enum = quote! {
            impl #impl_generics std::convert::From<#variant_name #ty_generics>
                for #enum_name #ty_generics
            #where_clause {
                fn from(variant_struct: #variant_name #ty_generics) -> Self {
                    // Deconstruct the parameter.
                    #deconstruct_variant_struct

                    #enum_name::#variant_name #construction_form
                }
            }
        };

        let impl_try_from_enum_for_variant = quote! {
            impl #impl_generics std::convert::TryFrom<#enum_name #ty_generics>
                for #variant_name #ty_generics
            #where_clause {
                type Error = #enum_name #ty_generics;

                fn try_from(enum_variant: #enum_name #ty_generics) -> Result<Self, Self::Error> {
                    // Deconstruct the variant.
                    if let #enum_name::#variant_name #construction_form = enum_variant {
                        std::result::Result::Ok(#variant_name #construction_form)
                    } else {
                        std::result::Result::Err(enum_variant)
                    }
                }
            }
        };

        quote! {
            #(#attrs_to_copy)*
            #variant_struct_attrs
            #vis #data_struct

            #impl_from_variant_for_enum

            #impl_try_from_enum_for_variant
        }
    });
    struct_declarations.extend(struct_declarations_iter);
    struct_declarations.into()
}

fn data_enum(ast: &DeriveInput) -> &DataEnum {
    if let Data::Enum(data_enum) = &ast.data {
        data_enum
    } else {
        panic!("`EnumVariantType` derive can only be used on an enum.");
    }
}
