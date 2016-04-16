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


struct GroupIter<I, F, K> where
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


struct GroupBy<I, F, K> where
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
                key_func: key_func,
                current_key: None,
            }
        }
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


#[cfg(test)]
mod tests {
    use super::GroupBy;

    #[test]
    fn it_works() {
        let mut g = GroupBy::new(vec![1,1,1,1,2,3,3].into_iter(), |x| x/2);
        for (key, group) in &mut g {
            println!("group {:?}", key);
            for x in group.take(2) {
                println!("{}", x);
            }
        }
    }
}
