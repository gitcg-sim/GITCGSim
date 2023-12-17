pub use constdefault::ConstDefault;

/// Create a `ConstDefault` implementation based on a particular enum case.
/// It's not possible to blanket-implement `ConstDefault` for all `enum_map::Enum`
/// due to the trait not providing a const value.
#[macro_export]
macro_rules! const_default_enum_impl {
    ($type: ident, $case: ident) => {
        impl $crate::data_structures::const_default::ConstDefault for $type {
            const DEFAULT: Self = $type::$case;
        }
    };
}
