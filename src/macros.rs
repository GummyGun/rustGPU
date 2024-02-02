#[macro_export]
macro_rules! impl_deref {
    ($base:ty, $ret_type:ty, $($field_path:ident).+) => {
        impl ::core::ops::Deref for $base {
            type Target  = $ret_type;
            fn deref(&self) -> &Self::Target {
                &self.$($field_path).+
            }
        }
    };
}
pub use impl_deref;


#[macro_export]
macro_rules! impl_deref_mut {
    ($base:ty, $ret_type:ty, $($field_path:ident).+) => {
        impl ::core::ops::Deref for $base {
            type Target  = $ret_type;
            fn deref(&self) -> &Self::Target {
                &self.$($field_path).+
            }
        }
        
        impl ::core::ops::DerefMut for $base {
            fn deref_mut (&mut self) -> &mut Self::Target {
                &mut self.$($field_path).+
            }
        }
    };
}
pub use impl_deref_mut;


#[macro_export]
macro_rules! impl_underlying {
    ($base:ty, $ret_type:ty, $($field_path:ident).+) => {
        impl $base {
            pub fn underlying(&self) -> $ret_type {
                self.$($field_path).+.clone()
            }
        }
    };
}
pub use impl_underlying;


