#[macro_export]
macro_rules! impl_string_newtype {
    ($name:ty) => {
        impl $name {
            pub fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }
        }
    };
}
