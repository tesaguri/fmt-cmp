#![feature(test)]

extern crate test;

use std::cmp::Ordering;
use std::fmt::Display;
use test::Bencher;

const D1L: u64 = 9;
const D1R: u64 = 1;
const D4L: u64 = 9_876;
const D4R: u64 = 1_234;
const D4A: u64 = 9_874;
const D16L: u64 = 9_876_543_210_123_456;
const D16R: u64 = 1_234_567_890_987_654;
const D16A: u64 = 9_876_543_210_123_454;

macro_rules! bench {
    (
        $cmp:expr;
        $(#[$attr:meta])*
        $name_1_eq:ident; $name_1_ne:ident;
        $name_4_eq:ident; $name_4_ne:ident; $name_4_approxeq:ident;
        $name_16_eq:ident; $name_16_ne:ident; $name_16_approxeq:ident;
        $name_4_16:ident;
    ) => {
        bench! { @fn $(#[$attr])* $name_1_eq(D1L, D1L) = $cmp }
        bench! { @fn $(#[$attr])* $name_1_ne(D1L, D1R) = $cmp }
        bench! { @fn $(#[$attr])* $name_4_eq(D4L, D4L) = $cmp }
        bench! { @fn $(#[$attr])* $name_4_approxeq(D4L, D4A) = $cmp }
        bench! { @fn $(#[$attr])* $name_4_ne(D4L, D4R) = $cmp }
        bench! { @fn $(#[$attr])* $name_16_eq(D16L, D16L) = $cmp }
        bench! { @fn $(#[$attr])* $name_16_ne(D16L, D16R) = $cmp }
        bench! { @fn $(#[$attr])* $name_16_approxeq(D16L, D16A) = $cmp }
        bench! { @fn $(#[$attr])* $name_4_16(D4L, D16R) = $cmp }
    };
    (@fn $(#[$attr:meta])* $name:ident($lhs:expr, $rhs:expr) = $cmp:expr) => {
        $(#[$attr])*
        #[bench]
        fn $name(b: &mut Bencher) {
            // Take `$cmp` as `impl Fn` so that type inference works on closure arguments.
            fn run(cmp: impl Fn(&u64, &u64) -> Ordering) -> (Ordering, Ordering) {
                let (lhs, rhs) = test::black_box(($lhs, $rhs));
                (cmp(&lhs, &rhs), cmp(&rhs, &lhs))
            }
            b.iter(|| run($cmp));
        }
    };
}

bench! {
    u64::cmp;
    #[ignore]
    native_01_digit_eq; native_01_digit_ne;
    native_04_digits_eq; native_04_digits_ne; native_04_digits_approxeq;
    native_16_digits_eq; native_16_digits_ne; native_16_digits_approxeq;
    native_04_16_digits;
}

bench! {
    |lhs, rhs| (*lhs.to_string()).cmp(&*rhs.to_string());
    #[ignore]
    to_string_01_digit_eq; to_string_01_digit_ne;
    to_string_04_digits_eq; to_string_04_digits_ne; to_string_04_digits_approxeq;
    to_string_16_digits_eq; to_string_16_digits_ne; to_string_16_digits_approxeq;
    to_string_04_16_digits;
}

bench! {
    |&lhs, &rhs| {
        let (mut lbuf, mut rbuf) = (itoa::Buffer::new(), itoa::Buffer::new());
        let (lhs, rhs) = (lbuf.format(lhs), rbuf.format(rhs));
        lhs.cmp(rhs)
    };
    itoa_01_digit_eq; itoa_01_digit_ne;
    itoa_04_digits_eq; itoa_04_digits_ne; itoa_04_digits_approxeq;
    itoa_16_digits_eq; itoa_16_digits_ne; itoa_16_digits_approxeq;
    itoa_04_16_digits;
}

bench! {
    fmt_cmp::cmp;
    fmt_cmp_01_digit_eq; fmt_cmp_01_digit_ne;
    fmt_cmp_04_digits_eq; fmt_cmp_04_digits_ne; fmt_cmp_04_digits_approxeq;
    fmt_cmp_16_digits_eq; fmt_cmp_16_digits_ne; fmt_cmp_16_digits_approxeq;
    fmt_cmp_04_16_digits;
}

bench! {
    |lhs, rhs| {
        let (lhs, rhs) = test::black_box::<(&dyn Display, &dyn Display)>((lhs, rhs));
        fmt_cmp::cmp(lhs, rhs)
    };
    fmt_cmp_dyn_01_digit_eq; fmt_cmp_dyn_01_digit_ne;
    fmt_cmp_dyn_04_digits_eq; fmt_cmp_dyn_04_digits_ne; fmt_cmp_dyn_04_digits_approxeq;
    fmt_cmp_dyn_16_digits_eq; fmt_cmp_dyn_16_digits_ne; fmt_cmp_dyn_16_digits_approxeq;
    fmt_cmp_dyn_04_16_digits;
}

bench! {
    |&lhs, &rhs| fmt_cmp::cmp_int(lhs, rhs, 10);
    cmp_int_01_digit_eq; cmp_int_01_digit_ne;
    cmp_int_04_digits_eq; cmp_int_04_digits_ne; cmp_int_04_digits_approxeq;
    cmp_int_16_digits_eq; cmp_int_16_digits_ne; cmp_int_16_digits_approxeq;
    cmp_int_04_16_digits;
}

bench! {
    |&lhs, &rhs| fmt_cmp::cmp_dec(lhs, rhs);
    cmp_dec_01_digit_eq; cmp_dec_01_digit_ne;
    cmp_dec_04_digits_eq; cmp_dec_04_digits_ne; cmp_dec_04_digits_approxeq;
    cmp_dec_16_digits_eq; cmp_dec_16_digits_ne; cmp_dec_16_digits_approxeq;
    cmp_dec_04_16_digits;
}
