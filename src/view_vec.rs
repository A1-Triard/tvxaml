use basic_oop::{class_unsafe, import, Vtable};
use std::iter::{FusedIterator, TrustedLen};
use std::mem::replace;
use std::num::NonZero;
use std::rc::{self};
use std::cell::{self, RefCell};
use crate::view::{View, ViewExt};

import! { pub view_vec:
    use [obj basic_oop::obj];
    use std::rc::Rc;
    use crate::view::IsView;
}

#[class_unsafe(inherits_Obj)]
pub struct ViewVec {
    owner: RefCell<rc::Weak<dyn IsView>>,
    items: RefCell<Vec<Rc<dyn IsView>>>,
    layout: bool,
    visual: bool,
    #[non_virt]
    owner: fn() -> Option<Rc<dyn IsView>>,
    #[virt]
    init: fn(owner: &Rc<dyn IsView>),
    #[virt]
    attach: fn(index: usize),
    #[virt]
    detach: fn(index: usize),
    #[virt]
    changed: fn(),
    #[non_virt]
    iter: fn() -> ViewVecIter,
    #[non_virt]
    at: fn(index: usize) -> Rc<dyn IsView>,
    #[non_virt]
    insert: fn(index: usize, element: Rc<dyn IsView>),
    #[non_virt]
    remove: fn(index: usize) -> Rc<dyn IsView>,
    #[non_virt]
    push: fn(value: Rc<dyn IsView>),
    #[non_virt]
    pop: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    clear: fn(),
    #[non_virt]
    len: fn() -> usize,
    #[non_virt]
    is_empty: fn() -> bool,
    #[non_virt]
    replace: fn(index: usize, element: Rc<dyn IsView>) -> Rc<dyn IsView>,
}

impl ViewVec {
    pub fn new(layout: bool, visual: bool) -> Rc<dyn IsViewVec> {
        Rc::new(unsafe { Self::new_raw(layout, visual, VIEW_VEC_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(layout: bool, visual: bool, vtable: Vtable) -> Self {
        ViewVec {
            obj: unsafe { Obj::new_raw(vtable) },
            owner: RefCell::new(<rc::Weak::<View>>::new()),
            items: RefCell::new(Vec::new()),
            layout,
            visual,
        }
    }

    pub fn owner_impl(this: &Rc<dyn IsViewVec>) -> Option<Rc<dyn IsView>> {
        this.view_vec().owner.borrow().upgrade()
    }

    pub fn init_impl(this: &Rc<dyn IsViewVec>, owner: &Rc<dyn IsView>) {
        this.view_vec().owner.replace(Rc::downgrade(owner));
    }

    pub fn attach_impl(this: &Rc<dyn IsViewVec>, index: usize) {
        let owner = this.view_vec().owner.borrow().upgrade();
        let vec = Self::as_vec(this);
        let item = &vec[index];
        if this.view_vec().layout {
            item.set_layout_parent(owner.as_ref());
        }
        if this.view_vec().visual {
            item.set_visual_parent(owner.as_ref());
            owner.map(|x| x.add_visual_child(item));
        }
    }

    pub fn detach_impl(this: &Rc<dyn IsViewVec>, index: usize) {
        let vec = Self::as_vec(this);
        let item = &vec[index];
        if this.view_vec().visual {
            let owner = this.view_vec().owner.borrow().upgrade();
            owner.map(|x| x.remove_visual_child(item));
            item.set_visual_parent(None);
        }
        if this.view_vec().layout {
            item.set_layout_parent(None);
        }
    }

    pub fn changed_impl(_this: &Rc<dyn IsViewVec>) { }

    fn as_vec(this: &Rc<dyn IsViewVec>) -> cell::Ref<Vec<Rc<dyn IsView>>> {
        this.view_vec().items.borrow()
    }

    pub fn at_impl(this: &Rc<dyn IsViewVec>, index: usize) -> Rc<dyn IsView> {
        let vec = Self::as_vec(this);
        vec[index].clone()
    }

    pub fn insert_impl(this: &Rc<dyn IsViewVec>, index: usize, element: Rc<dyn IsView>) {
        {
            let mut vec = this.view_vec().items.borrow_mut();
            vec.insert(index, element);
        }
        this.attach(index);
        this.changed();
    }

    pub fn remove_impl(this: &Rc<dyn IsViewVec>, index: usize) -> Rc<dyn IsView> {
        this.detach(index);
        let old = {
            let mut vec = this.view_vec().items.borrow_mut();
            vec.remove(index)
        };
        this.changed();
        old
    }

    pub fn push_impl(this: &Rc<dyn IsViewVec>, value: Rc<dyn IsView>) {
        let index = {
            let mut vec = this.view_vec().items.borrow_mut();
            vec.push(value);
            vec.len() - 1
        };
        this.attach(index);
        this.changed();
    }

    pub fn pop_impl(this: &Rc<dyn IsViewVec>) -> Option<Rc<dyn IsView>> {
        let len = this.len();
        if len == 0 { return None; }
        let index = len - 1;
        this.detach(index);
        let old = {
            let mut vec = this.view_vec().items.borrow_mut();
            vec.pop().unwrap()
        };
        this.changed();
        Some(old)
    }

    pub fn clear_impl(this: &Rc<dyn IsViewVec>) {
        let len = this.len();
        for index in 0 .. len {
            this.detach(index);
        }
        {
            let mut vec = this.view_vec().items.borrow_mut();
            vec.clear();
        }
        this.changed();
    }

    pub fn len_impl(this: &Rc<dyn IsViewVec>) -> usize {
        let vec = Self::as_vec(this);
        vec.len()
    }

    pub fn is_empty_impl(this: &Rc<dyn IsViewVec>) -> bool {
        let vec = Self::as_vec(this);
        vec.is_empty()
    }

    pub fn replace_impl(this: &Rc<dyn IsViewVec>, index: usize, element: Rc<dyn IsView>) -> Rc<dyn IsView> {
        this.detach(index);
        let old = {
            let mut vec = this.view_vec().items.borrow_mut();
            replace(&mut vec[index], element)
        };
        this.attach(index);
        this.changed();
        old
    }

    pub fn iter_impl(this: &Rc<dyn IsViewVec>) -> ViewVecIter {
        let len = this.len();
        ViewVecIter { vec: this.clone(), index: 0, len }
    }
}

pub struct ViewVecIter {
    vec: Rc<dyn IsViewVec>,
    index: usize,
    len: usize,
}

impl Iterator for ViewVecIter {
    type Item = Rc<dyn IsView>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.len { return None; }
        let vec = ViewVec::as_vec(&self.vec);
        let item = vec[self.index].clone();
        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.index;
        (size, Some(size))
    }

    fn count(self) -> usize {
        self.len - self.index
    }

    fn last(self) -> Option<Self::Item> {
        if self.index == self.len { return None; }
        let vec = ViewVec::as_vec(&self.vec);
        Some(vec[self.len - 1].clone())
    }

    fn advance_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        let size = self.len - self.index;
        if n <= size {
            self.index += n;
            Ok(())
        } else {
            self.index = self.len;
            Err(NonZero::new(n - size).unwrap())
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let size = self.len - self.index;
        if n < size {
            self.index += n;
            let vec = ViewVec::as_vec(&self.vec);
            let item = vec[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for ViewVecIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == self.len { return None; }
        self.len -= 1;
        let vec = ViewVec::as_vec(&self.vec);
        Some(vec[self.len].clone())
    }

    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        let size = self.len - self.index;
        if n <= size {
            self.len -= n;
            Ok(())
        } else {
            self.len = self.index;
            Err(NonZero::new(n - size).unwrap())
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let size = self.len - self.index;
        if n < size {
            self.len -= n;
            self.len -= 1;
            let vec = ViewVec::as_vec(&self.vec);
            Some(vec[self.len].clone())
        } else {
            None
        }
    }
}

impl ExactSizeIterator for ViewVecIter { }

impl FusedIterator for ViewVecIter { }

unsafe impl TrustedLen for ViewVecIter { }
