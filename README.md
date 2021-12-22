[![Crates.io](https://img.shields.io/crates/v/enum_variant_type.svg)](https://crates.io/crates/enum_variant_type)
[![docs.rs](https://img.shields.io/docsrs/enum_variant_type)](https://docs.rs/enum_variant_type)
[![CI](https://github.com/azriel91/enum_variant_type/workflows/CI/badge.svg)](https://github.com/azriel91/enum_variant_type/actions/workflows/ci.yml)
[![Coverage Status](https://codecov.io/gh/azriel91/enum_variant_type/branch/main/graph/badge.svg)](https://codecov.io/gh/azriel91/enum_variant_type)

# Enum Variant Type

Proc macro derive to generate structs from enum variants.

This is a poor-man's implementation of <https://github.com/rust-lang/rfcs/pull/2593>.

```toml
[dependencies]
enum_variant_type = "0.3.1"
```

## Examples

```rust,edition2018
use enum_variant_type::EnumVariantType;

#[derive(Debug, EnumVariantType, PartialEq)]
pub enum MyEnum {
    /// Unit variant.
    #[evt(derive(Clone, Copy, Debug, PartialEq))]
    Unit,
    /// Tuple variant.
    #[evt(derive(Debug, PartialEq))]
    Tuple(u32, u64),
    /// Struct variant.
    #[evt(derive(Debug))]
    Struct { field_0: u32, field_1: u64 },
    /// Skipped variant.
    #[evt(skip)]
    Skipped,
}

// Now you can do the following:
use core::convert::TryFrom;
let unit: Unit = Unit::try_from(MyEnum::Unit).unwrap();
let tuple: Tuple = Tuple::try_from(MyEnum::Tuple(12, 34)).unwrap();
let named: Struct = Struct::try_from(MyEnum::Struct {
    field_0: 12,
    field_1: 34,
})
.unwrap();

let enum_unit = MyEnum::from(unit);
let enum_tuple = MyEnum::from(tuple);
let enum_struct = MyEnum::from(named);

// If the enum variant doesn't match the variant type, then the original variant is returned in
// the `Result`'s `Err` variant.
assert_eq!(Err(MyEnum::Unit), Tuple::try_from(MyEnum::Unit));
```

<details>

<summary>Generated code</summary>

```rust,edition2018
use core::convert::TryFrom;

/// Unit variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unit;

/// Tuple variant.
#[derive(Debug, PartialEq)]
pub struct Tuple(pub u32, pub u64);

/// Struct variant.
#[derive(Debug)]
pub struct Struct {
    pub field_0: u32,
    pub field_1: u64,
}

impl From<Unit> for MyEnum {
    fn from(variant_struct: Unit) -> Self {
        MyEnum::Unit
    }
}

impl TryFrom<MyEnum> for Unit {
    type Error = MyEnum;
    fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
        if let MyEnum::Unit = enum_variant {
            Ok(Unit)
        } else {
            Err(enum_variant)
        }
    }
}

impl From<Tuple> for MyEnum {
    fn from(variant_struct: Tuple) -> Self {
        let Tuple(_0, _1) = variant_struct;
        MyEnum::Tuple(_0, _1)
    }
}

impl TryFrom<MyEnum> for Tuple {
    type Error = MyEnum;
    fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
        if let MyEnum::Tuple(_0, _1) = enum_variant {
            Ok(Tuple(_0, _1))
        } else {
            Err(enum_variant)
        }
    }
}

impl From<Struct> for MyEnum {
    fn from(variant_struct: Struct) -> Self {
        let Struct { field_0, field_1 } = variant_struct;
        MyEnum::Struct { field_0, field_1 }
    }
}

impl TryFrom<MyEnum> for Struct {
    type Error = MyEnum;
    fn try_from(enum_variant: MyEnum) -> Result<Self, Self::Error> {
        if let MyEnum::Struct { field_0, field_1 } = enum_variant {
            Ok(Struct { field_0, field_1 })
        } else {
            Err(enum_variant)
        }
    }
}

# pub enum MyEnum {
#     /// Unit variant.
#     Unit,
#     /// Tuple variant.
#     Tuple(u32, u64),
#     /// Struct variant.
#     Struct {
#         field_0: u32,
#         field_1: u64,
#     },
# }
#
```

</details>

#### Additional options specified by an `evt` attribute on enum:

* `#[evt(derive(Clone, Copy))]`: Derives `Clone`, `Copy` on **every** variant.
* `#[evt(module = "module1")]`: Generated structs are placed into `mod module1 { ... }`.
* `#[evt(implement_marker_traits(MarkerTrait1))]`: Generated structs all `impl MarkerTrait1`.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
