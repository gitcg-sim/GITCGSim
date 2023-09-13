/// A fixed-size struct that can be accessed like a slice of a particular element type and length.
///
/// # Safety
/// Can only be implemented for structs with all fields of SAME element type (`Self::Elem`) or AsSlice of SAME type.
pub unsafe trait AsSlice: Sized + Copy {
    type Elem: Sized + Copy;
    type Slice;
    const SIZE: usize = std::mem::size_of::<Self>();
    const LENGTH: usize = Self::SIZE / std::mem::size_of::<Self::Elem>();

    fn as_slice(self) -> Self::Slice;
    fn as_slice_ref(&self) -> &Self::Slice;
    fn as_slice_mut(&mut self) -> &mut Self::Slice;
    fn from_slice(slice: Self::Slice) -> Self;
}

#[macro_export]
macro_rules! impl_as_slice {
    ($type: ty, $elem: ty) => {
        unsafe impl $crate::training::as_slice::AsSlice for $type {
            type Elem = f32;
            type Slice = [f32; std::mem::size_of::<$type>() / std::mem::size_of::<$elem>()];

            fn as_slice(self) -> Self::Slice {
                unsafe { std::mem::transmute(self) }
            }

            fn as_slice_ref(&self) -> &Self::Slice {
                unsafe { std::mem::transmute(self) }
            }

            fn as_slice_mut(&mut self) -> &mut Self::Slice {
                unsafe { std::mem::transmute(self) }
            }

            fn from_slice(slice: Self::Slice) -> Self {
                unsafe { std::mem::transmute(slice) }
            }
        }
    };
}
