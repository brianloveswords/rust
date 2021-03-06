// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Optional values
//!
//! Type `Option` represents an optional value: every `Option`
//! is either `Some` and contains a value, or `None`, and
//! does not. `Option` types are very common in Rust code, as
//! they have a number of uses:
//!
//! * Initial values
//! * Return values for functions that are not defined
//!   over their entire input range (partial functions)
//! * Return value for otherwise reporting simple errors, where `None` is
//!   returned on error
//! * Optional struct fields
//! * Struct fields that can be loaned or "taken"
//! * Optional function arguments
//! * Nullable pointers
//! * Swapping things out of difficult situations
//!
//! Options are commonly paired with pattern matching to query the presence
//! of a value and take action, always accounting for the `None` case.
//!
//! ```
//! fn divide(numerator: f64, denominator: f64) -> Option<f64> {
//!     if denominator == 0.0 {
//!         None
//!     } else {
//!         Some(numerator / denominator)
//!     }
//! }
//!
//! // The return value of the function is an option
//! let result = divide(2.0, 3.0);
//!
//! // Pattern match to retrieve the value
//! match result {
//!     // The division was valid
//!     Some(x) => println!("Result: {}", x),
//!     // The division was invalid
//!     None    => println!("Cannot divide by 0")
//! }
//! ```
//!
//
// FIXME: Show how `Option` is used in practice, with lots of methods
//
//! # Options and pointers ("nullable" pointers)
//!
//! Rust's pointer types must always point to a valid location; there are
//! no "null" pointers. Instead, Rust has *optional* pointers, like
//! the optional owned box, `Option<Box<T>>`.
//!
//! The following example uses `Option` to create an optional box of
//! `int`. Notice that in order to use the inner `int` value first the
//! `check_optional` function needs to use pattern matching to
//! determine whether the box has a value (i.e. it is `Some(...)`) or
//! not (`None`).
//!
//! ```
//! let optional: Option<Box<int>> = None;
//! check_optional(&optional);
//!
//! let optional: Option<Box<int>> = Some(box 9000);
//! check_optional(&optional);
//!
//! fn check_optional(optional: &Option<Box<int>>) {
//!     match *optional {
//!         Some(ref p) => println!("have value {}", p),
//!         None => println!("have no value")
//!     }
//! }
//! ```
//!
//! This usage of `Option` to create safe nullable pointers is so
//! common that Rust does special optimizations to make the
//! representation of `Option<Box<T>>` a single pointer. Optional pointers
//! in Rust are stored as efficiently as any other pointer type.
//!
//! # Examples
//!
//! Basic pattern matching on `Option`:
//!
//! ```
//! let msg = Some("howdy");
//!
//! // Take a reference to the contained string
//! match msg {
//!     Some(ref m) => println!("{}", *m),
//!     None => ()
//! }
//!
//! // Remove the contained string, destroying the Option
//! let unwrapped_msg = match msg {
//!     Some(m) => m,
//!     None => "default message"
//! };
//! ```
//!
//! Initialize a result to `None` before a loop:
//!
//! ```
//! enum Kingdom { Plant(uint, &'static str), Animal(uint, &'static str) }
//!
//! // A list of data to search through.
//! let all_the_big_things = [
//!     Kingdom::Plant(250, "redwood"),
//!     Kingdom::Plant(230, "noble fir"),
//!     Kingdom::Plant(229, "sugar pine"),
//!     Kingdom::Animal(25, "blue whale"),
//!     Kingdom::Animal(19, "fin whale"),
//!     Kingdom::Animal(15, "north pacific right whale"),
//! ];
//!
//! // We're going to search for the name of the biggest animal,
//! // but to start with we've just got `None`.
//! let mut name_of_biggest_animal = None;
//! let mut size_of_biggest_animal = 0;
//! for big_thing in all_the_big_things.iter() {
//!     match *big_thing {
//!         Kingdom::Animal(size, name) if size > size_of_biggest_animal => {
//!             // Now we've found the name of some big animal
//!             size_of_biggest_animal = size;
//!             name_of_biggest_animal = Some(name);
//!         }
//!         Kingdom::Animal(..) | Kingdom::Plant(..) => ()
//!     }
//! }
//!
//! match name_of_biggest_animal {
//!     Some(name) => println!("the biggest animal is {}", name),
//!     None => println!("there are no animals :(")
//! }
//! ```

#![stable]

use self::Option::*;

use cmp::{Eq, Ord};
use default::Default;
use iter::{Iterator, IteratorExt, DoubleEndedIterator, FromIterator};
use iter::{ExactSizeIterator};
use mem;
use result::Result;
use result::Result::{Ok, Err};
use slice;
use slice::AsSlice;
use clone::Clone;
use ops::{Deref, FnOnce};

// Note that this is not a lang item per se, but it has a hidden dependency on
// `Iterator`, which is one. The compiler assumes that the `next` method of
// `Iterator` is an enumeration with one type parameter and two variants,
// which basically means it must be `Option`.

/// The `Option` type.
#[deriving(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Show, Hash)]
#[stable]
pub enum Option<T> {
    /// No value
    #[stable]
    None,
    /// Some value `T`
    #[stable]
    Some(T)
}

/////////////////////////////////////////////////////////////////////////////
// Type implementation
/////////////////////////////////////////////////////////////////////////////

impl<T> Option<T> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the option is a `Some` value
    ///
    /// # Example
    ///
    /// ```
    /// let x: Option<uint> = Some(2);
    /// assert_eq!(x.is_some(), true);
    ///
    /// let x: Option<uint> = None;
    /// assert_eq!(x.is_some(), false);
    /// ```
    #[inline]
    #[stable]
    pub fn is_some(&self) -> bool {
        match *self {
            Some(_) => true,
            None => false
        }
    }

    /// Returns `true` if the option is a `None` value
    ///
    /// # Example
    ///
    /// ```
    /// let x: Option<uint> = Some(2);
    /// assert_eq!(x.is_none(), false);
    ///
    /// let x: Option<uint> = None;
    /// assert_eq!(x.is_none(), true);
    /// ```
    #[inline]
    #[stable]
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for working with references
    /////////////////////////////////////////////////////////////////////////

    /// Convert from `Option<T>` to `Option<&T>`
    ///
    /// # Example
    ///
    /// Convert an `Option<String>` into an `Option<int>`, preserving the original.
    /// The `map` method takes the `self` argument by value, consuming the original,
    /// so this technique uses `as_ref` to first take an `Option` to a reference
    /// to the value inside the original.
    ///
    /// ```
    /// let num_as_str: Option<String> = Some("10".to_string());
    /// // First, cast `Option<String>` to `Option<&String>` with `as_ref`,
    /// // then consume *that* with `map`, leaving `num_as_str` on the stack.
    /// let num_as_int: Option<uint> = num_as_str.as_ref().map(|n| n.len());
    /// println!("still can print num_as_str: {}", num_as_str);
    /// ```
    #[inline]
    #[stable]
    pub fn as_ref<'r>(&'r self) -> Option<&'r T> {
        match *self {
            Some(ref x) => Some(x),
            None => None
        }
    }

    /// Convert from `Option<T>` to `Option<&mut T>`
    ///
    /// # Example
    ///
    /// ```
    /// let mut x = Some(2u);
    /// match x.as_mut() {
    ///     Some(v) => *v = 42,
    ///     None => {},
    /// }
    /// assert_eq!(x, Some(42u));
    /// ```
    #[inline]
    #[stable]
    pub fn as_mut<'r>(&'r mut self) -> Option<&'r mut T> {
        match *self {
            Some(ref mut x) => Some(x),
            None => None
        }
    }

    /// Convert from `Option<T>` to `&mut [T]` (without copying)
    ///
    /// # Example
    ///
    /// ```
    /// let mut x = Some("Diamonds");
    /// {
    ///     let v = x.as_mut_slice();
    ///     assert!(v == ["Diamonds"]);
    ///     v[0] = "Dirt";
    ///     assert!(v == ["Dirt"]);
    /// }
    /// assert_eq!(x, Some("Dirt"));
    /// ```
    #[inline]
    #[unstable = "waiting for mut conventions"]
    pub fn as_mut_slice<'r>(&'r mut self) -> &'r mut [T] {
        match *self {
            Some(ref mut x) => {
                let result: &mut [T] = slice::mut_ref_slice(x);
                result
            }
            None => {
                let result: &mut [T] = &mut [];
                result
            }
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Getting to contained values
    /////////////////////////////////////////////////////////////////////////

    /// Unwraps an option, yielding the content of a `Some`
    ///
    /// # Panics
    ///
    /// Panics if the value is a `None` with a custom panic message provided by
    /// `msg`.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Some("value");
    /// assert_eq!(x.expect("the world is ending"), "value");
    /// ```
    ///
    /// ```{.should_fail}
    /// let x: Option<&str> = None;
    /// x.expect("the world is ending"); // panics with `world is ending`
    /// ```
    #[inline]
    #[stable]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Some(val) => val,
            None => panic!("{}", msg),
        }
    }

    /// Returns the inner `T` of a `Some(T)`.
    ///
    /// # Panics
    ///
    /// Panics if the self value equals `None`.
    ///
    /// # Safety note
    ///
    /// In general, because this function may panic, its use is discouraged.
    /// Instead, prefer to use pattern matching and handle the `None`
    /// case explicitly.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Some("air");
    /// assert_eq!(x.unwrap(), "air");
    /// ```
    ///
    /// ```{.should_fail}
    /// let x: Option<&str> = None;
    /// assert_eq!(x.unwrap(), "air"); // fails
    /// ```
    #[inline]
    #[stable]
    pub fn unwrap(self) -> T {
        match self {
            Some(val) => val,
            None => panic!("called `Option::unwrap()` on a `None` value"),
        }
    }

    /// Returns the contained value or a default.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(Some("car").unwrap_or("bike"), "car");
    /// assert_eq!(None.unwrap_or("bike"), "bike");
    /// ```
    #[inline]
    #[stable]
    pub fn unwrap_or(self, def: T) -> T {
        match self {
            Some(x) => x,
            None => def
        }
    }

    /// Returns the contained value or computes it from a closure.
    ///
    /// # Example
    ///
    /// ```
    /// let k = 10u;
    /// assert_eq!(Some(4u).unwrap_or_else(|| 2 * k), 4u);
    /// assert_eq!(None.unwrap_or_else(|| 2 * k), 20u);
    /// ```
    #[inline]
    #[stable]
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Some(x) => x,
            None => f()
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Transforming contained values
    /////////////////////////////////////////////////////////////////////////

    /// Maps an `Option<T>` to `Option<U>` by applying a function to a contained value
    ///
    /// # Example
    ///
    /// Convert an `Option<String>` into an `Option<uint>`, consuming the original:
    ///
    /// ```
    /// let num_as_str: Option<String> = Some("10".to_string());
    /// // `Option::map` takes self *by value*, consuming `num_as_str`
    /// let num_as_int: Option<uint> = num_as_str.map(|n| n.len());
    /// ```
    #[inline]
    #[stable]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U> {
        match self {
            Some(x) => Some(f(x)),
            None => None
        }
    }

    /// Applies a function to the contained value or returns a default.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Some("foo");
    /// assert_eq!(x.map_or(42u, |v| v.len()), 3u);
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.map_or(42u, |v| v.len()), 42u);
    /// ```
    #[inline]
    #[stable]
    pub fn map_or<U, F: FnOnce(T) -> U>(self, def: U, f: F) -> U {
        match self {
            Some(t) => f(t),
            None => def
        }
    }

    /// Applies a function to the contained value or computes a default.
    ///
    /// # Example
    ///
    /// ```
    /// let k = 21u;
    ///
    /// let x = Some("foo");
    /// assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 3u);
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 42u);
    /// ```
    #[inline]
    #[stable]
    pub fn map_or_else<U, D: FnOnce() -> U, F: FnOnce(T) -> U>(self, def: D, f: F) -> U {
        match self {
            Some(t) => f(t),
            None => def()
        }
    }

    /// Transforms the `Option<T>` into a `Result<T, E>`, mapping `Some(v)` to
    /// `Ok(v)` and `None` to `Err(err)`.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Some("foo");
    /// assert_eq!(x.ok_or(0i), Ok("foo"));
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.ok_or(0i), Err(0i));
    /// ```
    #[inline]
    #[experimental]
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Some(v) => Ok(v),
            None => Err(err),
        }
    }

    /// Transforms the `Option<T>` into a `Result<T, E>`, mapping `Some(v)` to
    /// `Ok(v)` and `None` to `Err(err())`.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Some("foo");
    /// assert_eq!(x.ok_or_else(|| 0i), Ok("foo"));
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.ok_or_else(|| 0i), Err(0i));
    /// ```
    #[inline]
    #[experimental]
    pub fn ok_or_else<E, F: FnOnce() -> E>(self, err: F) -> Result<T, E> {
        match self {
            Some(v) => Ok(v),
            None => Err(err()),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Iterator constructors
    /////////////////////////////////////////////////////////////////////////

    /// Returns an iterator over the possibly contained value.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Some(4u);
    /// assert_eq!(x.iter().next(), Some(&4));
    ///
    /// let x: Option<uint> = None;
    /// assert_eq!(x.iter().next(), None);
    /// ```
    #[inline]
    #[stable]
    pub fn iter(&self) -> Iter<T> {
        Iter { inner: Item { opt: self.as_ref() } }
    }

    /// Returns a mutable iterator over the possibly contained value.
    ///
    /// # Example
    ///
    /// ```
    /// let mut x = Some(4u);
    /// match x.iter_mut().next() {
    ///     Some(&ref mut v) => *v = 42u,
    ///     None => {},
    /// }
    /// assert_eq!(x, Some(42));
    ///
    /// let mut x: Option<uint> = None;
    /// assert_eq!(x.iter_mut().next(), None);
    /// ```
    #[inline]
    #[unstable = "waiting for iterator conventions"]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut { inner: Item { opt: self.as_mut() } }
    }

    /// Returns a consuming iterator over the possibly contained value.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Some("string");
    /// let v: Vec<&str> = x.into_iter().collect();
    /// assert_eq!(v, vec!["string"]);
    ///
    /// let x = None;
    /// let v: Vec<&str> = x.into_iter().collect();
    /// assert!(v.is_empty());
    /// ```
    #[inline]
    #[stable]
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { inner: Item { opt: self } }
    }

    /////////////////////////////////////////////////////////////////////////
    // Boolean operations on the values, eager and lazy
    /////////////////////////////////////////////////////////////////////////

    /// Returns `None` if the option is `None`, otherwise returns `optb`.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Some(2u);
    /// let y: Option<&str> = None;
    /// assert_eq!(x.and(y), None);
    ///
    /// let x: Option<uint> = None;
    /// let y = Some("foo");
    /// assert_eq!(x.and(y), None);
    ///
    /// let x = Some(2u);
    /// let y = Some("foo");
    /// assert_eq!(x.and(y), Some("foo"));
    ///
    /// let x: Option<uint> = None;
    /// let y: Option<&str> = None;
    /// assert_eq!(x.and(y), None);
    /// ```
    #[inline]
    #[stable]
    pub fn and<U>(self, optb: Option<U>) -> Option<U> {
        match self {
            Some(_) => optb,
            None => None,
        }
    }

    /// Returns `None` if the option is `None`, otherwise calls `f` with the
    /// wrapped value and returns the result.
    ///
    /// # Example
    ///
    /// ```
    /// fn sq(x: uint) -> Option<uint> { Some(x * x) }
    /// fn nope(_: uint) -> Option<uint> { None }
    ///
    /// assert_eq!(Some(2).and_then(sq).and_then(sq), Some(16));
    /// assert_eq!(Some(2).and_then(sq).and_then(nope), None);
    /// assert_eq!(Some(2).and_then(nope).and_then(sq), None);
    /// assert_eq!(None.and_then(sq).and_then(sq), None);
    /// ```
    #[inline]
    #[stable]
    pub fn and_then<U, F: FnOnce(T) -> Option<U>>(self, f: F) -> Option<U> {
        match self {
            Some(x) => f(x),
            None => None,
        }
    }

    /// Returns the option if it contains a value, otherwise returns `optb`.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Some(2u);
    /// let y = None;
    /// assert_eq!(x.or(y), Some(2u));
    ///
    /// let x = None;
    /// let y = Some(100u);
    /// assert_eq!(x.or(y), Some(100u));
    ///
    /// let x = Some(2u);
    /// let y = Some(100u);
    /// assert_eq!(x.or(y), Some(2u));
    ///
    /// let x: Option<uint> = None;
    /// let y = None;
    /// assert_eq!(x.or(y), None);
    /// ```
    #[inline]
    #[stable]
    pub fn or(self, optb: Option<T>) -> Option<T> {
        match self {
            Some(_) => self,
            None => optb
        }
    }

    /// Returns the option if it contains a value, otherwise calls `f` and
    /// returns the result.
    ///
    /// # Example
    ///
    /// ```
    /// fn nobody() -> Option<&'static str> { None }
    /// fn vikings() -> Option<&'static str> { Some("vikings") }
    ///
    /// assert_eq!(Some("barbarians").or_else(vikings), Some("barbarians"));
    /// assert_eq!(None.or_else(vikings), Some("vikings"));
    /// assert_eq!(None.or_else(nobody), None);
    /// ```
    #[inline]
    #[stable]
    pub fn or_else<F: FnOnce() -> Option<T>>(self, f: F) -> Option<T> {
        match self {
            Some(_) => self,
            None => f()
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Misc
    /////////////////////////////////////////////////////////////////////////

    /// Takes the value out of the option, leaving a `None` in its place.
    ///
    /// # Example
    ///
    /// ```
    /// let mut x = Some(2u);
    /// x.take();
    /// assert_eq!(x, None);
    ///
    /// let mut x: Option<uint> = None;
    /// x.take();
    /// assert_eq!(x, None);
    /// ```
    #[inline]
    #[stable]
    pub fn take(&mut self) -> Option<T> {
        mem::replace(self, None)
    }
}

impl<'a, T: Clone, D: Deref<T>> Option<D> {
    /// Maps an Option<D> to an Option<T> by dereffing and cloning the contents of the Option.
    /// Useful for converting an Option<&T> to an Option<T>.
    #[unstable = "recently added as part of collections reform"]
    pub fn cloned(self) -> Option<T> {
        self.map(|t| t.deref().clone())
    }
}

impl<T: Default> Option<T> {
    /// Returns the contained value or a default
    ///
    /// Consumes the `self` argument then, if `Some`, returns the contained
    /// value, otherwise if `None`, returns the default value for that
    /// type.
    ///
    /// # Example
    ///
    /// Convert a string to an integer, turning poorly-formed strings
    /// into 0 (the default value for integers). `from_str` converts
    /// a string to any other type that implements `FromStr`, returning
    /// `None` on error.
    ///
    /// ```
    /// let good_year_from_input = "1909";
    /// let bad_year_from_input = "190blarg";
    /// let good_year = from_str(good_year_from_input).unwrap_or_default();
    /// let bad_year = from_str(bad_year_from_input).unwrap_or_default();
    ///
    /// assert_eq!(1909i, good_year);
    /// assert_eq!(0i, bad_year);
    /// ```
    #[inline]
    #[stable]
    pub fn unwrap_or_default(self) -> T {
        match self {
            Some(x) => x,
            None => Default::default()
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
// Trait implementations
/////////////////////////////////////////////////////////////////////////////

#[unstable = "waiting on the stability of the trait itself"]
impl<T> AsSlice<T> for Option<T> {
    /// Convert from `Option<T>` to `&[T]` (without copying)
    #[inline]
    fn as_slice<'a>(&'a self) -> &'a [T] {
        match *self {
            Some(ref x) => slice::ref_slice(x),
            None => {
                let result: &[_] = &[];
                result
            }
        }
    }
}

#[stable]
impl<T> Default for Option<T> {
    #[stable]
    #[inline]
    #[stable]
    fn default() -> Option<T> { None }
}

/////////////////////////////////////////////////////////////////////////////
// The Option Iterators
/////////////////////////////////////////////////////////////////////////////

#[deriving(Clone)]
struct Item<A> {
    opt: Option<A>
}

impl<A> Iterator<A> for Item<A> {
    #[inline]
    fn next(&mut self) -> Option<A> {
        self.opt.take()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        match self.opt {
            Some(_) => (1, Some(1)),
            None => (0, Some(0)),
        }
    }
}

impl<A> DoubleEndedIterator<A> for Item<A> {
    #[inline]
    fn next_back(&mut self) -> Option<A> {
        self.opt.take()
    }
}

impl<A> ExactSizeIterator<A> for Item<A> {}

/// An iterator over a reference of the contained item in an Option.
#[stable]
pub struct Iter<'a, A: 'a> { inner: Item<&'a A> }

impl<'a, A> Iterator<&'a A> for Iter<'a, A> {
    #[inline]
    fn next(&mut self) -> Option<&'a A> { self.inner.next() }
    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) { self.inner.size_hint() }
}

impl<'a, A> DoubleEndedIterator<&'a A> for Iter<'a, A> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a A> { self.inner.next_back() }
}

impl<'a, A> ExactSizeIterator<&'a A> for Iter<'a, A> {}

#[stable]
impl<'a, A> Clone for Iter<'a, A> {
    fn clone(&self) -> Iter<'a, A> {
        Iter { inner: self.inner.clone() }
    }
}

/// An iterator over a mutable reference of the contained item in an Option.
#[stable]
pub struct IterMut<'a, A: 'a> { inner: Item<&'a mut A> }

impl<'a, A> Iterator<&'a mut A> for IterMut<'a, A> {
    #[inline]
    fn next(&mut self) -> Option<&'a mut A> { self.inner.next() }
    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) { self.inner.size_hint() }
}

impl<'a, A> DoubleEndedIterator<&'a mut A> for IterMut<'a, A> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut A> { self.inner.next_back() }
}

impl<'a, A> ExactSizeIterator<&'a mut A> for IterMut<'a, A> {}

/// An iterator over the item contained inside an Option.
#[stable]
pub struct IntoIter<A> { inner: Item<A> }

impl<A> Iterator<A> for IntoIter<A> {
    #[inline]
    fn next(&mut self) -> Option<A> { self.inner.next() }
    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) { self.inner.size_hint() }
}

impl<A> DoubleEndedIterator<A> for IntoIter<A> {
    #[inline]
    fn next_back(&mut self) -> Option<A> { self.inner.next_back() }
}

impl<A> ExactSizeIterator<A> for IntoIter<A> {}

/////////////////////////////////////////////////////////////////////////////
// FromIterator
/////////////////////////////////////////////////////////////////////////////

#[stable]
impl<A, V: FromIterator<A>> FromIterator<Option<A>> for Option<V> {
    /// Takes each element in the `Iterator`: if it is `None`, no further
    /// elements are taken, and the `None` is returned. Should no `None` occur, a
    /// container with the values of each `Option` is returned.
    ///
    /// Here is an example which increments every integer in a vector,
    /// checking for overflow:
    ///
    /// ```rust
    /// use std::uint;
    ///
    /// let v = vec!(1u, 2u);
    /// let res: Option<Vec<uint>> = v.iter().map(|&x: &uint|
    ///     if x == uint::MAX { None }
    ///     else { Some(x + 1) }
    /// ).collect();
    /// assert!(res == Some(vec!(2u, 3u)));
    /// ```
    #[inline]
    #[stable]
    fn from_iter<I: Iterator<Option<A>>>(iter: I) -> Option<V> {
        // FIXME(#11084): This could be replaced with Iterator::scan when this
        // performance bug is closed.

        struct Adapter<Iter> {
            iter: Iter,
            found_none: bool,
        }

        impl<T, Iter: Iterator<Option<T>>> Iterator<T> for Adapter<Iter> {
            #[inline]
            fn next(&mut self) -> Option<T> {
                match self.iter.next() {
                    Some(Some(value)) => Some(value),
                    Some(None) => {
                        self.found_none = true;
                        None
                    }
                    None => None,
                }
            }
        }

        let mut adapter = Adapter { iter: iter, found_none: false };
        let v: V = FromIterator::from_iter(adapter.by_ref());

        if adapter.found_none {
            None
        } else {
            Some(v)
        }
    }
}
