#![deny(missing_debug_implementations, missing_docs)] // kcov-ignore
#![no_std]
#![recursion_limit = "128"]

//! Proc macro derive to generate structs from enum variants.
//!
//! This is a poor-man's implementation of <https://github.com/rust-lang/rfcs/pull/2593>.
//!
//! ```toml
//! [dependencies]
//! enum_variant_type = "0.2.0"
//! ```
//!
//! # Examples
//!
//! ```rust,edition2018
//! use enum_variant_type::EnumVariantType;
//!
//! #[derive(Debug, EnumVariantType, PartialEq)]
//! pub enum MyEnum {
//!     /// Unit variant.
//!     #[evt(derive(Clone, Copy, Debug, PartialEq))]
//!     Unit,
//!     /// Tuple variant.
//!     #[evt(derive(Debug, PartialEq))]
//!     Tuple(u32, u64),
//!     /// Struct variant.
//!     #[evt(derive(Debug))]
//!     Struct {
//!         field_0: u32,
//!         field_1: u64,
//!     },
//!     /// Skipped variant.
//!     #[evt(skip)]
//!     Skipped,
//! }
//!
//! // Now you can do the following:
//! use core::convert::TryFrom;
//! let unit: Unit = Unit::try_from(MyEnum::Unit).unwrap();
//! let tuple: Tuple = Tuple::try_from(MyEnum::Tuple(12, 34)).unwrap();
//! let named: Struct = Struct::try_from(MyEnum::Struct { field_0: 12, field_1: 34 }).unwrap();
//!
//! let enum_unit = MyEnum::from(unit);
//! let enum_tuple = MyEnum::from(tuple);
//! let enum_struct = MyEnum::from(named);
//!
//! // If the enum variant doesn't match the variant type, then the original variant is returned in
//! // the `Result`'s `Err` variant.
//! assert_eq!(Err(MyEnum::Unit), Tuple::try_from(MyEnum::Unit));
//! ```
//!
//! ### Additional options specified by a evt attribute on enum:
//! - `#[evt(derive(Clone, Copy))]`: derives `Clone`, `Copy` on **every** variant
//! - `#[evt(module = "module1")]`: all generated variants are put into `mod module1 { ... }`
//! - `#[evt(implement_marker_traits(MarkerTrait1))]`: all generated variants are implemented over `MarkerTrait1`
//!
//! <details>
//!
//! <summary>Generated code</summary>
//!
//! ```rust,edition2018
//! use core::convert::TryFrom;
//!
//! /// Unit variant.
//! #[derive(Clone, Copy, Debug, PartialEq)]
//! pub struct Unit;
//!
//! /// Tuple variant.
//! #[derive(Debug, PartialEq)]
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
//!     fn from(variant_struct: Unit) -> Self {
//!         MyEnum::Unit
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
//! impl From<Tuple> for MyEnum {
//!     fn from(variant_struct: Tuple) -> Self {
//!         let Tuple(_0, _1) = variant_struct;
//!         MyEnum::Tuple(_0, _1)
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
//! impl From<Struct> for MyEnum {
//!     fn from(variant_struct: Struct) -> Self {
//!         let Struct { field_0, field_1 } = variant_struct;
//!         MyEnum::Struct { field_0, field_1 }
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
//! ```
//!
//! </details>

extern crate alloc;
extern crate proc_macro;

use alloc::vec::Vec;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_roids::{namespace_parameters, FieldsExt};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DataEnum, DeriveInput, Field, Fields, Lit,
    Meta, NestedMeta, Path,
};

/// Attributes that should be copied across.
const ATTRIBUTES_TO_COPY: &[&str] = &["doc", "cfg", "allow", "deny"];

/// Derives a struct for each enum variant.
///
/// Struct fields including their attributes are copied over.
#[proc_macro_derive(EnumVariantType, attributes(evt))]
pub fn enum_variant_type(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // Need to do this, otherwise we can't unit test the input.
    enum_variant_type_impl(ast).into()
}

#[inline]
fn enum_variant_type_impl(ast: DeriveInput) -> proc_macro2::TokenStream {
    let enum_name = &ast.ident;
    let vis = &ast.vis;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let data_enum = data_enum(&ast);
    let variants = &data_enum.variants;

    let mut wrap_in_module = None::<Ident>;
    let mut derive_for_all_variants = None::<Attribute>;
    let mut marker_trait_paths = Vec::<Path>::new();

    for attr in ast.attrs.iter() {
        if attr.path.is_ident("evt") {
            if let Ok(Meta::List(list)) = attr.parse_meta() {
                for item in list.nested.iter() {
                    match item {
                        NestedMeta::Meta(Meta::NameValue(name_value)) => {
                            if let (true, Lit::Str(lit_str)) =
                                (name_value.path.is_ident("module"), &name_value.lit)
                            {
                                wrap_in_module =
                                    Some(Ident::new(&lit_str.value(), Span::call_site()));
                            } else {
                                panic!("Expected evt attribute argument of form #[evt(module = \"some_module_name\")]");
                            }
                        }
                        NestedMeta::Meta(Meta::List(list)) => {
                            if list.path.is_ident("derive") {
                                let items = list.nested.iter().map(|nested_meta| {
                                    if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
                                        path.clone()
                                    } else {
                                        panic!("Expected evt attribute argument of form #[evt(derive(Clone, Debug))]");
                                    }
                                });
                                derive_for_all_variants = Some(parse_quote! {
                                    #[derive( #(#items),* )]
                                });
                            } else if list.path.is_ident("implement_marker_traits") {
                                marker_trait_paths = list.nested
                                    .iter()
                                    .map(|nested| if let NestedMeta::Meta(Meta::Path(path)) = nested {
                                        path.clone()
                                } else {
                                    panic!("Expected evt attribute argument of form #[evt(implement_marker_traits(MarkerTrait1, MarkerTrait2))]");
                                }).collect();
                            }
                        }
                        _ => panic!("Incorrect usage of evt attribute, see README.md"),
                    }
                }
            } else {
                panic!("Invalid evt attr")
            }
        }
    }

    let mut struct_declarations = proc_macro2::TokenStream::new();

    let ns: Path = parse_quote!(evt);
    let skip: Path = parse_quote!(skip);
    let struct_declarations_iter = variants.iter()
        .filter(|variant| !proc_macro_roids::contains_tag(&variant.attrs,  &ns, &skip))
        .map(|variant| {

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

        let evt_meta_lists = namespace_parameters(&variant.attrs, &ns);
        let variant_struct_attrs = evt_meta_lists
            .into_iter()
            .fold(
                proc_macro2::TokenStream::new(),
                |mut attrs_tokens, variant_struct_attr| {
                    attrs_tokens.extend(quote!(#[#variant_struct_attr]));
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
            impl #impl_generics core::convert::From<#variant_name #ty_generics>
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
            impl #impl_generics core::convert::TryFrom<#enum_name #ty_generics>
                for #variant_name #ty_generics
            #where_clause {
                type Error = #enum_name #ty_generics;

                fn try_from(enum_variant: #enum_name #ty_generics) -> Result<Self, Self::Error> {
                    // Deconstruct the variant.
                    if let #enum_name::#variant_name #construction_form = enum_variant {
                        core::result::Result::Ok(#variant_name #construction_form)
                    } else {
                        core::result::Result::Err(enum_variant)
                    }
                }
            }
        };

        quote! {
            #(#attrs_to_copy)*
            #derive_for_all_variants
            #variant_struct_attrs
            #vis #data_struct

            #impl_from_variant_for_enum

            #impl_try_from_enum_for_variant

            #(impl #ty_generics #marker_trait_paths for #variant_name #ty_generics {})*
        }
    });
    struct_declarations.extend(struct_declarations_iter);

    if let Some(module_to_wrap_in) = wrap_in_module {
        quote! {
            #vis mod #module_to_wrap_in {
                use super::#enum_name;

                #struct_declarations
            }
        }
    } else {
        struct_declarations
    }
}

fn data_enum(ast: &DeriveInput) -> &DataEnum {
    if let Data::Enum(data_enum) = &ast.data {
        data_enum
    } else {
        panic!("`EnumVariantType` derive can only be used on an enum."); // kcov-ignore
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::string::ToString;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::{parse_quote, DeriveInput};

    use super::enum_variant_type_impl;

    #[test]
    fn generates_correct_tokens_for_basic_enum() {
        let ast: DeriveInput = parse_quote! {
            pub enum MyEnum {
                /// Unit variant.
                #[evt(derive(Clone, Copy, Debug, PartialEq))]
                Unit,
                /// Tuple variant.
                #[evt(derive(Debug))]
                Tuple(u32, u64),
                /// Struct variant.
                Struct {
                    field_0: u32,
                    field_1: u64,
                },
            }
        };

        let actual_tokens = enum_variant_type_impl(ast);
        let expected_tokens = quote! {
            /// Unit variant.
            #[derive(Clone, Copy, Debug, PartialEq)]
            pub struct Unit;

            impl core::convert::From<Unit> for MyEnum {
                fn from(variant_struct: Unit) -> Self {
                    MyEnum::Unit
                }
            }

            impl core::convert::TryFrom<MyEnum> for Unit {
                type Error = MyEnum;
                fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                    if let MyEnum::Unit = enum_variant {
                        core::result::Result::Ok(Unit)
                    } else {
                        core::result::Result::Err(enum_variant)
                    }
                }
            }

            /// Tuple variant.
            #[derive(Debug)]
            pub struct Tuple(pub u32, pub u64,);

            impl core::convert::From<Tuple> for MyEnum {
                fn from(variant_struct: Tuple) -> Self {
                    let Tuple(_0, _1,) = variant_struct;
                    MyEnum::Tuple(_0, _1,)
                }
            }

            impl core::convert::TryFrom<MyEnum> for Tuple {
                type Error = MyEnum;
                fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                    if let MyEnum::Tuple(_0, _1,) = enum_variant {
                        core::result::Result::Ok(Tuple(_0, _1,))
                    } else {
                        core::result::Result::Err(enum_variant)
                    }
                }
            }

            /// Struct variant.
            pub struct Struct {
                pub field_0: u32,
                pub field_1: u64,
            }

            impl core::convert::From<Struct> for MyEnum {
                fn from(variant_struct: Struct) -> Self {
                    let Struct { field_0, field_1, } = variant_struct;
                    MyEnum::Struct { field_0, field_1, }
                }
            }

            impl core::convert::TryFrom<MyEnum> for Struct {
                type Error = MyEnum;
                fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                    if let MyEnum::Struct { field_0, field_1, } = enum_variant {
                        core::result::Result::Ok(Struct { field_0, field_1, })
                    } else {
                        core::result::Result::Err(enum_variant)
                    }
                }
            }
        };

        assert_eq!(expected_tokens.to_string(), actual_tokens.to_string());
    }

    #[test]
    fn skips_variants_marked_with_evt_skip() {
        let ast: DeriveInput = parse_quote! {
            pub enum MyEnum {
                /// Unit variant.
                #[evt(derive(Clone, Copy, Debug, PartialEq))]
                Unit,
                /// Skipped variant.
                #[evt(skip)]
                UnitSkipped,
            }
        };

        let actual_tokens = enum_variant_type_impl(ast);
        let expected_tokens = quote! {
            /// Unit variant.
            #[derive(Clone, Copy, Debug, PartialEq)]
            pub struct Unit;

            impl core::convert::From<Unit> for MyEnum {
                fn from(variant_struct: Unit) -> Self {
                    MyEnum::Unit
                }
            }

            impl core::convert::TryFrom<MyEnum> for Unit {
                type Error = MyEnum;
                fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                    if let MyEnum::Unit = enum_variant {
                        core::result::Result::Ok(Unit)
                    } else {
                        core::result::Result::Err(enum_variant)
                    }
                }
            }
        };

        assert_eq!(expected_tokens.to_string(), actual_tokens.to_string());
    }

    #[test]
    fn put_variants_in_module() {
        let ast: DeriveInput = parse_quote! {
            #[evt(module = "example")]
            pub enum MyEnum {
                A,
                B
            }
        };

        let actual_tokens = enum_variant_type_impl(ast);
        let expected_tokens = quote! {
            pub mod example {
                use super::MyEnum;

                pub struct A;

                impl core::convert::From<A> for MyEnum {
                    fn from(variant_struct: A) -> Self {
                        MyEnum::A
                    }
                }

                impl core::convert::TryFrom<MyEnum> for A {
                    type Error = MyEnum;
                    fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                        if let MyEnum::A = enum_variant {
                            core::result::Result::Ok(A)
                        } else {
                            core::result::Result::Err(enum_variant)
                        }
                    }
                }

                pub struct B;

                impl core::convert::From<B> for MyEnum {
                    fn from(variant_struct: B) -> Self {
                        MyEnum::B
                    }
                }

                impl core::convert::TryFrom<MyEnum> for B {
                    type Error = MyEnum;
                    fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                        if let MyEnum::B = enum_variant {
                            core::result::Result::Ok(B)
                        } else {
                            core::result::Result::Err(enum_variant)
                        }
                    }
                }
            }
        };

        assert_eq!(expected_tokens.to_string(), actual_tokens.to_string());
    }

    #[test]
    fn derive_traits_for_all_variants() {
        let ast: DeriveInput = parse_quote! {
            #[evt(derive(Debug))]
            pub enum MyEnum {
                A,
                #[evt(derive(Clone))]
                B
            }
        };

        let actual_tokens = enum_variant_type_impl(ast);
        let expected_tokens = quote! {
            #[derive(Debug)]
            pub struct A;

            impl core::convert::From<A> for MyEnum {
                fn from(variant_struct: A) -> Self {
                    MyEnum::A
                }
            }

            impl core::convert::TryFrom<MyEnum> for A {
                type Error = MyEnum;
                fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                    if let MyEnum::A = enum_variant {
                        core::result::Result::Ok(A)
                    } else {
                        core::result::Result::Err(enum_variant)
                    }
                }
            }

            #[derive(Debug)]
            #[derive(Clone)]
            pub struct B;

            impl core::convert::From<B> for MyEnum {
                fn from(variant_struct: B) -> Self {
                    MyEnum::B
                }
            }

            impl core::convert::TryFrom<MyEnum> for B {
                type Error = MyEnum;
                fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                    if let MyEnum::B = enum_variant {
                        core::result::Result::Ok(B)
                    } else {
                        core::result::Result::Err(enum_variant)
                    }
                }
            }
        };

        assert_eq!(expected_tokens.to_string(), actual_tokens.to_string());
    }

    #[test]
    fn derive_marker_trait() {
        let ast: DeriveInput = parse_quote! {
            #[evt(implement_marker_traits(MarkerTrait1))]
            pub enum MyEnum {
                A,
                B
            }
        };

        let actual_tokens = enum_variant_type_impl(ast);
        let expected_tokens = quote! {
            pub struct A;

            impl core::convert::From<A> for MyEnum {
                fn from(variant_struct: A) -> Self {
                    MyEnum::A
                }
            }

            impl core::convert::TryFrom<MyEnum> for A {
                type Error = MyEnum;
                fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                    if let MyEnum::A = enum_variant {
                        core::result::Result::Ok(A)
                    } else {
                        core::result::Result::Err(enum_variant)
                    }
                }
            }

            impl MarkerTrait1 for A {}

            pub struct B;

            impl core::convert::From<B> for MyEnum {
                fn from(variant_struct: B) -> Self {
                    MyEnum::B
                }
            }

            impl core::convert::TryFrom<MyEnum> for B {
                type Error = MyEnum;
                fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
                    if let MyEnum::B = enum_variant {
                        core::result::Result::Ok(B)
                    } else {
                        core::result::Result::Err(enum_variant)
                    }
                }
            }

            impl MarkerTrait1 for B {}
        };

        assert_eq!(expected_tokens.to_string(), actual_tokens.to_string());
    }
}
