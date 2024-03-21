/// Helper for radix sort where the list is sorted by a very short list of possible keys.
/// Moves all elements within the slide that pass the predicate to the right side.
///
/// Postcondition: Let `n` be the return value. `range[..n].iter().all(|x| !pred(x))` and
/// `range[n..].iter().all(|x| pred(x))`.
pub(crate) fn move_to_right<T, F: for<'a> FnMut(&'a T) -> bool>(range: &mut [T], mut pred: F) -> usize {
    if range.is_empty() {
        return 0;
    }

    if range.len() <= 1 {
        return !pred(&range[0]) as usize;
    }

    let n = range.len();
    let mut i = n - 1;
    let mut j = n;
    loop {
        if pred(&range[i]) {
            j -= 1;
            if i < j {
                range.swap(i, j);
                continue;
            }
        }

        if i == 0 {
            break;
        }
        i -= 1;
        continue;
    }
    j
}

#[cfg(test)]
mod test {
    use super::move_to_right;
    use proptest::prelude::*;

    /// Check the postcondition for `move_to_right`, and return `Ok(..)` with the return value above if correct.
    fn check_cutoff<T, F: for<'a> FnMut(&'a T) -> bool>(range: &[T], cutoff: usize, mut pred: F) -> bool {
        range[..cutoff].iter().all(|x| !pred(x)) && range[cutoff..].iter().all(pred)
    }

    #[test]
    fn move_to_right_empty_or_singleton() {
        let mut empty: [usize; 0] = [];
        let mut single: [usize; 1] = [1];
        move_to_right(&mut empty, |_| true);
        move_to_right(&mut single, |_| true);
        assert_eq!([1], single);
        move_to_right(&mut single, |_| false);
        assert_eq!([1], single);
    }

    #[test]
    fn move_to_right_all_identical() {
        for i in 0..10 {
            let mut v: Vec<usize> = (0..i).map(|_| 0).collect();
            let expected = v.clone();
            move_to_right(&mut v, |_| false);
            assert_eq!(expected, v);
            move_to_right(&mut v, |_| true);
            assert_eq!(expected, v);
        }
    }

    #[test]
    fn move_to_right_double() {
        let mut forward: [usize; 2] = [1, 2];
        let mut reverse: [usize; 2] = [2, 1];
        move_to_right(&mut forward, |_| false);
        assert_eq!([1, 2], forward);
        move_to_right(&mut reverse, |_| true);
        assert_eq!([1, 2], forward);
    }

    #[test]
    fn move_to_right_example() {
        let mut list = [50, 2, 8, 15, 5, 3, 20, 9, 100, 1];

        let cutoff = move_to_right(&mut list[2..7], |&x| x >= 10);
        assert!(check_cutoff(&list[2..7], cutoff, |&x| x >= 10));

        let cutoff = move_to_right(&mut list, |&x| x >= 10);
        assert!(check_cutoff(&list, cutoff, |&x| x >= 10));
    }

    proptest! {
        #[test]
        fn move_to_right_same_as_sort(v in any::<Vec<u8>>(), cutoff in any::<u8>()) {
            let mut expected = v.clone();
            let mut actual = v.clone();
            expected.sort_unstable_by_key(|&a| a >= cutoff);
            let j = move_to_right(&mut actual[..], |&a| a >= cutoff);
            assert!(check_cutoff(&expected, j, |&a| a >= cutoff));
            assert!(check_cutoff(&actual, j, |&a| a >= cutoff));
        }
    }

    proptest! {
        #[test]
        fn sort_entire_list_with_move_to_right_greater_than(v in any::<Vec<u8>>()) {
            prop_assume!(!v.is_empty());
            let min_cutoff = v.iter().copied().min().expect("non-empty");
            let max_cutoff = v.iter().copied().max().expect("non-empty");
            let mut expected = v.clone();
            expected.sort_unstable_by_key(|&a| a);
            let mut actual = v.clone();
            let j = actual.len();
            for cutoff in (min_cutoff..=max_cutoff).rev() {
                move_to_right(&mut actual[..j], |&a| a >= cutoff);
            }
            assert_eq!(expected, actual);
        }
    }

    proptest! {
        #[test]
        fn sort_by_bucket_with_move_to_right(v in any::<Vec<u8>>()) {
            fn bucket(a: u8) -> u8 {
                a.min(30) / 10
            }

            prop_assume!(!v.is_empty());
            let mut expected = v.clone();
            expected.sort_unstable_by_key(|&a| bucket(a));
            let mut actual = v.clone();

            let j = move_to_right(&mut actual, |&a| a >= 30);
            let j = move_to_right(&mut actual[..j], |&a| a >= 20);
            move_to_right(&mut actual[..j], |&a| a >= 10);

            let expected_buckets: Vec<_> = expected.iter().map(|&a| bucket(a)).collect();
            let actual_buckets: Vec<_> = actual.iter().map(|&a| bucket(a)).collect();
            assert_eq!(expected_buckets, actual_buckets);
        }
    }
}
