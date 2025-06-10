use either::{Either, Left, Right};
use nonany::NonMaxUsize;
use std::mem::replace;
use std::iter::{Iterator, FusedIterator, Enumerate};
use std::ops::{Index, IndexMut};
use std::slice::{self};
use std::vec::{self};

#[derive(Debug, Clone)]
pub struct RegistryItems<T>(Vec<Either<Option<NonMaxUsize>, T>>);

impl<T> RegistryItems<T> {
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    pub fn iter(&self) -> Iter<T> {
        Iter(self.0.iter().enumerate())
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(self.0.iter_mut().enumerate())
    }
}

impl<'a, T> IntoIterator for &'a RegistryItems<T> {
    type Item = (Handle, &'a T);

    type IntoIter = Iter<'a, T>;
        
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut RegistryItems<T> {
    type Item = (Handle, &'a mut T);

    type IntoIter = IterMut<'a, T>;
        
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T> IntoIterator for RegistryItems<T> {
    type Item = (Handle, T);

    type IntoIter = IntoIter<T>;
        
    fn into_iter(self) -> IntoIter<T> {
        IntoIter(self.0.into_iter().enumerate())
    }
}

#[derive(Debug)]
pub struct Registry<T> {
    items: RegistryItems<T>,
    free_cell: Option<NonMaxUsize>,
}

#[derive(Debug, Clone, Copy)]
pub struct Handle(NonMaxUsize);

impl<T> Registry<T> {
    pub fn new() -> Self {
        Registry {
            items: RegistryItems(Vec::new()),
            free_cell: None,
        }
    }

    pub fn items(&self) -> &RegistryItems<T> { &self.items }

    //pub fn items_mut(&mut self) -> &mut RegistryItems<T> { &mut self.items }

    pub fn insert<R>(&mut self, f: impl FnOnce(Handle) -> (T, R)) -> R {
        if let Some(free_cell) = self.free_cell {
            let handle = Handle(free_cell);
            let (item, res) = f(handle);
            self.free_cell = replace(&mut self.items.0[free_cell.get()], Right(item)).left().unwrap();
            res
        } else {
            let handle = Handle(NonMaxUsize::new(self.items.0.len()).expect("out of memory"));
            let (item, res) = f(handle);
            self.items.0.push(Right(item));
            res
        }
    }

    pub fn remove(&mut self, handle: Handle) -> T {
        let item = replace(&mut self.items.0[handle.0.get()], Left(self.free_cell)).right().unwrap();
        self.free_cell = Some(handle.0);
        item
    }
}

impl<T> Index<Handle> for RegistryItems<T> {
    type Output = T;

    fn index(&self, index: Handle) -> &Self::Output {
        self.0[index.0.get()].as_ref().right().unwrap()
    }
}

impl<T> IndexMut<Handle> for RegistryItems<T> {
    fn index_mut(&mut self, index: Handle) -> &mut Self::Output {
        self.0[index.0.get()].as_mut().right().unwrap()
    }
}

impl<T> Index<Handle> for Registry<T> {
    type Output = T;

    fn index(&self, index: Handle) -> &Self::Output {
        &self.items[index]
    }
}

impl<T> IndexMut<Handle> for Registry<T> {
    fn index_mut(&mut self, index: Handle) -> &mut Self::Output {
        &mut self.items[index]
    }
}

pub struct Iter<'a, T>(Enumerate<slice::Iter<'a, Either<Option<NonMaxUsize>, T>>>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Handle, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some((_, Left(_))) => { },
                Some((i, Right(item))) => return Some((Handle(NonMaxUsize::new(i).unwrap()), item)),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (0, self.0.size_hint().1) }
}

impl<'a, T> FusedIterator for Iter<'a, T> { }

pub struct IterMut<'a, T>(Enumerate<slice::IterMut<'a, Either<Option<NonMaxUsize>, T>>>);

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Handle, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some((_, Left(_))) => { },
                Some((i, Right(item))) => return Some((Handle(NonMaxUsize::new(i).unwrap()), item)),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (0, self.0.size_hint().1) }
}

impl<'a, T> FusedIterator for IterMut<'a, T> { }

pub struct IntoIter<T>(Enumerate<vec::IntoIter<Either<Option<NonMaxUsize>, T>>>);

impl<T> Iterator for IntoIter<T> {
    type Item = (Handle, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some((_, Left(_))) => { },
                Some((i, Right(item))) => return Some((Handle(NonMaxUsize::new(i).unwrap()), item)),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (0, self.0.size_hint().1) }
}

impl<T> FusedIterator for IntoIter<T> { }
