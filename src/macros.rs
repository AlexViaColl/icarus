#[macro_export]
macro_rules! cstr(
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    }
);

#[macro_export]
macro_rules! opaque(
    ($pub_name: ident, $priv_name: ident) => {
        #[repr(C)]
        pub struct $priv_name {
            _data: [u8; 0],
            _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
        }

        #[repr(transparent)]
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct $pub_name(*mut $priv_name);
        impl Default for $pub_name {
            fn default() -> Self {
                Self(std::ptr::null_mut())
            }
        }
    }
);
