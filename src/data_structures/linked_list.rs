use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cons<T: Clone>(pub T, pub LinkedList<T>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedList<T: Clone>(pub Option<Arc<Cons<T>>>);

impl<T: Clone> Default for LinkedList<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Clone> LinkedList<T> {
    pub fn decons(&self) -> Option<(T, LinkedList<T>)> {
        self.0.as_ref().map(|rc| (rc.0.clone(), rc.1.clone()))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn len(&self) -> usize {
        self.clone().into_iter().fold(0, |c, _| c + 1)
    }

    pub fn head(&self) -> Option<T> {
        self.0.as_ref().map(|rc| rc.0.clone())
    }

    pub fn tail(&self) -> Option<LinkedList<T>> {
        self.0.as_ref().map(|rc| rc.1.clone())
    }
}

impl<T: Clone> Iterator for LinkedList<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.0 {
            None => None,
            Some(rc) => {
                let x = rc.0.clone();
                *self = rc.1.clone();
                Some(x)
            }
        }
    }
}

#[macro_export]
macro_rules! cons {
    ($x: expr, $xs: expr) => {
        $crate::data_structures::linked_list::LinkedList(Some(::std::sync::Arc::new(
            $crate::data_structures::linked_list::Cons($x, $xs),
        )))
    };
}

#[macro_export]
macro_rules! linked_list {
    ($(,)?) => {
        $crate::data_structures::linked_list::LinkedList(
            None
        )
    };
    ($x: expr $(,)?) => {
        $crate::data_structures::linked_list::LinkedList(
            Some(::std::sync::Arc::new(
                $crate::data_structures::linked_list::Cons(
                    $x, linked_list![]
                )
            ))
        )
    };
    ($x: expr, $($xs: expr),+) => {
        cons!($x, linked_list![$($xs),+])
    };
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_len() {
        let ll = linked_list![1, 2, 3];
        assert_eq!(3, ll.len())
    }

    #[test]
    pub fn test_loop() {
        let ll = linked_list![45, 10, 30];
        let mut c = 0;
        for x in ll {
            c += x;
        }
        assert_eq!(85, c)
    }
}
