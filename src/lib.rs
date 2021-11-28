#![warn(clippy::all, clippy::pedantic)]

//! `try-continue` provides one method, [`try_continue`](`TryContinue::try_continue`),
//! which allows you to work with iterators of type `Result<T, _>`, as if they were
//! simply iterators of type `T`. this is is implemented for all iterators providing
//! a `Result`. This is particularly useful if you need to map to a fallible function,
//! and would like to continue using the iterator API to process the elements, but still
//! know if the mapped function fails.
//!
//! For instance, consider a simple parser where you are provided a list of integers as
//! strings, and you would like to count all the strings that hold even numbers. If you
//! wanted to work with the iterator API exclusively, it may get a bit cumbersome to pass
//! along the `Result` if an element failed to parse. Worse, doing so may preclude you
//! from using methods such as [`Iterator::count`], as this would actually attempt to
//! count the `Result`s, forcing you to re-implement the counting with [`Iterator::fold`].
//! Using the [`try_continue`](`TryContinue::try_continue`) method will allow you to work
//! with an iterator of the parsed numbers directly.
//!
//! ```
//! use std::str::FromStr;
//! use try_continue::TryContinue;
//!
//! fn count_even_number_strings(elements: &[&str]) -> Result<usize, <u8 as FromStr>::Err> {
//!     elements
//!        .iter()
//!        .map(|&s| s.parse::<u8>())
//!        .try_continue(|iter| iter.filter(|n| n % 2 == 0).count())
//! }
//!
//! let num_evens_result = count_even_number_strings(&vec!["1", "2", "3", "24", "28"]);
//! assert_eq!(3, num_evens_result.unwrap());
//!
//! let num_evens_bad_result = count_even_number_strings(&vec!["1", "2", "three", "-4", "28"]);
//! assert!(num_evens_bad_result.is_err());
//! ```

/// Provides the [`TryContinue::try_continue`] method, which allows use of the
/// iterator API after mapping to fallible functions.
pub trait TryContinue<T, E>: Iterator<Item = Result<T, E>> {
    /// Allows one to continue processing an iterator of `Result<T, _>`, as if it were
    /// a `Result<T>`, provided that all of the elements are `Ok`. The iterator will
    /// short-circuit if an `Err` element is encountered.
    ///
    /// This is particularly useful if you need to map to a fallible function, and would
    /// like to continue using the iterator API to process the elements, but still know if
    /// the mapped function fails.
    ///
    /// # Errors
    /// The `Result` will only return an error if the given function returns one.
    ///
    /// # Examples
    ///
    /// ```
    /// use try_continue::TryContinue;
    ///
    /// let elements = vec!["1", "2", "3", "4"];
    /// let total = elements
    ///     .into_iter()
    ///     .map(str::parse::<u8>)
    ///     .try_continue(|iter| iter.sum());
    ///
    /// assert_eq!(10_u8, total.unwrap());
    /// ```
    ///
    /// ```
    /// use try_continue::TryContinue;
    ///
    /// let elements = vec!["1", "2", "three", "4"];
    /// let total = elements
    ///     .into_iter()
    ///     .map(str::parse::<u8>)
    ///     .try_continue(|iter| iter.sum::<u8>());
    ///
    /// assert!(total.is_err());
    /// ```
    fn try_continue<F, R>(self, f: F) -> Result<R, E>
    where
        Self: Sized,
        F: FnOnce(&mut TryContinueIter<Self, E>) -> R,
    {
        let mut iter = TryContinueIter::new(self);
        let iteration_output = f(&mut iter);
        iter.err.map_or(Ok(iteration_output), Err)
    }
}

impl<T, E, I: Iterator<Item = Result<T, E>>> TryContinue<T, E> for I {}

/// The iterator produced by [`TryContinue::try_continue`], which is passed to
/// the given closure. See its docs for more information.
pub struct TryContinueIter<I, E> {
    iter: I,
    err: Option<E>,
}

impl<I, E> TryContinueIter<I, E> {
    fn new(iter: I) -> Self {
        Self { iter, err: None }
    }
}

impl<T, E, I: Iterator<Item = Result<T, E>>> Iterator for TryContinueIter<I, E> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.iter.next()?;
        match res {
            Ok(value) => Some(value),
            Err(err) => {
                self.err = Some(err);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Debug)]
    struct TestError<'a>(&'a str);

    #[test]
    fn test_can_wrap_iterator() {
        let items = vec![1, 2, 3];
        let res = items
            .into_iter()
            .map(|x| -> Result<i32, TestError> { Ok(x) })
            .try_continue(|iter| iter.sum());

        assert_eq!(6, res.unwrap());
    }

    #[test]
    fn test_bubbles_out_error() {
        let items = vec![1, 2, 3];
        let res = items
            .into_iter()
            .map(|_| -> Result<i32, TestError> { Err(TestError("oh no this is bad")) })
            .try_continue(|iter| iter.count());

        assert_eq!(TestError("oh no this is bad"), res.unwrap_err());
    }

    #[test]
    fn test_bubbles_out_error_from_part_way() {
        let items = vec![1, 2, 3];
        let res: Result<i32, TestError> = items
            .into_iter()
            .enumerate()
            .map(|(i, x)| -> Result<i32, TestError> {
                if i == 1 {
                    Err(TestError("oh no this is bad"))
                } else {
                    Ok(x)
                }
            })
            .try_continue(|iter| {
                assert!(!iter.any(|n| n == 3));
                iter.sum()
            });

        assert_eq!(TestError("oh no this is bad"), res.unwrap_err());
    }

    #[test]
    fn test_construction_allows_type_conversions() {
        let items = vec![1, 2, 3];
        let res = items
            .into_iter()
            .map(|x| -> Result<i32, TestError> { Ok(x) })
            // An incorrect implementation would not allow the conversion from i32 to Vec<String>
            .try_continue(|iter| iter.map(|x| x.to_string()).collect::<Vec<String>>());

        assert_eq!(
            vec!("1".to_string(), "2".to_string(), "3".to_string()),
            res.unwrap()
        );
    }
}
