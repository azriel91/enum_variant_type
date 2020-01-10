[![Crates.io](https://img.shields.io/crates/v/enum_variant_type.svg)](https://crates.io/crates/enum_variant_type)
[![Build Status](https://ci.appveyor.com/api/projects/status/github/azriel91/enum_variant_type?branch=master&svg=true)](https://ci.appveyor.com/project/azriel91/enum_variant_type/branch/master)
[![Build Status](https://travis-ci.org/azriel91/enum_variant_type.svg?branch=master)](https://travis-ci.org/azriel91/enum_variant_type)
[![Coverage Status](https://codecov.io/gh/azriel91/enum_variant_type/branch/master/graph/badge.svg)](https://codecov.io/gh/azriel91/enum_variant_type)

# Enum Variant Type

Provides a proc macro derive to produce structs from enum variants.

This is a poor-man's implementation of <https://github.com/rust-lang/rfcs/pull/2593>.

## Examples

```rust,edition2018
use enum_variant_type::EnumVariantType;

#[derive(EnumVariantType)]
pub enum MyEnum {
    /// Unit variant.
    Unit,
    /// Tuple variant.
    Tuple(u32, u64),
    /// Struct variant.
    Struct {
        field_0: u32,
        field_1: u64,
    },
}
```

Generates:

```rust,edition2018
use std::convert::TryFrom;

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
/// Unit variant.
#[derive(Debug)]
pub struct Unit;

/// Tuple variant.
#[derive(Debug)]
pub struct Tuple(pub u32, pub u64);

/// Struct variant.
#[derive(Debug)]
pub struct Struct {
    pub field_0: u32,
    pub field_1: u64,
}

impl From<Unit> for MyEnum {
    fn from(enum_variant_type: Unit) -> Self {
        MyEnum::Unit
    }
}

impl From<Tuple> for MyEnum {
    fn from(enum_variant_type: Tuple) -> Self {
        let Tuple(_0, _1) = enum_variant_type;
        MyEnum::Tuple(_0, _1)
    }
}

impl From<Struct> for MyEnum {
    fn from(enum_variant_type: Struct) -> Self {
        let Struct { field_0, field_1 } = enum_variant_type;
        MyEnum::Struct { field_0, field_1 }
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
```

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
