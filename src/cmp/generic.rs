use std::cmp::Ordering;
use std::fmt::{self, Display, Write};
use std::hash::Hasher;

pub fn eq<T: Display + ?Sized, U: Display + ?Sized>(lhs: &T, rhs: &U) -> bool {
    cmp(lhs, rhs) == Ordering::Equal
}

pub fn cmp<T: Display + ?Sized, U: Display + ?Sized>(lhs: &T, rhs: &U) -> Ordering {
    struct State {
        ret: Ordering,
        rhs_is_remaining: bool,
    }

    struct Rhs<'a, T: ?Sized> {
        rhs: &'a T,
        /// Byte position in `lhs.to_string()` that we are reading.
        pos: usize,
        state: State,
    }

    let state = State {
        ret: Ordering::Equal,
        rhs_is_remaining: false,
    };
    let mut adapter = Rhs { rhs, pos: 0, state };

    // `write!` returns an error if: 1. the adapter is trying an early-return, or 2. `T::fmt`
    // returned an error. 2. indicates an incorrect `Display` implementation so we only need to
    // consider the case of 1.
    let _ = write!(&mut adapter, "{}", &lhs);

    return adapter.state.ret.then(if adapter.state.rhs_is_remaining {
        Ordering::Less
    } else {
        Ordering::Equal
    });

    struct Lhs<'a> {
        lhs: &'a [u8],
        /// Number of bytes to skip until we get to `rhs.to_string()[pos]`.
        skip: usize,
        state: &'a mut State,
    }

    impl<T: Display + ?Sized> Write for Rhs<'_, T> {
        fn write_str(&mut self, lhs: &str) -> fmt::Result {
            //       |-pos
            // T |---+-------+--|
            //       ^^^^^^^^^-lhs
            // U |-+---+---+-------+--|

            self.state.rhs_is_remaining = false;

            let mut adapter = Lhs {
                lhs: lhs.as_bytes(),
                skip: self.pos,
                state: &mut self.state,
            };

            let _ = write!(&mut adapter, "{}", self.rhs);

            // Get `is_empty` first to make borrowck happy.
            let lhs_is_empty = adapter.lhs.is_empty();
            if self.state.ret != Ordering::Equal {
                // Short-circuit by returning an error.
                return Err(fmt::Error);
            }
            if !lhs_is_empty {
                // `adapter.lhs` remained after `rhs` was exhausted, which means that `lhs` is
                // longer than `rhs`.
                // T |---+-------+--|
                //       ^-pos ^^^-adapter.lhs
                // U |-+---+---|
                self.state.ret = Ordering::Greater;
                return Err(fmt::Error);
            }

            self.pos += lhs.len();

            Ok(())
        }
    }

    impl Write for Lhs<'_> {
        fn write_str(&mut self, rhs: &str) -> fmt::Result {
            //       |-pos
            // T |---+-------+--|
            //       ^^^^^^^^^-lhs
            // U |-+---+---+-------+--|
            //     ^^^^^-rhs
            //     ^^^-range to skip

            let skip = self.skip.min(rhs.len());
            self.skip -= skip;
            let rhs = &rhs.as_bytes()[skip..];

            let read = rhs.len().min(self.lhs.len());
            self.state.ret = self.lhs[0..read].cmp(&rhs[0..read]);
            if self.state.ret != Ordering::Equal {
                return Err(fmt::Error);
            }
            self.lhs = &self.lhs[read..];
            if rhs.len() > read {
                // This chunk of `rhs` remained after `self.lhs` was exhausted, which means that
                // the whole `rhs` _may_ be longer than `lhs`. Although there may still be upcoming
                // `lhs` chunks, the `Formatter` won't let us know the existence of a next chunk,
                // so we are speculatively recording the fact on `rhs_is_remaining`, which will be
                // reverted if a next `lhs` chunk is provided.
                // T |---+-------+??|
                //       ^pos  ^^^-self.lhs
                // U |-+---+---+-------+--|
                //             ^^^^^^^^^-rhs
                //             ^^^-rhs[0..read]
                self.state.rhs_is_remaining = true;
                return Err(fmt::Error);
            }

            Ok(())
        }
    }
}

pub fn hash<T: Display + ?Sized, H: Hasher>(hashee: &T, hasher: &mut H) {
    struct Adapter<'a, H>(&'a mut H);
    impl<H: Hasher> Write for Adapter<'_, H> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.0.write(s.as_bytes());
            Ok(())
        }
    }

    write!(Adapter(&mut *hasher), "{}", &hashee).unwrap();
    // Pass an extra `0xFF` to avoid prefix collisions.
    // cf. <https://doc.rust-lang.org/1.57.0/core/hash/trait.Hash.html#prefix-collisions>
    hasher.write_u8(0xff);
}
