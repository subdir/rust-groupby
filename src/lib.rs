//! GroupBy iterator implemented without use of RefCell.
//!
//! Usage:
//! 
//! ```
//! use groupby::GroupByIterator;
//! for (key, grp) in vec![1,1,1,1,2,3,3,4].into_iter().group_by(|x| x/2).by_ref() {
//!     println!("Key {:?}", key);
//!     for item in grp.take(2) {
//!         println!(" - {:?}", item);
//!     }
//! }
//! ```

use std::mem;
use std::iter::Peekable;


macro_rules! reset_lifetime {
    ( &mut $t:ty, $v:expr ) => (
        unsafe {
            // attach lifetime declared by caller
            mem::transmute::<*mut $t, &mut $t>(
                // lose lifetime
                mem::transmute::<&mut $t, *mut $t>(
                    $v
                )
            )
        }
    );
    ( &$t:ty, $v:expr ) => (
        unsafe {
            // attach lifetime declared by caller
            mem::transmute::<*const $t, &$t>(
                // lose lifetime
                mem::transmute::<&$t, *const $t>(
                    $v
                )
            )
        }
    );
}


pub struct GroupIter<I, F, K> where
    I: Iterator,
    F: Fn(&I::Item) -> K
{
    iter: Peekable<I>,
    key_func: F,
    current_key: Option<K>,
}


impl<I, F, K> Iterator for GroupIter<I, F, K> where
    I: Iterator,
    F: Fn(&I::Item) -> K,
    K: PartialEq
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peek_key() {
            None => None,
            key => {
                if key == self.current_key {
                    self.iter.next()
                } else {
                    None
                }
            }
        }
    }
}


impl<I, F, K> GroupIter<I, F, K> where
    I: Iterator,
    F: Fn(&I::Item) -> K,
    K: PartialEq
{
    fn peek_key(&mut self) -> Option<K> {
        match self.iter.peek() {
            None => None,
            Some(item) => Some((self.key_func)(item))
        }
    }

    fn skip_to_next_key(&mut self) -> bool {
        match self.current_key {
            None => {
                match self.peek_key() {
                    None => false,
                    key => {
                        self.current_key = key;
                        true
                    }
                }
            },
            _ => loop {
                match self.peek_key() {
                    None => { return false },
                    key => {
                        if key == self.current_key {
                            self.iter.next();
                        } else {
                            self.current_key = key;
                            return true;
                        }
                    }
                }
            }
        }
    }
}


pub struct GroupBy<I, F, K> where
    I: Iterator,
    F: Fn(&I::Item) -> K,
{
    group_iter: GroupIter<I, F, K>,
}


impl<I, F, K> GroupBy<I, F, K> where
    I: Iterator,
    F: Fn(&I::Item) -> K,
    K: PartialEq
{
    fn new(iter: I, key_func: F) -> Self {
        GroupBy {
            group_iter: GroupIter {
                iter: iter.peekable(),
                key_func,
                current_key: None,
            }
        }
    }

    pub fn by_ref(&mut self) -> &mut Self {
        self
    }
}


impl<'a, I, F, K> Iterator for &'a mut GroupBy<I, F, K> where
    I: Iterator,
    F: Fn(&I::Item) -> K,
    K: PartialEq
{
    type Item = (&'a K, &'a mut GroupIter<I, F, K>);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.group_iter.skip_to_next_key() {
            None
        } else {
            Some((
                reset_lifetime!(&K, self.group_iter.current_key.as_ref().unwrap()),
                reset_lifetime!(&mut GroupIter<I, F, K>, &mut self.group_iter),
            ))
        }
    }
}


pub trait GroupByIterator {
    fn group_by<F, K>(self, f: F) -> GroupBy<Self, F, K>
        where Self: Sized + Iterator,
              F: Fn(&Self::Item) -> K,
              K: PartialEq
    {
        GroupBy::new(self, f)
    }
}

impl<T> GroupByIterator for T where T: Iterator { }


#[cfg(test)]
mod tests {
    use std::vec::Vec;
    use super::GroupByIterator;

    #[test]
    fn it_works() {
        let mut grp = vec![1,1,1,1,2,3,3,4].into_iter().group_by(|x| x/2);
        assert_eq!(
            Some((0, vec![1,1])),
            grp.by_ref().next().map(|(k, g)| (*k, g.take(2).collect::<Vec<i32>>()))
        );
        assert_eq!(
            Some((1, vec![2,3,3])),
            grp.by_ref().next().map(|(k, g)| (*k, g.collect::<Vec<i32>>()))
        );
        assert_eq!(
            Some((2, vec![4])),
            grp.by_ref().next().map(|(k, g)| (*k, g.take(5).collect::<Vec<i32>>()))
        );
        assert_eq!(
            None,
            grp.by_ref().next().map(|(k, g)| (*k, g.collect::<Vec<i32>>()))
        );
    }
}
