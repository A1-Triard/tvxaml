use basic_oop::{class_unsafe, import, Vtable};
use std::iter::{FusedIterator, TrustedLen};
use std::num::NonZero;
use std::cell::{self, RefCell};

import! { pub obj_col:
    use [obj basic_oop::obj];
    use std::rc::Rc;
}

#[class_unsafe(inherits_Obj)]
pub struct ObjCol {
    items: RefCell<Vec<Rc<dyn IsObj>>>,
    #[non_virt]
    iter: fn() -> ObjColIter,
    #[non_virt]
    at: fn(index: usize) -> Rc<dyn IsObj>,
    #[non_virt]
    insert: fn(element: Rc<dyn IsObj>),
    #[non_virt]
    remove: fn(index: usize) -> Rc<dyn IsObj>,
    #[non_virt]
    clear: fn(),
    #[non_virt]
    len: fn() -> usize,
    #[non_virt]
    is_empty: fn() -> bool,
}

impl ObjCol {
    pub fn new() -> Rc<dyn IsObjCol> {
        Rc::new(unsafe { Self::new_raw(OBJ_COL_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        ObjCol {
            obj: unsafe { Obj::new_raw(vtable) },
            items: RefCell::new(Vec::new()),
        }
    }

    fn as_vec(this: &Rc<dyn IsObjCol>) -> cell::Ref<Vec<Rc<dyn IsObj>>> {
        this.obj_col().items.borrow()
    }

    pub fn at_impl(this: &Rc<dyn IsObjCol>, index: usize) -> Rc<dyn IsObj> {
        let vec = Self::as_vec(this);
        vec[index].clone()
    }

    pub fn insert_impl(this: &Rc<dyn IsObjCol>, element: Rc<dyn IsObj>) {
        let mut vec = this.obj_col().items.borrow_mut();
        vec.push(element);
    }

    pub fn remove_impl(this: &Rc<dyn IsObjCol>, index: usize) -> Rc<dyn IsObj> {
        let mut vec = this.obj_col().items.borrow_mut();
        vec.swap_remove(index)
    }

    pub fn clear_impl(this: &Rc<dyn IsObjCol>) {
        let mut vec = this.obj_col().items.borrow_mut();
        vec.clear();
    }

    pub fn len_impl(this: &Rc<dyn IsObjCol>) -> usize {
        let vec = Self::as_vec(this);
        vec.len()
    }

    pub fn is_empty_impl(this: &Rc<dyn IsObjCol>) -> bool {
        let vec = Self::as_vec(this);
        vec.is_empty()
    }

    pub fn iter_impl(this: &Rc<dyn IsObjCol>) -> ObjColIter {
        ObjColIter { col: this.clone(), index: 0 }
    }
}

pub struct ObjColIter {
    col: Rc<dyn IsObjCol>,
    index: usize,
}

impl Iterator for ObjColIter {
    type Item = Rc<dyn IsObj>;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = ObjCol::as_vec(&self.col);
        if self.index == vec.len() { return None; }
        let item = vec[self.index].clone();
        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.col.len() - self.index;
        (size, Some(size))
    }

    fn count(self) -> usize {
        self.col.len() - self.index
    }

    fn last(self) -> Option<Self::Item> {
        let vec = ObjCol::as_vec(&self.col);
        if self.index == vec.len() { return None; }
        Some(vec[vec.len() - 1].clone())
    }

    fn advance_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        let len = self.col.len();
        let size = len - self.index;
        if n <= size {
            self.index += n;
            Ok(())
        } else {
            self.index = len;
            Err(NonZero::new(n - size).unwrap())
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let vec = ObjCol::as_vec(&self.col);
        let size = vec.len() - self.index;
        if n < size {
            self.index += n;
            let item = vec[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for ObjColIter { }

impl FusedIterator for ObjColIter { }

unsafe impl TrustedLen for ObjColIter { }
