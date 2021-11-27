// #![warn(clippy::all, clippy::pedantic)]

pub trait TryContinue<T, E>: Iterator<Item = Result<T, E>> {
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
