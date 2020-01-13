# Changelog

## 0.2.0 (2020-01-13)

* Allow variants to be skipped using `#[evt(skip)]`.
* ***Breaking:*** `#[evt(..)]` specifies the attributes to attach to the generated type (previously `#[evt_attr(..)]`).

## 0.1.0 (2020-01-10)

* Generates unit, tuple, named struct for each enum variant.
* `impl From<EnumVariantType> for Enum`.
* `impl TryFrom<Enum, Error = Enum> for EnumVariantType`.
* `#[cfg(..)]`, `#[doc = ".."]`, `#[allow(..)]`, `#[deny(..)]` attributes are copied from the variant.
* `#[evt_attr(..)]` specifies the attributes to attach to the generated type.
