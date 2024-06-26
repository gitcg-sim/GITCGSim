#[macro_export]
#[doc(hidden)]
macro_rules! impl_from_to_builder {
    ($Type: ty, $Builder: ty) => {
        impl From<$Builder> for $Type {
            #[inline]
            fn from(value: $Builder) -> Self {
                value.build()
            }
        }

        impl From<$Type> for $Builder {
            #[inline]
            fn from(value: $Type) -> Self {
                value.into_builder()
            }
        }
    };
}

/// Creates a struct type with updater methods for each field named after the fields themselves.
/// `field_name: T --> fn field_name(mut self, new_value: T) -> Self {...}`
#[macro_export]
#[doc(hidden)]
macro_rules! with_updaters {
    (
        $(#[$attr: meta])* $vis: vis struct $Type: ident
        {
            $($(#[$field_attr: meta])* $field_vis: vis $field_name: ident : $field_type: ty),*
            $(,)?
        }
    ) => {
        $(#[$attr])* $vis struct $Type {
            $(
                $(#[$field_attr])*
                $field_vis $field_name : $field_type
            ),*
        }

        impl $Type {
            $(
                #[doc = "Update the field named after this method in place and return the updated `self`."]
                $field_vis fn $field_name (mut self, value: $field_type) -> Self {
                    self.$field_name = value;
                    self
                }
            )+
        }
    }
}
