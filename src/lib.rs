#![no_std]

//! Helper crate to concat static strings during compilation.
//!
//! Strings (`&'static str`) that are known during compile-time can't
//! currently easily concatenated nor joined with a seperator. This
//! tiny crate aims to help with the situation by providing conviencne
//! macros that execute the required steps.
//!
//! Some of the limitations that have to be worked around are:
//!   * No const iterators, yet.
//!   * Allocating static memory must be done with a fixed and know size.
//!   * Writing into strings must be done "manually" bytewise as there
//!     is no copy implementation.
//!   * Byteslices aren't available in const-context requiring
//!     manually indexing into the buffers.
//!
//! All of those aren't show-stoppers but don't exactly enable
//! idiomatic Rust code and require walking the thin line of whats
//! possible and what isn't.
//!
//! Example usage:
//! ```rust
//! use const_str_join::declare_joined_str;
//!
//! const A: &'static str = "A";
//! const B: &'static str = "B";
//! const C: &'static str = "C";
//! const ALL: [&'static str; 3] = [A, B, C] ;
//! // declare a new &'static str with the final string
//! const ALL_JOINED: &'static str = declare_joined_str!(ALL, ",");
//! assert_eq!(ALL_JOINED, "A,B,C");
//! ```

#[doc(hidden)]
pub const fn concated_size<const N: usize>(array: [&'static str; N], sep: &'static str) -> usize {
    let mut n = if N == 0 { 0 } else { N - 1 } * sep.len();
    let mut i = N;
    loop {
        if i <= 0 {
            break;
        }
        i -= 1;
        n += array[i].len();
    }
    n
}

#[doc(hidden)]
pub const fn copy_bytes(src: &[u8], dest: &mut [u8], offset: usize) -> usize {
    let mut i = 0;
    let mut op = offset;
    if !src.is_empty() {
        loop {
            assert!(dest[op] == b'\0');
            dest[op] = src[i];
            op += 1;
            i += 1;
            if i >= src.len() {
                break;
            }
        }
    }

    op
}

#[doc(hidden)]
pub const fn join_strings(inputs: &[&str], sep: Option<&str>, mut output: &mut [u8]) -> usize {
    assert!(output.len() > 0);
    let mut n = 0;
    let mut op = 0;
    loop {
        op = copy_bytes(&inputs[n].as_bytes(), &mut output, op);

        if n + 1 < inputs.len()
            && let Some(sep) = sep
        {
            let s = sep.as_bytes();
            op = copy_bytes(s, &mut output, op);
        }

        n += 1;
        if n >= inputs.len() {
            break;
        }
    }
    op
}

/// Returns a buffer (`[u8; N]`) that contains the joined string as bytes.
///
/// Example usage:
/// ```rust
/// let s: [u8; _] = const_str_join::joined_array!(["A", "B", "C"], "<>");
/// assert_eq!(&s, b"A<>B<>C")
/// ```
#[macro_export]
macro_rules! joined_array {
    ($array:expr, $sep:expr) => {
        const {
            const SIZE: usize = $crate::concated_size($array, $sep);
	    $crate::joined_array!($array, $sep, SIZE)
        }
    };
    ($array:expr, $sep:expr, $size:expr) => {
        const {
            const ARRAY_LEN: usize = $size;

            let sep = $sep;
	    let sep = if sep.len() > 0 { Some(sep) } else { None };
            let array = &$array;

            let mut buffer = [0u8; ARRAY_LEN];
	    let next_position = $crate::join_strings(array, sep, &mut buffer);

	    // when we are done we should have written to all the bytes, if not then something is off
	    assert!(next_position == ARRAY_LEN);
            buffer
        }
    };

}

/// Declares a new constant value `name` with the joined string of `array` and `sep`.
/// Example usage:
/// ```rust
/// const FLAGS: &'static str = const_str_join::declare_joined_str!(["--help", "--version", "--verbose"], "|");
/// const HELP: &'static str = const_str_join::declare_joined_str!(["flags:", FLAGS], " ");
/// assert_eq!(HELP, "flags: --help|--version|--verbose");
/// ```

#[macro_export]
macro_rules! declare_joined_str {
    ($array:expr, $sep:expr) => {
        const {
            const SIZE: usize = $crate::concated_size($array, $sep);
            static STORAGE: [u8; SIZE] = $crate::joined_array!($array, $sep, SIZE);
            // no unwrap in const :|
            if let Ok(v) = core::str::from_utf8(&STORAGE) {
                v
            } else {
                panic!("joined array isn't a valid utf8 string");
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &'static str = "A";
    const B: &'static str = "B";
    const C: &'static str = "C";
    const ARRAY_OF_STRINGS: [&'static str; 3] = [A, B, C];

    #[test]
    fn nested() {
        const FOO: &'static str = declare_joined_str!(ARRAY_OF_STRINGS, ":");
        const MORE_PARTS: [&'static str; 3] = ["<", FOO, ">"];
        const MORE: &'static str = declare_joined_str!(MORE_PARTS, "");
        assert_eq!(FOO, "A:B:C");
        assert_eq!(MORE, "<A:B:C>");
    }

    #[test]
    fn joined_array() {
        let s = joined_array!(ARRAY_OF_STRINGS, "-");
        assert_eq!(&s, b"A-B-C");
    }
}
