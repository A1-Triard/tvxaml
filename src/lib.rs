#![feature(iter_advance_by)]
#![feature(trusted_len)]

use basic_oop::{Vtable, class_unsafe, Obj};
use dynamic_cast::dyn_cast_rc;
use int_vec_2d::{Vector, HAlign, VAlign, Rect, Thickness, Point};
use std::iter::{FusedIterator, TrustedLen};
use std::mem::replace;
use std::num::NonZero;
use std::rc::{self, Rc};
use std::cell::{self, RefCell};

struct ViewData {
    parent: rc::Weak<dyn TView>,
    measure_size: Option<(Option<i16>, Option<i16>)>,
    desired_size: Vector,
    arrange_size: Option<Vector>,
    render_rect: Rect,
    min_size: Vector,
    max_size: Vector,
    h_align: Option<HAlign>,
    v_align: Option<VAlign>,
    margin: Thickness,
}

#[class_unsafe(basic_oop::inherited_from_Obj)]
pub struct View {
    __mod__: ::tvxaml,
    data: RefCell<ViewData>,
    #[virt]
    init: fn(),
    #[non_virt]
    parent: fn() -> Option<Rc<dyn TView>>,
    #[non_virt]
    set_parent: fn(value: Option<&Rc<dyn TView>>),
    #[non_virt]
    min_size: fn() -> Vector,
    #[non_virt]
    set_min_size: fn(value: Vector),
    #[non_virt]
    max_size: fn() -> Vector,
    #[non_virt]
    set_max_size: fn(value: Vector),
    #[non_virt]
    h_align: fn() -> Option<HAlign>,
    #[non_virt]
    set_h_align: fn(value: Option<HAlign>),
    #[non_virt]
    v_align: fn() -> Option<VAlign>,
    #[non_virt]
    set_v_align: fn(value: Option<VAlign>),
    #[non_virt]
    margin: fn() -> Thickness,
    #[non_virt]
    set_margin: fn(value: Thickness),
    #[non_virt]
    desired_size: fn() -> Vector,
    #[non_virt]
    render_rect: fn() -> Rect,
    #[non_virt]
    invalidate_measure: fn(),
    #[non_virt]
    measure: fn(w: Option<i16>, h: Option<i16>),
    #[virt]
    measure_override: fn(w: Option<i16>, h: Option<i16>) -> Vector,
    #[non_virt]
    invalidate_arrange: fn(),
    #[non_virt]
    arrange: fn(rect: Rect),
    #[virt]
    arrange_override: fn(size: Vector) -> Vector,
}

impl View {
    pub fn new() -> Rc<dyn TView> {
        Rc::new(unsafe { Self::new_raw(VIEW_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        View {
            obj: unsafe { Obj::new_raw(vtable) },
            data: RefCell::new(ViewData {
                parent: <rc::Weak::<View>>::new(),
                min_size: Vector::null(),
                max_size: Vector { x: -1, y: -1 },
                h_align: None,
                v_align: None,
                margin: Thickness::all(0),
                measure_size: None,
                desired_size: Vector::null(),
                arrange_size: None,
                render_rect: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
            })
        }
    }

    pub fn init_impl(_this: &Rc<dyn TView>) { }

    pub fn parent_impl(this: &Rc<dyn TView>) -> Option<Rc<dyn TView>> {
        this.view().data.borrow().parent.upgrade()
    }

    pub fn set_parent_impl(this: &Rc<dyn TView>, value: Option<&Rc<dyn TView>>) {
        this.view().data.borrow_mut().parent = value.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade);
    }

    pub fn min_size_impl(this: &Rc<dyn TView>) -> Vector {
        this.view().data.borrow().min_size
    }

    pub fn set_min_size_impl(this: &Rc<dyn TView>, value: Vector) {
        this.view().data.borrow_mut().min_size = value;
        this.invalidate_measure();
    }

    pub fn max_size_impl(this: &Rc<dyn TView>) -> Vector {
        this.view().data.borrow().max_size
    }

    pub fn set_max_size_impl(this: &Rc<dyn TView>, value: Vector) {
        this.view().data.borrow_mut().max_size = value;
        this.invalidate_measure();
    }

    pub fn h_align_impl(this: &Rc<dyn TView>) -> Option<HAlign> {
        this.view().data.borrow().h_align
    }

    pub fn set_h_align_impl(this: &Rc<dyn TView>, value: Option<HAlign>) {
        this.view().data.borrow_mut().h_align = value;
        this.invalidate_measure();
    }

    pub fn v_align_impl(this: &Rc<dyn TView>) -> Option<VAlign> {
        this.view().data.borrow().v_align
    }

    pub fn set_v_align_impl(this: &Rc<dyn TView>, value: Option<VAlign>) {
        this.view().data.borrow_mut().v_align = value;
        this.invalidate_measure();
    }

    pub fn margin_impl(this: &Rc<dyn TView>) -> Thickness {
        this.view().data.borrow().margin
    }

    pub fn set_margin_impl(this: &Rc<dyn TView>, value: Thickness) {
        this.view().data.borrow_mut().margin = value;
        this.invalidate_measure();
    }

    pub fn invalidate_measure_impl(this: &Rc<dyn TView>) {
        let mut data = this.view().data.borrow_mut();
        data.measure_size = None;
        data.arrange_size = None;
        this.parent().map(|x| x.invalidate_measure());
    }

    pub fn desired_size_impl(this: &Rc<dyn TView>) -> Vector {
        this.view().data.borrow().desired_size
    }

    pub fn measure_impl(this: &Rc<dyn TView>, w: Option<i16>, h: Option<i16>) {
        let (a_w, a_h) = {
            let this = this.view().data.borrow();
            if Some((w, h)) == this.measure_size { return; }
            let g_w = if this.h_align.is_some() { None } else { w };
            let g_h = if this.v_align.is_some() { None } else { h };
            let a = Vector { x: g_w.unwrap_or(0), y: g_h.unwrap_or(0) };
            let a = this.margin.shrink_rect_size(a);
            let a = a.min(this.max_size).max(this.min_size);
            (g_w.map(|_| a.x), g_h.map(|_| a.y))
        };
        let desired_size = this.measure_override(a_w, a_h);
        {
            let mut this = this.view().data.borrow_mut();
            let desired_size = desired_size.min(this.max_size).max(this.min_size);
            let desired_size = this.margin.expand_rect_size(desired_size);
            this.measure_size = Some((w, h));
            this.desired_size = desired_size;
        }
    }

    pub fn measure_override_impl(_this: &Rc<dyn TView>, _w: Option<i16>, _h: Option<i16>) -> Vector {
        Vector::null()
    }

    pub fn invalidate_arrange_impl(this: &Rc<dyn TView>) {
        this.view().data.borrow_mut().arrange_size = None;
        this.parent().map(|x| x.invalidate_arrange());
    }

    pub fn render_rect_impl(this: &Rc<dyn TView>) -> Rect {
        this.view().data.borrow().render_rect
    }

    pub fn arrange_impl(this: &Rc<dyn TView>, rect: Rect) {
        let render_size = {
            let data = this.view().data.borrow();
            if Some(rect.size) == data.arrange_size {
                data.render_rect.size
            } else {
                let a_size = data.margin.shrink_rect_size(rect.size).min(data.max_size).max(data.min_size);
                let render_size = this.arrange_override(a_size);
                let data = this.view().data.borrow();
                data.margin.expand_rect_size(render_size.min(data.max_size).max(data.min_size))
            }
        };
        let mut this = this.view().data.borrow_mut();
        let h_align = this.h_align.unwrap_or(HAlign::Left);
        let v_align = this.v_align.unwrap_or(VAlign::Top);
        let align = Thickness::align(render_size, rect.size, h_align, v_align);
        this.arrange_size = Some(rect.size);
        this.render_rect = align.shrink_rect(rect)
    }

    pub fn arrange_override_impl(_this: &Rc<dyn TView>, _size: Vector) -> Vector {
        Vector::null()
    }
}

#[class_unsafe(basic_oop::inherited_from_Obj)]
pub struct ViewVec {
    __mod__: ::tvxaml,
    owner: RefCell<rc::Weak<dyn TView>>,
    items: RefCell<Vec<Rc<dyn TView>>>,
    #[non_virt]
    owner: fn() -> Option<Rc<dyn TView>>,
    #[virt]
    init: fn(owner: &Rc<dyn TView>),
    #[virt]
    attach: fn(index: usize),
    #[virt]
    detach: fn(index: usize),
    #[virt]
    changed: fn(),
    #[non_virt]
    iter: fn() -> ViewVecIter,
    #[non_virt]
    at: fn(index: usize) -> Rc<dyn TView>,
    #[non_virt]
    insert: fn(index: usize, element: Rc<dyn TView>),
    #[non_virt]
    remove: fn(index: usize) -> Rc<dyn TView>,
    #[non_virt]
    push: fn(value: Rc<dyn TView>),
    #[non_virt]
    pop: fn() -> Option<Rc<dyn TView>>,
    #[non_virt]
    clear: fn(),
    #[non_virt]
    len: fn() -> usize,
    #[non_virt]
    is_empty: fn() -> bool,
    #[non_virt]
    replace: fn(index: usize, element: Rc<dyn TView>) -> Rc<dyn TView>,
}

impl ViewVec {
    pub fn new() -> Rc<dyn TViewVec> {
        Rc::new(unsafe { Self::new_raw(VIEW_VEC_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        ViewVec {
            obj: unsafe { Obj::new_raw(vtable) },
            owner: RefCell::new(<rc::Weak::<View>>::new()),
            items: RefCell::new(Vec::new())
        }
    }

    pub fn owner_impl(this: &Rc<dyn TViewVec>) -> Option<Rc<dyn TView>> {
        this.view_vec().owner.borrow().upgrade()
    }

    pub fn init_impl(this: &Rc<dyn TViewVec>, owner: &Rc<dyn TView>) {
        this.view_vec().owner.replace(Rc::downgrade(owner));
    }

    pub fn attach_impl(this: &Rc<dyn TViewVec>, index: usize) {
        let owner = this.view_vec().owner.borrow().upgrade();
        let vec = Self::as_vec(this);
        vec[index].set_parent(owner.as_ref());
    }

    pub fn detach_impl(this: &Rc<dyn TViewVec>, index: usize) {
        let vec = Self::as_vec(this);
        vec[index].set_parent(None);
    }

    pub fn changed_impl(_this: &Rc<dyn TViewVec>) { }

    fn as_vec(this: &Rc<dyn TViewVec>) -> cell::Ref<Vec<Rc<dyn TView>>> {
        this.view_vec().items.borrow()
    }

    pub fn at_impl(this: &Rc<dyn TViewVec>, index: usize) -> Rc<dyn TView> {
        let vec = Self::as_vec(this);
        vec[index].clone()
    }

    pub fn insert_impl(this: &Rc<dyn TViewVec>, index: usize, element: Rc<dyn TView>) {
        let mut vec = this.view_vec().items.borrow_mut();
        vec.insert(index, element);
        this.attach(index);
        this.changed();
    }

    pub fn remove_impl(this: &Rc<dyn TViewVec>, index: usize) -> Rc<dyn TView> {
        this.detach(index);
        let mut vec = this.view_vec().items.borrow_mut();
        let old = vec.remove(index);
        this.changed();
        old
    }

    pub fn push_impl(this: &Rc<dyn TViewVec>, value: Rc<dyn TView>) {
        let mut vec = this.view_vec().items.borrow_mut();
        vec.push(value);
        let index = vec.len() - 1;
        this.attach(index);
        this.changed();
    }

    pub fn pop_impl(this: &Rc<dyn TViewVec>) -> Option<Rc<dyn TView>> {
        let len = this.len();
        if len == 0 { return None; }
        let index = len - 1;
        this.detach(index);
        let mut vec = this.view_vec().items.borrow_mut();
        let old = vec.pop().unwrap();
        this.changed();
        Some(old)
    }

    pub fn clear_impl(this: &Rc<dyn TViewVec>) {
        let len = this.len();
        for index in 0 .. len {
            this.detach(index);
        }
        let mut vec = this.view_vec().items.borrow_mut();
        vec.clear();
        this.changed();
    }

    pub fn len_impl(this: &Rc<dyn TViewVec>) -> usize {
        let vec = Self::as_vec(this);
        vec.len()
    }

    pub fn is_empty_impl(this: &Rc<dyn TViewVec>) -> bool {
        let vec = Self::as_vec(this);
        vec.is_empty()
    }

    pub fn replace_impl(this: &Rc<dyn TViewVec>, index: usize, element: Rc<dyn TView>) -> Rc<dyn TView> {
        this.detach(index);
        let mut vec = this.view_vec().items.borrow_mut();
        let old = replace(&mut vec[index], element);
        this.attach(index);
        this.changed();
        old
    }

    pub fn iter_impl(this: &Rc<dyn TViewVec>) -> ViewVecIter {
        let len = this.len();
        ViewVecIter { vec: this.clone(), index: 0, len }
    }
}

pub struct ViewVecIter {
    vec: Rc<dyn TViewVec>,
    index: usize,
    len: usize,
}

impl Iterator for ViewVecIter {
    type Item = Rc<dyn TView>;

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

#[class_unsafe(inherited_from_ViewVec)]
struct PanelChildrenVec {
    __mod__: ::tvxaml,
    #[over]
    changed: (),
}

impl PanelChildrenVec {
    fn new() -> Rc<dyn TPanelChildrenVec> {
        Rc::new(unsafe { Self::new_raw(PANEL_CHILDREN_VEC_VTABLE.as_ptr()) })
    }

    unsafe fn new_raw(vtable: Vtable) -> Self {
        PanelChildrenVec {
            view_vec: unsafe { ViewVec::new_raw(vtable) },
        }
    }

    fn changed_impl(this: &Rc<dyn TViewVec>) {
        ViewVec::changed_impl(this);
        this.owner().map(|x| x.invalidate_measure());
    }
}

#[class_unsafe(inherited_from_View)]
pub struct Panel {
    __mod__: ::tvxaml,
    children: Rc<dyn TViewVec>,
    #[over]
    init: (),
    #[non_virt]
    children: fn() -> Rc<dyn TViewVec>,
}

impl Panel {
    pub fn new() -> Rc<dyn TPanel> {
        Rc::new(unsafe { Self::new_raw(PANEL_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Panel {
            view: unsafe { View::new_raw(vtable) },
            children: dyn_cast_rc(PanelChildrenVec::new()).unwrap()
        }
    }

    pub fn init_impl(this: &Rc<dyn TView>) {
        View::init_impl(this);
        let panel: Rc<dyn TPanel> = dyn_cast_rc(this.clone()).unwrap();
        panel.panel().children.init(this);
    }

    pub fn children_impl(this: &Rc<dyn TPanel>) -> Rc<dyn TViewVec> {
        this.panel().children.clone()
    }
}
