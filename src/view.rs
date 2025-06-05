use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::cmp::min;
use std::mem::replace;
use std::rc::{self};
use std::cell::RefCell;
use crate::template::{Template, Names};
use crate::app::{App, AppExt};
use crate::obj_col::{ObjCol, ObjColExt};

import! { pub layout:
    use [obj basic_oop::obj];
    use std::rc::Rc;
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename="Layout")]
pub struct LayoutTemplate { }

#[typetag::serde(name="Layout")]
impl Template for LayoutTemplate {
    fn create_instance(&self) -> Rc<dyn IsObj> {
        dyn_cast_rc(Layout::new()).unwrap()
    }

    fn apply(&self, _instance: &Rc<dyn IsObj>, _names: &mut Names) { }
}

#[class_unsafe(inherits_Obj)]
pub struct Layout {
    owner: RefCell<rc::Weak<dyn IsView>>,
    #[non_virt]
    owner: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    set_owner: fn(value: Option<&Rc<dyn IsView>>),
}

impl Layout {
    pub fn new() -> Rc<dyn IsLayout> {
        Rc::new(unsafe { Self::new_raw(LAYOUT_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Layout {
            obj: unsafe { Obj::new_raw(vtable) },
            owner: RefCell::new(<rc::Weak::<View>>::new()),
        }
    }

    pub fn owner_impl(this: &Rc<dyn IsLayout>) -> Option<Rc<dyn IsView>> {
        this.layout().owner.borrow().upgrade()
    }

    pub fn set_owner_impl(this: &Rc<dyn IsLayout>, value: Option<&Rc<dyn IsView>>) {
        this.layout().owner.replace(value.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade));
    }
}

import! { pub view:
    use [obj basic_oop::obj];
    use std::rc::Rc;
    use int_vec_2d::{Vector, HAlign, VAlign, Rect, Thickness, Point};
    use crate::app::IsApp;
    use crate::obj_col::IsObjCol;
    use crate::render_port::RenderPort;
}

fn is_false(value: &bool) -> bool {
    !value
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
#[serde(rename="HAlign")]
enum HAlignSurrogate { Left, Center, Right, Stretch }

fn serialize_h_align<S>(value: &Option<Option<HAlign>>, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let surrogate = match value {
        Some(Some(HAlign::Left)) => Some(HAlignSurrogate::Left),
        Some(Some(HAlign::Center)) => Some(HAlignSurrogate::Center),
        Some(Some(HAlign::Right)) => Some(HAlignSurrogate::Right),
        Some(None) => Some(HAlignSurrogate::Stretch),
        None => None,
    };
    surrogate.serialize(s)
}

fn deserialize_h_align<'de, D>(d: D) -> Result<Option<Option<HAlign>>, D::Error> where D: Deserializer<'de> {
    let surrogate: Option<HAlignSurrogate> = Deserialize::deserialize(d)?;
    Ok(match surrogate {
        Some(HAlignSurrogate::Left) => Some(Some(HAlign::Left)),
        Some(HAlignSurrogate::Center) => Some(Some(HAlign::Center)),
        Some(HAlignSurrogate::Right) => Some(Some(HAlign::Right)),
        Some(HAlignSurrogate::Stretch) => Some(None),
        None => None,
    })
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
#[serde(rename="VAlign")]
enum VAlignSurrogate { Top, Center, Bottom, Stretch }

fn serialize_v_align<S>(value: &Option<Option<VAlign>>, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let surrogate = match value {
        Some(Some(VAlign::Top)) => Some(VAlignSurrogate::Top),
        Some(Some(VAlign::Center)) => Some(VAlignSurrogate::Center),
        Some(Some(VAlign::Bottom)) => Some(VAlignSurrogate::Bottom),
        Some(None) => Some(VAlignSurrogate::Stretch),
        None => None,
    };
    surrogate.serialize(s)
}

fn deserialize_v_align<'de, D>(d: D) -> Result<Option<Option<VAlign>>, D::Error> where D: Deserializer<'de> {
    let surrogate: Option<VAlignSurrogate> = Deserialize::deserialize(d)?;
    Ok(match surrogate {
        Some(VAlignSurrogate::Top) => Some(Some(VAlign::Top)),
        Some(VAlignSurrogate::Center) => Some(Some(VAlign::Center)),
        Some(VAlignSurrogate::Bottom) => Some(Some(VAlign::Bottom)),
        Some(VAlignSurrogate::Stretch) => Some(None),
        None => None,
    })
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename="View")]
pub struct ViewTemplate {
    #[serde(default)]
    #[serde(skip_serializing_if="is_false")]
    pub is_name_scope: bool,
    #[serde(default)]
    #[serde(skip_serializing_if="String::is_empty")]
    pub name: String,
    #[serde(default)]
    #[serde(skip_serializing_if="Vec::is_empty")]
    pub resources: Vec<Box<dyn Template>>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub layout: Option<Box<dyn Template>>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub min_size: Option<Vector>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub max_size: Option<Vector>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(serialize_with="serialize_h_align")]
    #[serde(deserialize_with="deserialize_h_align")]
    pub h_align: Option<Option<HAlign>>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(serialize_with="serialize_v_align")]
    #[serde(deserialize_with="deserialize_v_align")]
    pub v_align: Option<Option<VAlign>>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub margin: Option<Thickness>,
    #[serde(default)]
    #[serde(skip_serializing_if="String::is_empty")]
    pub focus_tab: String,
}

#[typetag::serde(name="View")]
impl Template for ViewTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = View::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut Names) {
        let obj: Rc<dyn IsView> = dyn_cast_rc(instance.clone()).unwrap();
        for resource in &self.resources {
            obj.resources().insert(resource.load_content(names));
        }
        self.layout.as_ref().map(|x| obj.set_layout(dyn_cast_rc(x.load_content(names)).unwrap()));
        self.min_size.map(|x| obj.set_min_size(x));
        self.max_size.map(|x| obj.set_max_size(x));
        self.h_align.map(|x| obj.set_h_align(x));
        self.v_align.map(|x| obj.set_v_align(x));
        self.margin.map(|x| obj.set_margin(x)); 
        {
            let obj = obj.clone();
            names.resolve(
                self.focus_tab.clone(),
                Box::new(move |x| obj.set_focus_tab(Some(&dyn_cast_rc(x).unwrap())))
            );
        }
    }
}

struct ViewData {
    resources: Rc<dyn IsObjCol>,
    layout: Rc<dyn IsLayout>,
    layout_parent: rc::Weak<dyn IsView>,
    visual_parent: rc::Weak<dyn IsView>,
    measure_size: Option<(Option<i16>, Option<i16>)>,
    desired_size: Vector,
    arrange_size: Option<Vector>,
    render_bounds: Rect,
    real_render_bounds: Rect,
    min_size: Vector,
    max_size: Vector,
    h_align: Option<HAlign>,
    v_align: Option<VAlign>,
    margin: Thickness,
    app: rc::Weak<dyn IsApp>,
    focus_tab: rc::Weak<dyn IsView>,
}

#[class_unsafe(inherits_Obj)]
pub struct View {
    data: RefCell<ViewData>,
    #[virt]
    init: fn(),
    #[non_virt]
    resources: fn() -> Rc<dyn IsObjCol>,
    #[non_virt]
    layout: fn() -> Rc<dyn IsLayout>,
    #[non_virt]
    set_layout: fn(value: Rc<dyn IsLayout>),
    #[non_virt]
    layout_parent: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    set_layout_parent: fn(value: Option<&Rc<dyn IsView>>),
    #[non_virt]
    visual_parent: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    set_visual_parent: fn(value: Option<&Rc<dyn IsView>>),
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
    focus_tab: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    set_focus_tab: fn(value: Option<&Rc<dyn IsView>>),
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
    arrange_override: fn(bounds: Rect) -> Vector,
    #[non_virt]
    app: fn() -> Option<Rc<dyn IsApp>>,
    #[non_virt]
    set_app: fn(value: Option<&Rc<dyn IsApp>>),
    #[non_virt]
    invalidate_render: fn(),
    #[non_virt]
    add_visual_child: fn(child: &Rc<dyn IsView>),
    #[non_virt]
    remove_visual_child: fn(child: &Rc<dyn IsView>),
    #[virt]
    visual_children_count: fn() -> usize,
    #[virt]
    visual_child: fn(index: usize) -> Rc<dyn IsView>,
    #[virt]
    render: fn(rp: &mut RenderPort),
}

impl View {
    pub fn new() -> Rc<dyn IsView> {
        Rc::new(unsafe { Self::new_raw(VIEW_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        View {
            obj: unsafe { Obj::new_raw(vtable) },
            data: RefCell::new(ViewData {
                resources: ObjCol::new(),
                layout: Layout::new(),
                layout_parent: <rc::Weak::<View>>::new(),
                visual_parent: <rc::Weak::<View>>::new(),
                min_size: Vector::null(),
                max_size: Vector { x: -1, y: -1 },
                h_align: None,
                v_align: None,
                margin: Thickness::all(0),
                focus_tab: <rc::Weak::<View>>::new(),
                measure_size: None,
                desired_size: Vector::null(),
                arrange_size: None,
                render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                real_render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                app: <rc::Weak::<App>>::new(),
            })
        }
    }

    pub fn init_impl(_this: &Rc<dyn IsView>) { }

    pub fn resources_impl(this: &Rc<dyn IsView>) -> Rc<dyn IsObjCol> {
        this.view().data.borrow().resources.clone()
    }

    pub fn layout_impl(this: &Rc<dyn IsView>) -> Rc<dyn IsLayout> {
        this.view().data.borrow().layout.clone()
    }

    pub fn set_layout_impl(this: &Rc<dyn IsView>, value: Rc<dyn IsLayout>) {
        value.set_owner(Some(this));
        let (old, parent) = {
            let mut data = this.view().data.borrow_mut();
            let old = replace(&mut data.layout, value);
            let parent = data.layout_parent.upgrade();
            (old, parent)
        };
        old.set_owner(None);
        parent.map(|x| x.invalidate_measure());
    }

    pub fn layout_parent_impl(this: &Rc<dyn IsView>) -> Option<Rc<dyn IsView>> {
        this.view().data.borrow().layout_parent.upgrade()
    }

    pub fn set_layout_parent_impl(this: &Rc<dyn IsView>, value: Option<&Rc<dyn IsView>>) {
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

    pub fn visual_parent_impl(this: &Rc<dyn IsView>) -> Option<Rc<dyn IsView>> {
        this.view().data.borrow().visual_parent.upgrade()
    }

    pub fn set_visual_parent_impl(this: &Rc<dyn IsView>, value: Option<&Rc<dyn IsView>>) {
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

    pub fn min_size_impl(this: &Rc<dyn IsView>) -> Vector {
        this.view().data.borrow().min_size
    }

    pub fn set_min_size_impl(this: &Rc<dyn IsView>, value: Vector) {
        this.view().data.borrow_mut().min_size = value;
        this.invalidate_measure();
    }

    pub fn max_size_impl(this: &Rc<dyn IsView>) -> Vector {
        this.view().data.borrow().max_size
    }

    pub fn set_max_size_impl(this: &Rc<dyn IsView>, value: Vector) {
        this.view().data.borrow_mut().max_size = value;
        this.invalidate_measure();
    }

    pub fn h_align_impl(this: &Rc<dyn IsView>) -> Option<HAlign> {
        this.view().data.borrow().h_align
    }

    pub fn set_h_align_impl(this: &Rc<dyn IsView>, value: Option<HAlign>) {
        this.view().data.borrow_mut().h_align = value;
        this.invalidate_measure();
    }

    pub fn v_align_impl(this: &Rc<dyn IsView>) -> Option<VAlign> {
        this.view().data.borrow().v_align
    }

    pub fn set_v_align_impl(this: &Rc<dyn IsView>, value: Option<VAlign>) {
        this.view().data.borrow_mut().v_align = value;
        this.invalidate_measure();
    }

    pub fn margin_impl(this: &Rc<dyn IsView>) -> Thickness {
        this.view().data.borrow().margin
    }

    pub fn set_margin_impl(this: &Rc<dyn IsView>, value: Thickness) {
        this.view().data.borrow_mut().margin = value;
        this.invalidate_measure();
    }

    pub fn focus_tab_impl(this: &Rc<dyn IsView>) -> Option<Rc<dyn IsView>> {
        this.view().data.borrow().focus_tab.upgrade()
    }

    pub fn set_focus_tab_impl(this: &Rc<dyn IsView>, value: Option<&Rc<dyn IsView>>) {
        this.view().data.borrow_mut().focus_tab
            = value.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade);
    }

    pub fn invalidate_measure_impl(this: &Rc<dyn IsView>) {
        {
            let mut data = this.view().data.borrow_mut();
            data.measure_size = None;
            data.arrange_size = None;
        }
        this.layout_parent().map(|x| x.invalidate_measure());
    }

    pub fn desired_size_impl(this: &Rc<dyn IsView>) -> Vector {
        this.view().data.borrow().desired_size
    }

    pub fn measure_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) {
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
            let desired_size = Vector {
                x: w.map_or(desired_size.x, |w| min(w as u16, desired_size.x as u16) as i16),
                y: h.map_or(desired_size.y, |h| min(h as u16, desired_size.y as u16) as i16),
            };
            this.measure_size = Some((w, h));
            this.desired_size = desired_size;
        }
    }

    pub fn measure_override_impl(_this: &Rc<dyn IsView>, _w: Option<i16>, _h: Option<i16>) -> Vector {
        Vector::null()
    }

    pub fn invalidate_arrange_impl(this: &Rc<dyn IsView>) {
        this.view().data.borrow_mut().arrange_size = None;
        this.layout_parent().map(|x| x.invalidate_arrange());
    }

    pub fn render_bounds_impl(this: &Rc<dyn IsView>) -> Rect {
        this.view().data.borrow().render_bounds
    }

    pub fn inner_render_bounds_impl(this: &Rc<dyn IsView>) -> Rect {
        Rect {
            tl: Point { x: 0, y: 0 },
            size: this.view().data.borrow().real_render_bounds.size
        }
    }

    pub fn arrange_impl(this: &Rc<dyn IsView>, bounds: Rect) {
        let render_size = {
            let data = this.view().data.borrow();
            if Some(bounds.size) == data.arrange_size {
                data.render_bounds.size
            } else {
                let a_size = data.margin.shrink_rect_size(bounds.size).min(data.max_size).max(data.min_size);
                let render_size = this.arrange_override(Rect { tl: Point { x: 0, y: 0 }, size: a_size });
                let data = this.view().data.borrow();
                data.margin.expand_rect_size(render_size.min(data.max_size).max(data.min_size)).min(bounds.size)
            }
        };
        let (render_bounds, real_render_bounds) = {
            let mut data = this.view().data.borrow_mut();
            let h_align = data.h_align.unwrap_or(HAlign::Left);
            let v_align = data.v_align.unwrap_or(VAlign::Top);
            let align = Thickness::align(render_size, bounds.size, h_align, v_align);
            let render_bounds = align.shrink_rect(bounds);
            let real_render_bounds = data.margin.shrink_rect(render_bounds);
            if real_render_bounds == data.real_render_bounds {
                data.arrange_size = Some(bounds.size);
                data.render_bounds = render_bounds;
                return;
            }
            (render_bounds, real_render_bounds)
        };
        this.invalidate_render();
        {
            let mut data = this.view().data.borrow_mut();
            data.arrange_size = Some(bounds.size);
            data.render_bounds = render_bounds;
            data.real_render_bounds = real_render_bounds;
        }
        this.invalidate_render();
    }

    pub fn arrange_override_impl(_this: &Rc<dyn IsView>, _bounds: Rect) -> Vector {
        Vector::null()
    }

    pub fn app_impl(this: &Rc<dyn IsView>) -> Option<Rc<dyn IsApp>> {
        this.view().data.borrow().app.upgrade()
    }

    pub fn set_app_impl(this: &Rc<dyn IsView>, value: Option<&Rc<dyn IsApp>>) {
        let set = value.is_some();
        let app = &mut this.view().data.borrow_mut().app;
        let old_app = replace(app, value.map_or_else(|| <rc::Weak::<App>>::new(), Rc::downgrade));
        if set && old_app.upgrade().is_some() {
            *app = old_app;
            panic!("app is already set");
        }
    }

    fn invalidate_render_raw(this: &Rc<dyn IsView>, rect: Rect) {
        let offset = this.view().data.borrow().real_render_bounds.tl;
        let parent_rect = rect.absolute_with(offset);
        if let Some(app) = this.app() {
            app.invalidate_render(parent_rect);
        } else if let Some(parent) = this.visual_parent() {
            Self::invalidate_render_raw(&parent, parent_rect);
        }
    }

    pub fn invalidate_render_impl(this: &Rc<dyn IsView>) {
        Self::invalidate_render_raw(this, this.inner_render_bounds());
    }

    pub fn add_visual_child_impl(_this: &Rc<dyn IsView>, child: &Rc<dyn IsView>) {
        child.invalidate_render();
    }

    pub fn remove_visual_child_impl(_this: &Rc<dyn IsView>, child: &Rc<dyn IsView>) {
        child.invalidate_render();
    }

    pub fn visual_children_count_impl(_this: &Rc<dyn IsView>) -> usize {
        0
    }

    pub fn visual_child_impl(_this: &Rc<dyn IsView>, _index: usize) -> Rc<dyn IsView> {
        panic!("visual child index out of bounds")
    }

    pub fn render_impl(_this: &Rc<dyn IsView>, _rp: &mut RenderPort) { }
}
