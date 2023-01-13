// MIT License

// Copyright (c) 2021 bonsairobo

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Simple, fast linearization of 2D, 3D, and 4D coordinates.
//!
//! The canonical choice of linearization function is row-major, i.e. stepping linearly through an N dimensional array would
//! step by X first, then Y, then Z, etc, assuming that `[T; N]` coordinates are provided as `[X, Y, Z, ...]`. More explicitly:
//!
//! ```text
//! linearize([x, y, z, ...]) = x + X_SIZE * y + X_SIZE * Y_SIZE * z + ...
//! ```
//!
//! To achieve a different layout, one only needs to choose a different permutation of coordinates. For example, column-major
//! layout would require coordinates specified as `[..., Z, Y, X]`. For a 3D layout where each Y level set is contiguous in
//! memory, either layout `[X, Z, Y]` or `[Z, X, Y]` would work.
//!
//! # Example: Indexing Multidimensional Arrays
//!
//! ```
//! use ndshape::{Shape, ConstShape3u32, ConstShape4u32, ConstPow2Shape3u32, RuntimeShape};
//!
//! // An arbitrary shape.
//! let shape = ConstShape3u32::<5, 6, 7>;
//! let index = shape.linearize([1, 2, 3]);
//! assert_eq!(index, 101);
//! assert_eq!(shape.delinearize(index), [1, 2, 3]);
//!
//! // A shape with power-of-two dimensions
//! // This allows us to use bit shifting and masking for linearization.
//! let shape = ConstPow2Shape3u32::<1, 2, 3>; // These are number of bits per dimension.
//! let index = shape.linearize([1, 2, 3]);
//! assert_eq!(index, 0b011_10_1);
//! assert_eq!(shape.delinearize(index), [1, 2, 3]);
//!
//! // A runtime shape.
//! let shape = RuntimeShape::<u32, 3>::new([5, 6, 7]);
//! let index = shape.linearize([1, 2, 3]);
//! assert_eq!(index, 101);
//! assert_eq!(shape.delinearize(index), [1, 2, 3]);
//!
//! // Use a shape for indexing an array in 4D.
//! // Step X, then Y, then Z, since that results in monotonic increasing indices.
//! // (Believe it or not, Rust's N-dimensional array (e.g. `[[T; N]; M]`)
//! // indexing is significantly slower than this).
//! let shape = ConstShape4u32::<5, 6, 7, 8>;
//! let data = [0; 5 * 6 * 7 * 8];
//! for w in 0..8 {
//!     for z in 0..7 {
//!         for y in 0..6 {
//!             for x in 0..5 {
//!                 let i = shape.linearize([x, y, z, w]);
//!                 assert_eq!(0, data[i as usize]);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Example: Negative Strides with Modular Arithmetic
//!
//! It is often beneficial to linearize a negative vector that results in a negative linear "stride." But when using unsigned
//! linear indices, a negative stride would require a modular arithmetic representation, where e.g. `-1` maps to `u32::MAX`.
//! This works fine with any [`Shape`](crate::Shape). You just need to be sure to use modular arithmetic with the resulting
//! linear strides, e.g. [`u32::wrapping_add`](u32::wrapping_add) and [`u32::wrapping_mul`](u32::wrapping_mul). Also, it is not
//! possible to delinearize a negative stride with modular arithmetic. For that, you must use signed integer coordinates.
//!
//! ```
//! use ndshape::{Shape, ConstShape3u32, ConstShape3i32};
//!
//! let shape = ConstShape3u32::<10, 10, 10>;
//! let stride = shape.linearize([0, -1i32 as u32, 0]);
//! assert_eq!(stride, -10i32 as u32);
//!
//! // Delinearize does not work with unsigned coordinates!
//! assert_ne!(shape.delinearize(stride), [0, -1i32 as u32, 0]);
//! assert_eq!(shape.delinearize(stride), [6, 8, 42949672]);
//!
//! let shape = ConstShape3i32::<10, 10, 10>;
//! let stride = shape.linearize([0, -1, 0]);
//! assert_eq!(stride, -10);
//!
//! // Delinearize works with signed coordinates.
//! assert_eq!(shape.delinearize(stride), [0, -1, 0]);
//! ```

/// The shape of an array with unspecified dimensionality.
pub trait AbstractShape<Coord, Vector> {
    /// The number of elements in an array with this shape.
    fn size(&self) -> Coord;
    /// Translates a vector `V` (with an unspecified number of dimensions) into a single number `T` that can be used for
    /// linear indexing.
    fn linearize(&self, p: Vector) -> Coord;
    /// The inverse of `linearize`.
    fn delinearize(&self, i: Coord) -> Vector;
}

/// The shape of an `N`-dimensional array.
pub trait Shape<const N: usize> {
    type Coord;

    /// The number of elements in an array with this shape.
    fn size(&self) -> Self::Coord;
    /// The same as `self.size() as usize`.
    fn usize(&self) -> usize;
    /// The dimensions of the shape.
    fn as_array(&self) -> [Self::Coord; N];
    /// Translate an `N`-dimensional vector into a single number `T` that can be used for linear indexing.
    fn linearize(&self, p: [Self::Coord; N]) -> Self::Coord;
    /// The inverse of `linearize`.
    fn delinearize(&self, i: Self::Coord) -> [Self::Coord; N];
}

/// A constant shape of an `N`-dimensional array.
pub trait ConstShape<const N: usize> {
    type Coord;

    /// The number of elements in an array with this shape.
    const SIZE: Self::Coord;
    /// Same as `Self::SIZE as usize`.
    const USIZE: usize;
    /// The dimensions of the shape.
    const ARRAY: [Self::Coord; N];
    /// Translate an `N`-dimensional vector into a single number `T` that can be used for linear indexing.
    fn linearize(p: [Self::Coord; N]) -> Self::Coord;
    /// The inverse of `linearize`.
    fn delinearize(i: Self::Coord) -> [Self::Coord; N];
}

impl<S, const N: usize> AbstractShape<S::Coord, [S::Coord; N]> for S
where
    S: Shape<N>,
{
    #[inline]
    fn size(&self) -> S::Coord {
        self.size()
    }
    #[inline]
    fn linearize(&self, p: [S::Coord; N]) -> S::Coord {
        self.linearize(p)
    }
    #[inline]
    fn delinearize(&self, i: S::Coord) -> [S::Coord; N] {
        self.delinearize(i)
    }
}

impl<S, const N: usize> Shape<N> for S
where
    S: ConstShape<N>,
{
    type Coord = S::Coord;

    #[inline]
    fn size(&self) -> Self::Coord {
        S::SIZE
    }
    #[inline]
    fn usize(&self) -> usize {
        S::USIZE
    }
    #[inline]
    fn as_array(&self) -> [Self::Coord; N] {
        S::ARRAY
    }
    #[inline]
    fn linearize(&self, p: [Self::Coord; N]) -> Self::Coord {
        S::linearize(p)
    }
    #[inline]
    fn delinearize(&self, i: Self::Coord) -> [Self::Coord; N] {
        S::delinearize(i)
    }
}

use static_assertions::assert_impl_all;

macro_rules! impl_const_shape2 {
    ($name:ident, $scalar:ty) => {
        #[derive(Clone, Debug, Copy, Eq, PartialEq)]
        pub struct $name<const X: $scalar, const Y: $scalar>;

        impl<const X: $scalar, const Y: $scalar> $name<X, Y> {
            pub const STRIDES: [$scalar; 2] = [1, X];
        }

        impl<const X: $scalar, const Y: $scalar> ConstShape<2> for $name<X, Y> {
            type Coord = $scalar;

            const ARRAY: [$scalar; 2] = [X, Y];
            const SIZE: $scalar = X * Y;
            const USIZE: usize = Self::SIZE as usize;

            #[inline]
            fn linearize(p: [$scalar; 2]) -> $scalar {
                p[0] + Self::STRIDES[1].wrapping_mul(p[1])
            }

            #[inline]
            fn delinearize(i: $scalar) -> [$scalar; 2] {
                let y = i / Self::STRIDES[1];
                let x = i % Self::STRIDES[1];
                [x, y]
            }
        }

        assert_impl_all!($name<1, 1>: AbstractShape<$scalar, [$scalar; 2]>);
        assert_impl_all!($name<1, 1>: Shape<2>);
    };
}

impl_const_shape2!(ConstShape2u8, u8);
impl_const_shape2!(ConstShape2u16, u16);
impl_const_shape2!(ConstShape2u32, u32);
impl_const_shape2!(ConstShape2u64, u64);
impl_const_shape2!(ConstShape2usize, usize);

impl_const_shape2!(ConstShape2i8, i8);
impl_const_shape2!(ConstShape2i16, i16);
impl_const_shape2!(ConstShape2i32, i32);
impl_const_shape2!(ConstShape2i64, i64);

macro_rules! impl_const_shape3 {
    ($name:ident, $scalar:ty) => {
        #[derive(Clone, Debug, Copy, Eq, PartialEq)]
        pub struct $name<const X: $scalar, const Y: $scalar, const Z: $scalar>;

        impl<const X: $scalar, const Y: $scalar, const Z: $scalar> $name<X, Y, Z> {
            pub const STRIDES: [$scalar; 3] = [1, X, X * Y];
        }

        impl<const X: $scalar, const Y: $scalar, const Z: $scalar> ConstShape<3>
            for $name<X, Y, Z>
        {
            type Coord = $scalar;

            const ARRAY: [$scalar; 3] = [X, Y, Z];
            const SIZE: $scalar = X * Y * Z;
            const USIZE: usize = Self::SIZE as usize;

            #[inline]
            fn linearize(p: [$scalar; 3]) -> $scalar {
                p[0] + Self::STRIDES[1].wrapping_mul(p[1]) + Self::STRIDES[2].wrapping_mul(p[2])
            }

            #[inline]
            fn delinearize(mut i: $scalar) -> [$scalar; 3] {
                let z = i / Self::STRIDES[2];
                i -= z * Self::STRIDES[2];
                let y = i / Self::STRIDES[1];
                let x = i % Self::STRIDES[1];
                [x, y, z]
            }
        }

        assert_impl_all!($name<1, 1, 1>: AbstractShape<$scalar, [$scalar; 3]>);
        assert_impl_all!($name<1, 1, 1>: Shape<3>);
    };
}

impl_const_shape3!(ConstShape3u8, u8);
impl_const_shape3!(ConstShape3u16, u16);
impl_const_shape3!(ConstShape3u32, u32);
impl_const_shape3!(ConstShape3u64, u64);
impl_const_shape3!(ConstShape3usize, usize);

impl_const_shape3!(ConstShape3i8, i8);
impl_const_shape3!(ConstShape3i16, i16);
impl_const_shape3!(ConstShape3i32, i32);
impl_const_shape3!(ConstShape3i64, i64);

macro_rules! impl_const_shape4 {
    ($name:ident, $scalar:ty) => {
        #[derive(Clone, Debug, Copy, Eq, PartialEq)]
        pub struct $name<const X: $scalar, const Y: $scalar, const Z: $scalar, const W: $scalar>;

        impl<const X: $scalar, const Y: $scalar, const Z: $scalar, const W: $scalar>
            $name<X, Y, Z, W>
        {
            pub const STRIDES: [$scalar; 4] = [1, X, X * Y, X * Y * Z];
        }

        impl<const X: $scalar, const Y: $scalar, const Z: $scalar, const W: $scalar>
            ConstShape<4> for $name<X, Y, Z, W>
        {
            type Coord = $scalar;

            const ARRAY: [$scalar; 4] = [X, Y, Z, W];
            const SIZE: $scalar = X * Y * Z * W;
            const USIZE: usize = Self::SIZE as usize;

            #[inline]
            fn linearize(p: [$scalar; 4]) -> $scalar {
                p[0] +
                    Self::STRIDES[1].wrapping_mul(p[1]) +
                    Self::STRIDES[2].wrapping_mul(p[2]) +
                    Self::STRIDES[3].wrapping_mul(p[3])
            }

            #[inline]
            fn delinearize(mut i: $scalar) -> [$scalar; 4] {
                let w = i / Self::STRIDES[3];
                i -= w * Self::STRIDES[3];
                let z = i / Self::STRIDES[2];
                i -= z * Self::STRIDES[2];
                let y = i / Self::STRIDES[1];
                let x = i % Self::STRIDES[1];
                [x, y, z, w]
            }
        }

        assert_impl_all!($name<1, 1, 1, 1>: AbstractShape<$scalar, [$scalar; 4]>);
        assert_impl_all!($name<1, 1, 1, 1>: Shape<4>);
    };
}

impl_const_shape4!(ConstShape4u8, u8);
impl_const_shape4!(ConstShape4u16, u16);
impl_const_shape4!(ConstShape4u32, u32);
impl_const_shape4!(ConstShape4u64, u64);
impl_const_shape4!(ConstShape4usize, usize);

impl_const_shape4!(ConstShape4i8, i8);
impl_const_shape4!(ConstShape4i16, i16);
impl_const_shape4!(ConstShape4i32, i32);
impl_const_shape4!(ConstShape4i64, i64);

macro_rules! impl_const_pow2_shape2 {
    ($name:ident, $scalar:ty) => {
        #[derive(Clone, Debug, Copy, Eq, PartialEq)]
        pub struct $name<const X: $scalar, const Y: $scalar>;

        impl<const X: $scalar, const Y: $scalar> $name<X, Y> {
            pub const SHIFTS: [$scalar; 2] = [0, X];

            pub const MASKS: [$scalar; 2] = [
                !(!0 << X),
                !(!0 << Y) << Self::SHIFTS[1]
            ];
        }

        impl<const X: $scalar, const Y: $scalar> ConstShape<2> for $name<X, Y> {
            type Coord = $scalar;

            const ARRAY: [$scalar; 2] = [1 << X, 1 << Y];
            const SIZE: $scalar = 1 << (X + Y);
            const USIZE: usize = Self::SIZE as usize;

            #[inline]
            fn linearize(p: [$scalar; 2]) -> $scalar {
                (p[1] << Self::SHIFTS[1]) | p[0]
            }

            #[inline]
            fn delinearize(i: $scalar) -> [$scalar; 2] {
                [(i & Self::MASKS[0]), ((i & Self::MASKS[1]) >> Self::SHIFTS[1])]
            }
        }

        assert_impl_all!($name<1, 1>: AbstractShape<$scalar, [$scalar; 2]>);
        assert_impl_all!($name<1, 1>: Shape<2>);
    };
}

impl_const_pow2_shape2!(ConstPow2Shape2u8, u8);
impl_const_pow2_shape2!(ConstPow2Shape2u16, u16);
impl_const_pow2_shape2!(ConstPow2Shape2u32, u32);
impl_const_pow2_shape2!(ConstPow2Shape2u64, u64);
impl_const_pow2_shape2!(ConstPow2Shape2usize, usize);

impl_const_pow2_shape2!(ConstPow2Shape2i8, i8);
impl_const_pow2_shape2!(ConstPow2Shape2i16, i16);
impl_const_pow2_shape2!(ConstPow2Shape2i32, i32);
impl_const_pow2_shape2!(ConstPow2Shape2i64, i64);

macro_rules! impl_const_pow2_shape3 {
    ($name:ident, $scalar:ty) => {
        #[derive(Clone, Debug, Copy, Eq, PartialEq)]
        pub struct $name<const X: $scalar, const Y: $scalar, const Z: $scalar>;

        impl<const X: $scalar, const Y: $scalar, const Z: $scalar> $name<X, Y, Z> {
            pub const SHIFTS: [$scalar; 3] = [0, X, X + Y];

            pub const MASKS: [$scalar; 3] = [
                !(!0 << X),
                !(!0 << Y) << Self::SHIFTS[1],
                !(!0 << Z) << Self::SHIFTS[2],
            ];
        }

        impl<const X: $scalar, const Y: $scalar, const Z: $scalar> ConstShape<3>
            for $name<X, Y, Z>
        {
            type Coord = $scalar;

            const ARRAY: [$scalar; 3] = [1 << X, 1 << Y, 1 << Z];
            const SIZE: $scalar = 1 << (X + Y + Z);
            const USIZE: usize = Self::SIZE as usize;

            #[inline]
            fn linearize(p: [$scalar; 3]) -> $scalar {
                (p[2] << Self::SHIFTS[2]) | (p[1] << Self::SHIFTS[1]) | p[0]
            }

            #[inline]
            fn delinearize(i: $scalar) -> [$scalar; 3] {
                [
                    (i & Self::MASKS[0]),
                    ((i & Self::MASKS[1]) >> Self::SHIFTS[1]),
                    ((i & Self::MASKS[2]) >> Self::SHIFTS[2]),
                ]
            }
        }

        assert_impl_all!($name<1, 1, 1>: AbstractShape<$scalar, [$scalar; 3]>);
        assert_impl_all!($name<1, 1, 1>: Shape<3>);
    };
}

impl_const_pow2_shape3!(ConstPow2Shape3u8, u8);
impl_const_pow2_shape3!(ConstPow2Shape3u16, u16);
impl_const_pow2_shape3!(ConstPow2Shape3u32, u32);
impl_const_pow2_shape3!(ConstPow2Shape3u64, u64);
impl_const_pow2_shape3!(ConstPow2Shape3usize, usize);

impl_const_pow2_shape3!(ConstPow2Shape3i8, i8);
impl_const_pow2_shape3!(ConstPow2Shape3i16, i16);
impl_const_pow2_shape3!(ConstPow2Shape3i32, i32);
impl_const_pow2_shape3!(ConstPow2Shape3i64, i64);

macro_rules! impl_const_pow2_shape4 {
    ($name:ident, $scalar:ty) => {
        #[derive(Clone, Debug, Copy, Eq, PartialEq)]
        pub struct $name<const X: $scalar, const Y: $scalar, const Z: $scalar, const W: $scalar>;

        impl<const X: $scalar, const Y: $scalar, const Z: $scalar, const W: $scalar>
            $name<X, Y, Z, W>
        {
            pub const SHIFTS: [$scalar; 4] = [0, X, X + Y, X + Y + Z];

            pub const MASKS: [$scalar; 4] = [
                !(!0 << X),
                !(!0 << Y) << Self::SHIFTS[1],
                !(!0 << Z) << Self::SHIFTS[2],
                !(!0 << W) << Self::SHIFTS[3],
            ];
        }

        impl<const X: $scalar, const Y: $scalar, const Z: $scalar, const W: $scalar>
            ConstShape<4> for $name<X, Y, Z, W>
        {
            type Coord = $scalar;

            const ARRAY: [$scalar; 4] = [1 << X, 1 << Y, 1 << Z, 1 << W];
            const SIZE: $scalar = 1 << (X + Y + Z + W);
            const USIZE: usize = Self::SIZE as usize;

            #[inline]
            fn linearize(p: [$scalar; 4]) -> $scalar {
                (p[3] << Self::SHIFTS[3]) | (p[2] << Self::SHIFTS[2]) | (p[1] << Self::SHIFTS[1]) | p[0]
            }

            #[inline]
            fn delinearize(i: $scalar) -> [$scalar; 4] {
                [
                    (i & Self::MASKS[0]),
                    ((i & Self::MASKS[1]) >> Self::SHIFTS[1]),
                    ((i & Self::MASKS[2]) >> Self::SHIFTS[2]),
                    ((i & Self::MASKS[3]) >> Self::SHIFTS[3]),
                ]
            }
        }

        assert_impl_all!($name<1, 1, 1, 1>: AbstractShape<$scalar, [$scalar; 4]>);
        assert_impl_all!($name<1, 1, 1, 1>: Shape<4>);
    };
}

impl_const_pow2_shape4!(ConstPow2Shape4u8, u8);
impl_const_pow2_shape4!(ConstPow2Shape4u16, u16);
impl_const_pow2_shape4!(ConstPow2Shape4u32, u32);
impl_const_pow2_shape4!(ConstPow2Shape4u64, u64);
impl_const_pow2_shape4!(ConstPow2Shape4usize, usize);

impl_const_pow2_shape4!(ConstPow2Shape4i8, i8);
impl_const_pow2_shape4!(ConstPow2Shape4i16, i16);
impl_const_pow2_shape4!(ConstPow2Shape4i32, i32);
impl_const_pow2_shape4!(ConstPow2Shape4i64, i64);
