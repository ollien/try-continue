# try-continue

`try-continue` provides one method, [`try_continue`](`https://docs.rs/try-continue/0.1.0/try_continue/trait.TryContinue.html#method.try_continue`),
which allows you to work with iterators of type `Result<T, _>`, as if they were
simply iterators of type `T`. this is is implemented for all iterators providing
a `Result`. This is particularly useful if you need to map to a fallible function,
and would like to continue using the iterator API to process the elements, but still
know if the mapped function fails.

For instance, consider a simple parser where you are provided a list of integers as
strings, and you would like to count all the strings that hold even numbers. If you
wanted to work with the iterator API exclusively, it may get a bit cumbersome to pass
along the `Result` if an element failed to parse. Worse, doing so may preclude you
from using methods such as `Iterator::count`, as this would actually attempt to
count the `Result`s, forcing you to re-implement the counting with `Iterator::fold`.
Using the `try_continue` method will allow you to work with an iterator of the
parsed numbers directly.

```rs
use std::str::FromStr;
use try_continue::TryContinue;

fn count_even_number_strings(elements: &[&str]) -> Result<usize, <u8 as FromStr>::Err> {
    elements
       .iter()
       .map(|&s| s.parse::<u8>())
       .try_continue(|iter| iter.filter(|n| n % 2 == 0).count())
}

let num_evens_result = count_even_number_strings(&vec!["1", "2", "3", "24", "28"]);
assert_eq!(3, num_evens_result.unwrap());

let num_evens_bad_result = count_even_number_strings(&vec!["1", "2", "three", "-4", "28"]);
assert!(num_evens_bad_result.is_err());
```
