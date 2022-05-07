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

#[macro_export]
macro_rules! bitflag_struct {
    ($struct_name:ident : $enum_name:ident) => {
        #[derive(Copy, Clone, Default)]
        #[repr(C)]
        pub struct $struct_name {
            pub value: u32,
        }
        impl From<u32> for $struct_name {
            fn from(value: u32) -> Self {
                Self {
                    value,
                }
            }
        }
        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut value = self.value;
                for i in 0..31 {
                    let flag = (value & 1) << i;
                    value >>= 1;
                    if flag != 0 {
                        write!(f, "{:?}({})", $enum_name::from(flag), flag)?;
                        if value > 0 {
                            write!(f, " | ")?;
                        }
                    }
                }
                write!(f, "")
            }
        }
    };
}

#[macro_export]
macro_rules! bitflag_enum {
    (
        $enum_name:ident {
            $($variant:ident = $value:expr,)*
        }
    ) => {
        #[repr(u32)]
        #[derive(Debug)]
        pub enum $enum_name {
            $($variant = $value,)*
        }
        impl From<u32> for $enum_name {
            fn from(flag: u32) -> Self {
                match flag {
                    $($value => $enum_name::$variant,)*
                    n => panic!("Invalid flag: {}", n),
                }
            }
        }
        $(pub const $variant: u32 = $value;)*
    };
}
