/// A trait for assigning a type a default value in the const context.
/// This trait is used by `CappedLengthList8` to provide an uninitialized
/// value without unsafe code.
pub trait ConstDefault: Copy + Sized {
    const DEFAULT: Self;
}

macro_rules! const_default_zero_impls {
    ($($type: ident),+ $(,)?) => {
        $(
            impl $crate::data_structures::const_default::ConstDefault for $type {
                const DEFAULT: Self = 0;
            }
        )+
    };
}

const_default_zero_impls!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize,);

impl ConstDefault for () {
    const DEFAULT: Self = ();
}

impl ConstDefault for bool {
    const DEFAULT: Self = false;
}

impl ConstDefault for char {
    const DEFAULT: Self = '\0';
}

impl ConstDefault for f32 {
    const DEFAULT: Self = 0.0;
}

impl ConstDefault for f64 {
    const DEFAULT: Self = 0.0;
}

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
