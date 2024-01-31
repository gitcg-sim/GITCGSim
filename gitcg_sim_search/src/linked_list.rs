use std::sync::Arc;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cons<T: Clone>(pub T, pub LinkedList<T>);

/// A singly linked list with pointers based on `std::sync::Arc`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        self.into_iter().count()
    }

    pub fn head(&self) -> Option<T> {
        self.0.as_ref().map(|rc| rc.0.clone())
    }

    pub fn tail(&self) -> Option<LinkedList<T>> {
        self.0.as_ref().map(|rc| rc.1.clone())
    }
}

impl<'a, T: Clone> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;

    type IntoIter = LinkedListIterView<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        LinkedListIterView { ptr: self }
    }
}

pub struct LinkedListIterView<'a, T: Clone> {
    ptr: &'a LinkedList<T>,
}

impl<'a, T: Clone> Iterator for LinkedListIterView<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(cons) = &self.ptr.0 else { return None };
        let Cons(a, b) = cons.as_ref();
        self.ptr = b;
        Some(a)
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! cons {
    ($x: expr, $xs: expr) => {
        $crate::linked_list::LinkedList(Some(::std::sync::Arc::new($crate::linked_list::Cons($x, $xs))))
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! linked_list {
    ($(,)?) => {
        $crate::linked_list::LinkedList(
            None
        )
    };
    ($x: expr $(,)?) => {
        $crate::linked_list::LinkedList(
            Some(::std::sync::Arc::new(
                $crate::linked_list::Cons(
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
mod tests {
    use crate::LinkedList;

    #[test]
    pub fn len() {
        let empty: LinkedList<()> = linked_list![];
        assert_eq!(0, empty.len());
        let ll = linked_list![1, 2, 3];
        assert_eq!(3, ll.len())
    }

    #[test]
    pub fn iterator() {
        let ll = linked_list![45, 10, 30];
        let mut c = 0;
        for x in &ll {
            c += x;
        }
        assert_eq!(85, c)
    }
}
