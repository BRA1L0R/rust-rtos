use core::fmt::Debug;

macro_rules! define_svcallid {
    ($strn:tt, $errn:tt, $($name:tt: $val:expr),+) => {
        #[repr(u32)]
        pub enum $strn {
            $(
                $name = $val,
            )*
        }

        #[derive(Debug)]
        pub struct $errn;

        impl TryFrom<u32> for $strn {
            type Error = $errn;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $val => Ok($strn::$name),
                    )*
                    _ => Err($errn)
                }
            }
        }
    };
}

define_svcallid!(
    SVCallId, InvalidSVC,
    Yield: 0x00,
    Print: 0x01,
    ReadChar: 0x02,
    FreeMem: 0x03,
    Stop: 0x04
);
