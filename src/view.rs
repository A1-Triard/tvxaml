use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use std::mem::replace;
use std::rc::{self};
use std::cell::RefCell;
use crate::template::Template;
use crate::app::{App, AppExt};

import! { pub view:
    use [obj basic_oop::obj];
    use std::rc::Rc;
    use int_vec_2d::{Vector, HAlign, VAlign, Rect, Thickness, Point};
    use crate::app::TApp;
}

#[derive(Serialize, Deserialize)]
#[serde(rename="Layout")]
pub struct LayoutTemplate { }

#[typetag::serde]
impl Template for LayoutTemplate {
    fn create_instance(&self) -> Rc<dyn TObj> {
        dyn_cast_rc(Layout::new()).unwrap()
    }

    fn apply(&self, _instance: &Rc<dyn TObj>) { }
}

#[class_unsafe(inherits_Obj)]
pub struct Layout {
    owner: RefCell<rc::Weak<dyn TView>>,
    #[non_virt]
    owner: fn() -> Option<Rc<dyn TView>>,
    #[non_virt]
    set_owner: fn(value: Option<&Rc<dyn TView>>),
}

impl Layout {
    pub fn new() -> Rc<dyn TLayout> {
        Rc::new(unsafe { Self::new_raw(LAYOUT_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Layout {
            obj: unsafe { Obj::new_raw(vtable) },
            owner: RefCell::new(<rc::Weak::<View>>::new()),
        }
    }

    pub fn owner_impl(this: &Rc<dyn TLayout>) -> Option<Rc<dyn TView>> {
        this.layout().owner.borrow().upgrade()
    }

    pub fn set_owner_impl(this: &Rc<dyn TLayout>, value: Option<&Rc<dyn TView>>) {
        this.layout().owner.replace(value.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade));
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename="View")]
pub struct ViewTemplate {
    pub layout: Box<dyn Template>,
    pub min_size: Vector,
    pub max_size: Vector,
    pub h_align: Option<HAlign>,
    pub v_align: Option<VAlign>,
    pub margin: Thickness,
}

#[typetag::serde]
impl Template for ViewTemplate {
    fn create_instance(&self) -> Rc<dyn TObj> {
        let obj = View::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn TObj>) {
        let obj: Rc<dyn TView> = dyn_cast_rc(instance.clone()).unwrap();
        obj.set_layout(dyn_cast_rc(self.layout.load_content()).unwrap());
        obj.set_min_size(self.min_size);
        obj.set_max_size(self.max_size);
        obj.set_h_align(self.h_align);
        obj.set_v_align(self.v_align);
        obj.set_margin(self.margin); 
    }
}

struct ViewData {
    layout: Rc<dyn TLayout>,
    layout_parent: rc::Weak<dyn TView>,
    visual_parent: rc::Weak<dyn TView>,
    measure_size: Option<(Option<i16>, Option<i16>)>,
    desired_size: Vector,
    arrange_size: Option<Vector>,
    render_bounds: Rect,
    min_size: Vector,
    max_size: Vector,
    h_align: Option<HAlign>,
    v_align: Option<VAlign>,
    margin: Thickness,
    app: rc::Weak<dyn TApp>,
}

#[class_unsafe(inherits_Obj)]
pub struct View {
    data: RefCell<ViewData>,
    #[virt]
    init: fn(),
    #[non_virt]
    layout: fn() -> Rc<dyn TLayout>,
    #[non_virt]
    set_layout: fn(value: Rc<dyn TLayout>),
    #[non_virt]
    layout_parent: fn() -> Option<Rc<dyn TView>>,
    #[non_virt]
    set_layout_parent: fn(value: Option<&Rc<dyn TView>>),
    #[non_virt]
    visual_parent: fn() -> Option<Rc<dyn TView>>,
    #[non_virt]
    set_visual_parent: fn(value: Option<&Rc<dyn TView>>),
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
    render_bounds: fn() -> Rect,
    #[non_virt]
    inner_render_bounds: fn() -> Rect,
    #[non_virt]
    invalidate_measure: fn(),
    #[non_virt]
    measure: fn(w: Option<i16>, h: Option<i16>),
    #[virt]
    measure_override: fn(w: Option<i16>, h: Option<i16>) -> Vector,
    #[non_virt]
    invalidate_arrange: fn(),
    #[non_virt]
    arrange: fn(bounds: Rect),
    #[virt]
    arrange_override: fn(size: Vector) -> Vector,
    #[non_virt]
    app: fn() -> Option<Rc<dyn TApp>>,
    #[non_virt]
    set_app: fn(value: Option<&Rc<dyn TApp>>),
    #[non_virt]
    invalidate_render: fn(),
    #[non_virt]
    add_visual_child: fn(child: &Rc<dyn TView>),
    #[non_virt]
    remove_visual_child: fn(child: &Rc<dyn TView>),
}

impl View {
    pub fn new() -> Rc<dyn TView> {
        Rc::new(unsafe { Self::new_raw(VIEW_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        View {
            obj: unsafe { Obj::new_raw(vtable) },
            data: RefCell::new(ViewData {
                layout: Layout::new(),
                layout_parent: <rc::Weak::<View>>::new(),
                visual_parent: <rc::Weak::<View>>::new(),
                min_size: Vector::null(),
                max_size: Vector { x: -1, y: -1 },
                h_align: None,
                v_align: None,
                margin: Thickness::all(0),
                measure_size: None,
                desired_size: Vector::null(),
                arrange_size: None,
                render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                app: <rc::Weak::<App>>::new(),
            })
        }
    }

    pub fn init_impl(_this: &Rc<dyn TView>) { }

    pub fn layout_impl(this: &Rc<dyn TView>) -> Rc<dyn TLayout> {
        this.view().data.borrow().layout.clone()
    }

    pub fn set_layout_impl(this: &Rc<dyn TView>, value: Rc<dyn TLayout>) {
        value.set_owner(Some(this));
        let mut data = this.view().data.borrow_mut();
        let old = replace(&mut data.layout, value);
        let parent = data.layout_parent.upgrade();
        old.set_owner(None);
        parent.map(|x| x.invalidate_measure());
    }

    pub fn layout_parent_impl(this: &Rc<dyn TView>) -> Option<Rc<dyn TView>> {
        this.view().data.borrow().layout_parent.upgrade()
    }

    pub fn set_layout_parent_impl(this: &Rc<dyn TView>, value: Option<&Rc<dyn TView>>) {
        let set = value.is_some();
        let layout_parent = &mut this.view().data.borrow_mut().layout_parent;
        let old_parent = replace(
            layout_parent,
            value.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade)
        );
        if set && old_parent.upgrade().is_some() {
            *layout_parent = old_parent;
            panic!("layout parent is already set");
        }
    }

    pub fn visual_parent_impl(this: &Rc<dyn TView>) -> Option<Rc<dyn TView>> {
        this.view().data.borrow().visual_parent.upgrade()
    }

    pub fn set_visual_parent_impl(this: &Rc<dyn TView>, value: Option<&Rc<dyn TView>>) {
        let set = value.is_some();
        let visual_parent = &mut this.view().data.borrow_mut().visual_parent;
        let old_parent = replace(
            visual_parent,
            value.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade)
        );
        if set && old_parent.upgrade().is_some() {
            *visual_parent = old_parent;
            panic!("visual parent is already set");
        }
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
        this.layout_parent().map(|x| x.invalidate_measure());
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
        this.layout_parent().map(|x| x.invalidate_arrange());
    }

    pub fn render_bounds_impl(this: &Rc<dyn TView>) -> Rect {
        this.view().data.borrow().render_bounds
    }

    pub fn inner_render_bounds_impl(this: &Rc<dyn TView>) -> Rect {
        Rect {
            tl: Point { x: 0, y: 0 },
            size: this.margin().shrink_rect(this.render_bounds()).size
        }
    }

    pub fn arrange_impl(this: &Rc<dyn TView>, bounds: Rect) {
        let render_size = {
            let data = this.view().data.borrow();
            if Some(bounds.size) == data.arrange_size {
                data.render_bounds.size
            } else {
                let a_size = data.margin.shrink_rect_size(bounds.size).min(data.max_size).max(data.min_size);
                let render_size = this.arrange_override(a_size);
                let data = this.view().data.borrow();
                data.margin.expand_rect_size(render_size.min(data.max_size).max(data.min_size))
            }
        };
        let render_bounds = {
            let mut data = this.view().data.borrow_mut();
            let h_align = data.h_align.unwrap_or(HAlign::Left);
            let v_align = data.v_align.unwrap_or(VAlign::Top);
            let align = Thickness::align(render_size, bounds.size, h_align, v_align);
            let render_bounds = align.shrink_rect(bounds);
            if render_bounds == data.render_bounds {
                data.arrange_size = Some(bounds.size);
                return;
            }
            render_bounds
        };
        this.invalidate_render();
        let mut data = this.view().data.borrow_mut();
        data.arrange_size = Some(bounds.size);
        data.render_bounds = render_bounds;
        this.invalidate_render();
    }

    pub fn arrange_override_impl(_this: &Rc<dyn TView>, _size: Vector) -> Vector {
        Vector::null()
    }

    pub fn app_impl(this: &Rc<dyn TView>) -> Option<Rc<dyn TApp>> {
        this.view().data.borrow().app.upgrade()
    }

    pub fn set_app_impl(this: &Rc<dyn TView>, value: Option<&Rc<dyn TApp>>) {
        let set = value.is_some();
        let app = &mut this.view().data.borrow_mut().app;
        let old_app = replace(app, value.map_or_else(|| <rc::Weak::<App>>::new(), Rc::downgrade));
        if set && old_app.upgrade().is_some() {
            *app = old_app;
            panic!("app is already set");
        }
    }

    fn invalidate_render_raw(this: &Rc<dyn TView>, rect: Rect) {
        let offset = this.margin().shrink_rect(this.render_bounds()).tl;
        let parent_rect = rect.absolute_with(offset);
        if let Some(app) = this.app() {
            app.invalidate_render(parent_rect);
        } else if let Some(parent) = this.visual_parent() {
            Self::invalidate_render_raw(&parent, parent_rect);
        }
    }

    pub fn invalidate_render_impl(this: &Rc<dyn TView>) {
        Self::invalidate_render_raw(this, this.inner_render_bounds());
    }

    pub fn add_visual_child_impl(_this: &Rc<dyn TView>, child: &Rc<dyn TView>) {
        child.invalidate_render();
    }

    pub fn remove_visual_child_impl(_this: &Rc<dyn TView>, child: &Rc<dyn TView>) {
        child.invalidate_render();
    }
}
