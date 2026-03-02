# Changelog

## 0.4.0 (2026-03-03)

* Propagate `#[repr(C)]` onto generated structs. ([#11][#11], [#12][#12])
* Update to `syn` `2`. ([#12][#12])

[#11]: https://github.com/azriel91/enum_variant_type/issues/11
[#12]: https://github.com/azriel91/enum_variant_type/pull/12


## 0.3.1 (2021-12-22)

* Import all items from parent scope when generating structs in submodule. ([#9][#9])

[#9]: https://github.com/azriel91/enum_variant_type/pull/9


## 0.3.0 (2021-12-18)

* `#[evt(derive(..))]` on enum adds derives on every variant. ([#6][#6], [#7][#7])
* `#[evt(module = "module1")]` generates structs inside `mod module1`. ([#5][#5], [#7][#7])
* `#[evt(implement_marker_traits(MarkerTrait1))]` on enum generates `impl MarkerTrait1` for all generated structs. ([#7][#7])

[#5]: https://github.com/azriel91/enum_variant_type/issues/5
[#6]: https://github.com/azriel91/enum_variant_type/issues/6
[#7]: https://github.com/azriel91/enum_variant_type/pull/7


## 0.2.1 (2021-04-24)

* `no-std` support by default. ([#2][#2], [#3][#3])

[#2]: https://github.com/azriel91/enum_variant_type/issues/2
[#3]: https://github.com/azriel91/enum_variant_type/pull/3


## 0.2.0 (2020-01-13)

* Allow variants to be skipped using `#[evt(skip)]`.
* ***Breaking:*** `#[evt(..)]` specifies the attributes to attach to the generated type (previously `#[evt_attr(..)]`).


## 0.1.0 (2020-01-10)

* Generates unit, tuple, named struct for each enum variant.
* `impl From<EnumVariantType> for Enum`.
* `impl TryFrom<Enum, Error = Enum> for EnumVariantType`.
* `#[cfg(..)]`, `#[doc = ".."]`, `#[allow(..)]`, `#[deny(..)]` attributes are copied from the variant.
* `#[evt_attr(..)]` specifies the attributes to attach to the generated type.
