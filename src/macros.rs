use crate::err::*;
use crate::Color;

#[macro_export]
macro_rules! calc {
    (@call $fn: ident; $a: expr; $b: expr) => {{
        $a.$fn($b)
    }};
    ($a: tt + $b: tt $($tail: tt)*) => {{
        let result = calc!(@call saturating_add; $a; $b);
        calc!(result $($tail)*)
    }};
    ($a: tt - $b: tt $($tail:tt)*) => {{
        let result = calc!(@call saturating_sub; $a; $b);
        calc!(result $($tail)*)
    }};
    ($a: tt * $b: tt $($tail:tt)*) => {{
        let result = calc!(@call saturating_mul; $a; $b);
        calc!(result $($tail)*)
    }};
    ($a: expr) => {{
        $a
    }};
}

/// Convert number type to another using try_into(), and map the error to
/// PngErr::IntOverflow on failure. The second argument can be a calc!
/// computation, but this is executed first and then converted.
///
/// @example
/// convert!(usize; 1000u32).unwrap(); // 1000usize
///
/// @example
/// convert!(u16; 200u8 + 100).unwrap(); // 255u16
///
/// @example
/// convert!(expect u8; 256); // Panics
#[macro_export]
macro_rules! convert {
    ($type: ty; $($tail: tt)+) => {{
        calc!($($tail)+).try_into().map_err(|_| PngErr::IntOverflow) as PngRes<$type>
    }};
    (ex $type: ty; $($tail: tt)+) => {{
        convert!($type; $($tail)+).expect("Could not convert number type.")
    }};
}

#[macro_export]
macro_rules! col {
    ($r: expr, $g: expr, $b: expr, $a: expr) => {{
        u32::from_be_bytes([$r, $g, $b, $a])
    }};
    ($r: expr, $g: expr, $b: expr) => {{
        u32::from_be_bytes([$r, $g, $b, 255])
    }};
    ($col: expr) => {{
        let [r_, rg, gb, ba] = $col._color_value().to_be_bytes();
        if (r_ > 0) {
            col!(r_, rg, gb, ba)
        } else {
            col!(rg, gb, ba)
        }
    }};
}

#[macro_export]
macro_rules! area {
    ($a: expr, $b: expr) => {
        calc!(
            convert!(ex usize; $a) * convert!(ex usize; $b)
        )
    }
}

mod tests {
    use super::Color;
    use crate::err::*;
    use std::panic;

    #[test]
    fn area() {
        assert_eq!(area!(10u32, 10u32), 100usize);
        assert!(panic::catch_unwind(|| area!(usize::MAX, usize::MAX)).is_err());
    }

    #[test]
    fn col() {
        let opaque = u32::from_be_bytes([0xAA, 0xBB, 0xCC, 0xFF]);
        let transparent = u32::from_be_bytes([0xAA, 0xBB, 0xCC, 0xDD]);

        assert_eq!(col!(0xAA, 0xBB, 0xCC), opaque);
        assert_eq!(col!(0xAA, 0xBB, 0xCC, 0xDD), transparent);
        assert_eq!(col!(0xAABBCC), opaque);
        assert_eq!(col!(0xAABBCCDD), transparent);
    }

    #[test]
    fn calc() {
        assert_eq!(calc!(10u8 + 20 * 5 - 30 * 2 - 239), 1);
        assert_eq!(calc!(100u8 * 2 + 100 - 254), 1);
        assert_eq!(calc!(50u8 - 51), 0);
    }

    #[test]
    fn convert() {
        assert_eq!(convert!(u16; 100u8).unwrap(), 100u16);
        assert!(convert!(u8; 256u16).is_err());
        assert!(convert!(u8; -1i8).is_err());

        assert_eq!(convert!(u16; 10u8 + 20 * 5 - 30 * 2 - 239).unwrap(), 1u16);
        assert_eq!(convert!(u16; 100u8 * 2 + 100 - 254).unwrap(), 1u16);
        assert_eq!(convert!(u16; 50u8 - 51).unwrap(), 0u16);

        assert!(panic::catch_unwind(|| convert!(ex u8; 256)).is_err())
    }
}
