use basic_oop::{class_unsafe, import, Vtable};
use bitflags::bitflags;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::Unexpected;
use serde::de::Error as de_Error;
use std::cell::RefCell;
use std::cmp::min;
use std::mem::replace;
use std::ptr::addr_eq;
use std::rc::{self};
use std::str::FromStr;
use crate::base::{HAlign, VAlign, option_addr_eq};
use crate::event_handler::EventHandler;
use crate::template::{Template, NameResolver};
use crate::app::{App, AppExt};
use crate::obj_col::ObjCol;

import! { pub layout:
    use [obj basic_oop::obj];
    use std::rc::Rc;
}

#[macro_export]
macro_rules! layout_template {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident in $mod:ident {
            $(use $path:path as $import:ident;)*
            $($(
                $(#[$field_attr:meta])*
                pub $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        $crate::template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                $(use $path as $import;)*
                #[serde(default)]
                #[serde(skip_serializing_if="String::is_empty")]
                pub name: String,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! layout_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        let _ = $this;
        let _ = $instance;
        let _ = $names;
    };
}

layout_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="Layout")]
    pub struct LayoutTemplate in layout_template { }
}

#[typetag::serde(name="Layout")]
impl Template for LayoutTemplate {
    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        Layout::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        layout_apply_template!(this, instance, names);
    }
}

#[class_unsafe(inherits_Obj)]
pub struct Layout {
    owner: RefCell<rc::Weak<dyn IsView>>,
    #[non_virt]
    owner: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    _set_owner: fn(value: Option<&Rc<dyn IsView>>),
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

    pub fn _set_owner_impl(this: &Rc<dyn IsLayout>, value: Option<&Rc<dyn IsView>>) {
        this.layout().owner.replace(value.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade));
    }
}

import! { pub view:
    use [obj basic_oop::obj];
    use std::rc::Rc;
    use crate::base::{Key, Vector, Rect, Thickness, Point};
    use crate::app::IsApp;
    use crate::obj_col::IsObjCol;
    use crate::render_port::RenderPort;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum SecondaryFocusKeys { None, LeftRight, UpDown }

bitflags! {
    #[derive(Default)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PrePostProcess: u8 {
        const PRE_PROCESS = 1 << 0;
        const POST_PROCESS = 1 << 1;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum ViewHAlign { Left, Center, Right, Stretch }

impl From<ViewHAlign> for Option<HAlign> {
    fn from(a: ViewHAlign) -> Option<HAlign> {
        match a {
            ViewHAlign::Left => Some(HAlign::Left),
            ViewHAlign::Center => Some(HAlign::Center),
            ViewHAlign::Right => Some(HAlign::Right),
            ViewHAlign::Stretch => None,
        }
    }
}

impl From<Option<HAlign>> for ViewHAlign {
    fn from(a: Option<HAlign>) -> ViewHAlign {
        match a {
            Some(HAlign::Left) => ViewHAlign::Left,
            Some(HAlign::Center) => ViewHAlign::Center,
            Some(HAlign::Right) => ViewHAlign::Right,
            None => ViewHAlign::Stretch,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum ViewVAlign { Top, Center, Bottom, Stretch }

impl From<ViewVAlign> for Option<VAlign> {
    fn from(a: ViewVAlign) -> Option<VAlign> {
        match a {
            ViewVAlign::Top => Some(VAlign::Top),
            ViewVAlign::Center => Some(VAlign::Center),
            ViewVAlign::Bottom => Some(VAlign::Bottom),
            ViewVAlign::Stretch => None,
        }
    }
}

impl From<Option<VAlign>> for ViewVAlign {
    fn from(a: Option<VAlign>) -> ViewVAlign {
        match a {
            Some(VAlign::Top) => ViewVAlign::Top,
            Some(VAlign::Center) => ViewVAlign::Center,
            Some(VAlign::Bottom) => ViewVAlign::Bottom,
            None => ViewVAlign::Stretch,
        }
    }
}

#[doc(hidden)]
pub fn is_false(b: &bool) -> bool { !b }

#[derive(Serialize, Deserialize)]
#[serde(rename="OptionalI16")]
enum OptionalI16NHRSurrogate {
    None,
    Some(i16)
}

impl From<Option<i16>> for OptionalI16NHRSurrogate {
    fn from(value: Option<i16>) -> OptionalI16NHRSurrogate {
        match value {
            None => OptionalI16NHRSurrogate::None,
            Some(x) => OptionalI16NHRSurrogate::Some(x),
        }
    }
}

impl From<OptionalI16NHRSurrogate> for Option<i16> {
    fn from(value: OptionalI16NHRSurrogate) -> Option<i16> {
        match value {
            OptionalI16NHRSurrogate::None => None,
            OptionalI16NHRSurrogate::Some(x) => Some(x),
        }
    }
}

#[doc(hidden)]
pub fn serialize_optional_i16<S>(
    value: &Option<Option<i16>>, serializer: S
) -> Result<S::Ok, S::Error> where S: Serializer {
    if serializer.is_human_readable() {
        let s = value.map(|x| x.map_or_else(|| "None".to_string(), |x| x.to_string()));
        s.serialize(serializer)
    } else {
        value.map(OptionalI16NHRSurrogate::from).serialize(serializer)
    }
}

#[doc(hidden)]
pub fn deserialize_optional_i16<'de, D>(
    deserializer: D
) -> Result<Option<Option<i16>>, D::Error> where D: Deserializer<'de> {
    if deserializer.is_human_readable() {
        let s = <Option<String>>::deserialize(deserializer)?;
        let Some(s) = s else { return Ok(None); };
        if s == "None" { return Ok(Some(None)); }
        Ok(Some(Some(i16::from_str(&s).map_err(|_| D::Error::invalid_value(Unexpected::Str(&s), &"i16"))?)))
    } else {
        let v = <Option<OptionalI16NHRSurrogate>>::deserialize(deserializer)?;
        Ok(v.map(|x| x.into()))
    }
}

#[macro_export]
macro_rules! view_template {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident in $mod:ident {
            $(use $path:path as $import:ident;)*
            $($(
                $(#[$field_attr:meta])*
                pub $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        $crate::template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                use $crate::view::is_false as tvxaml_view_is_false;
                use $crate::view::serialize_optional_i16 as tvxaml_view_serialize_optional_i16;
                use $crate::view::deserialize_optional_i16 as tvxaml_view_deserialize_optional_i16;
                $(use $path as $import;)*

                #[serde(default)]
                #[serde(skip_serializing_if="tvxaml_view_is_false")]
                pub is_name_scope: bool,
                #[serde(default)]
                #[serde(skip_serializing_if="String::is_empty")]
                pub name: String,
                #[serde(default)]
                #[serde(skip_serializing_if="Vec::is_empty")]
                pub resources: Vec<Box<dyn $crate::template::Template>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub layout: Option<Box<dyn $crate::template::Template>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                #[serde(serialize_with="tvxaml_view_serialize_optional_i16")]
                #[serde(deserialize_with="tvxaml_view_deserialize_optional_i16")]
                pub width: Option<Option<i16>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                #[serde(serialize_with="tvxaml_view_serialize_optional_i16")]
                #[serde(deserialize_with="tvxaml_view_deserialize_optional_i16")]
                pub height: Option<Option<i16>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub min_size: Option<$crate::base::Vector>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                #[serde(serialize_with="tvxaml_view_serialize_optional_i16")]
                #[serde(deserialize_with="tvxaml_view_deserialize_optional_i16")]
                pub max_width: Option<Option<i16>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                #[serde(serialize_with="tvxaml_view_serialize_optional_i16")]
                #[serde(deserialize_with="tvxaml_view_deserialize_optional_i16")]
                pub max_height: Option<Option<i16>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub h_align: Option<$crate::view::ViewHAlign>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub v_align: Option<$crate::view::ViewVAlign>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub margin: Option<$crate::base::Thickness>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub allow_focus: Option<bool>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub is_enabled: Option<bool>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub secondary_focus_keys: Option<$crate::view::SecondaryFocusKeys>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! view_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        {
            use $crate::obj_col::ObjColExt;
            use $crate::view::ViewExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::view::IsView>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            for resource in &$this.resources {
                obj.resources().insert(resource.load_content($names));
            }
            $this.layout.as_ref().map(|x|
                obj.set_layout($crate::dynamic_cast_dyn_cast_rc(x.load_content($names)).unwrap())
            );
            $this.width.map(|x| obj.set_width(x));
            $this.height.map(|x| obj.set_height(x));
            $this.min_size.map(|x| obj.set_min_size(x));
            $this.max_width.map(|x| obj.set_max_width(x));
            $this.max_height.map(|x| obj.set_max_height(x));
            $this.h_align.map(|x| obj.set_h_align(x));
            $this.v_align.map(|x| obj.set_v_align(x));
            $this.margin.map(|x| obj.set_margin(x)); 
            $this.allow_focus.map(|x| obj.set_allow_focus(x)); 
            $this.is_enabled.map(|x| obj.set_is_enabled(x));
            $this.secondary_focus_keys.map(|x| obj.set_secondary_focus_keys(x));
        }
    };
}

view_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="View")]
    pub struct ViewTemplate in view_template { }
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
        View::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        view_apply_template!(this, instance, names);
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
    width: Option<i16>,
    height: Option<i16>,
    min_size: Vector,
    max_width: Option<i16>,
    max_height: Option<i16>,
    h_align: ViewHAlign,
    v_align: ViewVAlign,
    margin: Thickness,
    app: rc::Weak<dyn IsApp>,
    allow_focus: bool,
    inherited_is_enabled: bool,
    is_enabled_core: bool,
    changing_is_enabled: bool,
    is_focused_primary: bool,
    is_focused_secondary: bool,
    preview_key_handler: EventHandler<Option<Box<dyn FnMut(Key, &Rc<dyn IsView>) -> bool>>>,
    key_handler: EventHandler<Option<Box<dyn FnMut(Key, &Rc<dyn IsView>) -> bool>>>,
    secondary_focus_keys: SecondaryFocusKeys,
    secondary_focus_root: rc::Weak<dyn IsView>,
}

#[class_unsafe(inherits_Obj)]
pub struct View {
    data: RefCell<ViewData>,
    #[virt]
    _init: fn(),
    #[non_virt]
    resources: fn() -> Rc<dyn IsObjCol>,
    #[non_virt]
    layout: fn() -> Rc<dyn IsLayout>,
    #[non_virt]
    set_layout: fn(value: Rc<dyn IsLayout>),
    #[non_virt]
    layout_parent: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    _set_layout_parent: fn(value: Option<&Rc<dyn IsView>>),
    #[non_virt]
    visual_parent: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    _set_visual_parent: fn(value: Option<&Rc<dyn IsView>>),
    #[non_virt]
    _secondary_focus_root: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    width: fn() -> Option<i16>,
    #[non_virt]
    set_width: fn(value: Option<i16>),
    #[non_virt]
    height: fn() -> Option<i16>,
    #[non_virt]
    set_height: fn(value: Option<i16>),
    #[non_virt]
    min_size: fn() -> Vector,
    #[non_virt]
    set_min_size: fn(value: Vector),
    #[non_virt]
    max_width: fn() -> Option<i16>,
    #[non_virt]
    set_max_width: fn(value: Option<i16>),
    #[non_virt]
    max_height: fn() -> Option<i16>,
    #[non_virt]
    set_max_height: fn(value: Option<i16>),
    #[non_virt]
    h_align: fn() -> ViewHAlign,
    #[non_virt]
    set_h_align: fn(value: ViewHAlign),
    #[non_virt]
    v_align: fn() -> ViewVAlign,
    #[non_virt]
    set_v_align: fn(value: ViewVAlign),
    #[non_virt]
    margin: fn() -> Thickness,
    #[non_virt]
    set_margin: fn(value: Thickness),
    #[non_virt]
    allow_focus: fn() -> bool,
    #[non_virt]
    set_allow_focus: fn(value: bool),
    #[non_virt]
    is_enabled_core: fn() -> bool,
    #[non_virt]
    is_enabled: fn() -> bool,
    #[non_virt]
    set_is_enabled: fn(value: bool),
    #[virt]
    is_enabled_changed: fn(),
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
    #[virt]
    _attach_to_app: fn(value: &Rc<dyn IsApp>),
    #[virt]
    _detach_from_app: fn(),
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
    #[virt]
    is_focused_changed: fn(primary_focus: bool),
    #[non_virt]
    is_focused: fn(primary_focus: Option<bool>) -> bool,
    #[non_virt]
    _set_is_focused: fn(primary_focus: bool, value: bool),
    #[non_virt]
    is_visual_ancestor_of: fn(descendant: Rc<dyn IsView>) -> bool,
    #[virt]
    pre_post_process: fn() -> PrePostProcess,
    #[virt]
    preview_key: fn(key: Key, original_source: &Rc<dyn IsView>) -> bool,
    #[virt]
    key: fn(key: Key, original_source: &Rc<dyn IsView>) -> bool,
    #[virt]
    pre_process_key: fn(key: Key) -> bool,
    #[virt]
    post_process_key: fn(key: Key) -> bool,
    #[non_virt]
    handle_preview_key: fn(handler: Option<Box<dyn FnMut(Key, &Rc<dyn IsView>) -> bool>>),
    #[non_virt]
    handle_key: fn(handler: Option<Box<dyn FnMut(Key, &Rc<dyn IsView>) -> bool>>),
    #[non_virt]
    _raise_key: fn(key: Key) -> bool,
    #[non_virt]
    secondary_focus_keys: fn() -> SecondaryFocusKeys,
    #[non_virt]
    set_secondary_focus_keys: fn(value: SecondaryFocusKeys),
}

impl View {
    pub fn new() -> Rc<dyn IsView> {
        let res: Rc<dyn IsView> = Rc::new(unsafe { Self::new_raw(VIEW_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        View {
            obj: unsafe { Obj::new_raw(vtable) },
            data: RefCell::new(ViewData {
                resources: ObjCol::new(),
                layout: Layout::new(),
                layout_parent: <rc::Weak::<View>>::new(),
                visual_parent: <rc::Weak::<View>>::new(),
                width: None,
                height: None,
                min_size: Vector::null(),
                max_width: None,
                max_height: None,
                h_align: ViewHAlign::Stretch,
                v_align: ViewVAlign::Stretch,
                margin: Thickness::all(0),
                allow_focus: false,
                measure_size: None,
                desired_size: Vector::null(),
                arrange_size: None,
                render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                real_render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                app: <rc::Weak::<App>>::new(),
                inherited_is_enabled: true,
                is_enabled_core: true,
                changing_is_enabled: false,
                is_focused_primary: false,
                is_focused_secondary: false,
                preview_key_handler: Default::default(),
                key_handler: Default::default(),
                secondary_focus_keys: SecondaryFocusKeys::None,
                secondary_focus_root: <rc::Weak::<View>>::new(),
            })
        }
    }

    pub fn _init_impl(_this: &Rc<dyn IsView>) { }

    pub fn resources_impl(this: &Rc<dyn IsView>) -> Rc<dyn IsObjCol> {
        this.view().data.borrow().resources.clone()
    }

    pub fn layout_impl(this: &Rc<dyn IsView>) -> Rc<dyn IsLayout> {
        this.view().data.borrow().layout.clone()
    }

    pub fn set_layout_impl(this: &Rc<dyn IsView>, value: Rc<dyn IsLayout>) {
        let (old, parent) = {
            let mut data = this.view().data.borrow_mut();
            let old = replace(&mut data.layout, value.clone());
            if addr_eq(Rc::as_ptr(&old), Rc::as_ptr(&value)) { return; }
            let parent = data.layout_parent.upgrade();
            (old, parent)
        };
        old._set_owner(None);
        value._set_owner(Some(this));
        parent.map(|x| x.invalidate_measure());
    }

    pub fn is_enabled_core_impl(this: &Rc<dyn IsView>) -> bool {
        this.view().data.borrow().is_enabled_core
    }

    pub fn is_enabled_impl(this: &Rc<dyn IsView>) -> bool {
        let data = this.view().data.borrow();
        data.is_enabled_core && data.inherited_is_enabled
    }

    pub fn set_is_enabled_impl(this: &Rc<dyn IsView>, value: bool) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.is_enabled_core == value { return; }
            assert!(!data.changing_is_enabled);
            data.is_enabled_core = value;
            if !data.inherited_is_enabled { return; }
            data.changing_is_enabled = true;
        }
        this.is_enabled_changed();
        for i in 0 .. this.visual_children_count() {
            let child = this.visual_child(i);
            Self::update_is_enabled(&child, value);
        }
        this.view().data.borrow_mut().changing_is_enabled = false;
    }

    fn update_is_enabled(this: &Rc<dyn IsView>, is_enabled: bool) {
        {
            let mut data = this.view().data.borrow_mut();
            assert!(!data.changing_is_enabled);
            data.inherited_is_enabled = is_enabled;
            if !data.is_enabled_core { return; }
            data.changing_is_enabled = true;
        }
        this.is_enabled_changed();
        for i in 0 .. this.visual_children_count() {
            let child = this.visual_child(i);
            Self::update_is_enabled(&child, is_enabled);
        }
        this.view().data.borrow_mut().changing_is_enabled = false;
    }

    pub fn is_enabled_changed_impl(_this: &Rc<dyn IsView>) { }

    pub fn layout_parent_impl(this: &Rc<dyn IsView>) -> Option<Rc<dyn IsView>> {
        this.view().data.borrow().layout_parent.upgrade()
    }

    pub fn _set_layout_parent_impl(this: &Rc<dyn IsView>, value: Option<&Rc<dyn IsView>>) {
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

    pub fn _set_visual_parent_impl(this: &Rc<dyn IsView>, value: Option<&Rc<dyn IsView>>) {
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

    pub fn _secondary_focus_root_impl(this: &Rc<dyn IsView>) -> Option<Rc<dyn IsView>> {
        this.view().data.borrow().secondary_focus_root.upgrade()
    }

    pub fn width_impl(this: &Rc<dyn IsView>) -> Option<i16> {
        this.view().data.borrow().width
    }

    pub fn set_width_impl(this: &Rc<dyn IsView>, value: Option<i16>) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.width == value { return; }
            data.width = value;
        }
        this.invalidate_measure();
    }

    pub fn height_impl(this: &Rc<dyn IsView>) -> Option<i16> {
        this.view().data.borrow().height
    }

    pub fn set_height_impl(this: &Rc<dyn IsView>, value: Option<i16>) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.height == value { return; }
            data.height = value;
        }
        this.invalidate_measure();
    }

    pub fn min_size_impl(this: &Rc<dyn IsView>) -> Vector {
        this.view().data.borrow().min_size
    }

    pub fn set_min_size_impl(this: &Rc<dyn IsView>, value: Vector) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.min_size == value { return; }
            data.min_size = value;
        }
        this.invalidate_measure();
    }

    pub fn max_width_impl(this: &Rc<dyn IsView>) -> Option<i16> {
        this.view().data.borrow().max_width
    }

    pub fn set_max_width_impl(this: &Rc<dyn IsView>, value: Option<i16>) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.max_width == value { return; }
            data.max_width = value;
        }
        this.invalidate_measure();
    }

    pub fn max_height_impl(this: &Rc<dyn IsView>) -> Option<i16> {
        this.view().data.borrow().max_height
    }

    pub fn set_max_height_impl(this: &Rc<dyn IsView>, value: Option<i16>) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.max_height == value { return; }
            data.max_height = value;
        }
        this.invalidate_measure();
    }

    pub fn h_align_impl(this: &Rc<dyn IsView>) -> ViewHAlign {
        this.view().data.borrow().h_align
    }

    pub fn set_h_align_impl(this: &Rc<dyn IsView>, value: ViewHAlign) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.h_align == value { return; }
            data.h_align = value;
        }
        this.invalidate_measure();
    }

    pub fn v_align_impl(this: &Rc<dyn IsView>) -> ViewVAlign {
        this.view().data.borrow().v_align
    }

    pub fn set_v_align_impl(this: &Rc<dyn IsView>, value: ViewVAlign) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.v_align == value { return; }
            data.v_align = value;
        }
        this.invalidate_measure();
    }

    pub fn margin_impl(this: &Rc<dyn IsView>) -> Thickness {
        this.view().data.borrow().margin
    }

    pub fn set_margin_impl(this: &Rc<dyn IsView>, value: Thickness) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.margin == value { return; }
            data.margin = value;
        }
        this.invalidate_measure();
    }

    pub fn allow_focus_impl(this: &Rc<dyn IsView>) -> bool {
        this.view().data.borrow().allow_focus
    }

    pub fn set_allow_focus_impl(this: &Rc<dyn IsView>, value: bool) {
        this.view().data.borrow_mut().allow_focus = value;
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
        let (a_w, a_h, max_size, min_size) = {
            let this = this.view().data.borrow();
            let max_width = this.width.or(this.max_width);
            let max_height = this.height.or(this.max_height);
            let max_size = Vector { x: max_width.unwrap_or(-1), y: max_height.unwrap_or(-1) };
            let min_size = Vector {
                x: this.width.unwrap_or(this.min_size.x),
                y: this.height.unwrap_or(this.min_size.y),
            };
            if Some((w, h)) == this.measure_size { return; }
            let g_w = if this.h_align != ViewHAlign::Stretch { None } else { w };
            let g_h = if this.v_align != ViewVAlign::Stretch { None } else { h };
            let g_w = g_w.or(max_width);
            let g_h = g_h.or(max_height);
            let a = Vector { x: g_w.unwrap_or(0), y: g_h.unwrap_or(0) };
            let a = this.margin.shrink_rect_size(a);
            let a = a.min(max_size).max(min_size);
            (g_w.map(|_| a.x), g_h.map(|_| a.y), max_size, min_size)
        };
        let desired_size = this.measure_override(a_w, a_h);
        {
            let mut this = this.view().data.borrow_mut();
            let desired_size = desired_size.min(max_size).max(min_size);
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
            let (a_size, max_size, min_size) = {
                let data = this.view().data.borrow();
                let max_width = data.width.or(data.max_width);
                let max_height = data.height.or(data.max_height);
                let max_size = Vector { x: max_width.unwrap_or(-1), y: max_height.unwrap_or(-1) };
                let min_size = Vector {
                    x: data.width.unwrap_or(data.min_size.x),
                    y: data.height.unwrap_or(data.min_size.y),
                };
                if Some(bounds.size) == data.arrange_size {
                    (None, max_size, min_size)
                } else {
                    let a_size = data.margin.shrink_rect_size(bounds.size).min(max_size).max(min_size);
                    let d_size = data.margin.shrink_rect_size(data.desired_size);
                    (Some((a_size, data.h_align, data.v_align, d_size)), max_size, min_size)
                }
            };
            if let Some((a_size, h_align, v_align, desired_size)) = a_size {
                let a_size = Vector {
                    x: if h_align == ViewHAlign::Stretch { a_size.x } else { desired_size.x },
                    y: if v_align == ViewVAlign::Stretch { a_size.y } else { desired_size.y }
                };
                let render_size = this.arrange_override(Rect { tl: Point { x: 0, y: 0 }, size: a_size });
                let data = this.view().data.borrow();
                data.margin.expand_rect_size(render_size.min(max_size).max(min_size)).min(bounds.size)
            } else {
                let data = this.view().data.borrow();
                data.render_bounds.size
            }
        };
        let (render_bounds, real_render_bounds) = {
            let mut data = this.view().data.borrow_mut();
            let h_align = <Option<HAlign>>::from(data.h_align).unwrap_or(HAlign::Left);
            let v_align = <Option<VAlign>>::from(data.v_align).unwrap_or(VAlign::Top);
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

    pub fn _attach_to_app_impl(this: &Rc<dyn IsView>, value: &Rc<dyn IsApp>) {
        this.view().data.borrow_mut().app = Rc::downgrade(value);
        let pre_post_process = this.pre_post_process();
        if pre_post_process.contains(PrePostProcess::PRE_PROCESS) {
            value._add_pre_process(this);
        }
        if pre_post_process.contains(PrePostProcess::POST_PROCESS) {
            value._add_post_process(this);
        }
        for i in 0 .. this.visual_children_count() {
            this.visual_child(i)._attach_to_app(value);
        }
    }

    pub fn _detach_from_app_impl(this: &Rc<dyn IsView>) {
        let app = replace(&mut this.view().data.borrow_mut().app, <rc::Weak::<App>>::new()).upgrade().unwrap();
        let pre_post_process = this.pre_post_process();
        if pre_post_process.contains(PrePostProcess::PRE_PROCESS) {
            app._remove_pre_process(this);
        }
        if pre_post_process.contains(PrePostProcess::POST_PROCESS) {
            app._remove_post_process(this);
        }
        for i in 0 .. this.visual_children_count() {
            this.visual_child(i)._detach_from_app();
        }
    }

    fn invalidate_render_raw(this: &Rc<dyn IsView>, rect: Rect) {
        let rect = rect.intersect(this.inner_render_bounds());
        let offset = this.view().data.borrow().real_render_bounds.tl;
        let parent_rect = rect.absolute_with(offset);
        if let Some(parent) = this.visual_parent() {
            Self::invalidate_render_raw(&parent, parent_rect);
        } else if let Some(app) = this.app() {
            app.invalidate_render(parent_rect);
        }
    }

    pub fn invalidate_render_impl(this: &Rc<dyn IsView>) {
        Self::invalidate_render_raw(this, this.inner_render_bounds());
    }

    pub fn add_visual_child_impl(this: &Rc<dyn IsView>, child: &Rc<dyn IsView>) {
        if let Some(app) = this.app() {
            child._attach_to_app(&app);
        }
        child.invalidate_render();
        let is_enabled = {
            let data = this.view().data.borrow();
            data.is_enabled_core && data.inherited_is_enabled
        };
        if !is_enabled {
            Self::update_is_enabled(child, false);
        }
        if let Some(child_secondary_focus_root) = child.view().data.borrow().secondary_focus_root.upgrade() {
            Self::set_secondary_focus_root(this.clone(), &child_secondary_focus_root);
        }
    }

    pub fn is_visual_ancestor_of_impl(this: &Rc<dyn IsView>, mut descendant: Rc<dyn IsView>) -> bool {
        loop {
            if addr_eq(Rc::as_ptr(&descendant), Rc::as_ptr(this)) {
                return true;
            }
            if let Some(parent) = descendant.visual_parent() {
                descendant = parent;
            } else {
                return false;
            }
        }
    }

    pub fn remove_visual_child_impl(this: &Rc<dyn IsView>, child: &Rc<dyn IsView>) {
        if let Some(child_secondary_focus_root) = child.view().data.borrow().secondary_focus_root.upgrade() {
            Self::reset_secondary_focus_root(this.clone(), &child_secondary_focus_root);
        }
        if let Some(app) = this.app() {
            if let Some(focused) = app.focused(true) {
                if child.is_visual_ancestor_of(focused) {
                    app.focus(None, true);
                }
            }
            if let Some(focused) = app.focused(false) {
                if child.is_visual_ancestor_of(focused) {
                    app.focus(None, false);
                }
            }
        }
        child.invalidate_render();
        let is_enabled = {
            let data = this.view().data.borrow();
            data.is_enabled_core && data.inherited_is_enabled
        };
        if !is_enabled {
            Self::update_is_enabled(child, true);
        }
        child._detach_from_app();
    }

    pub fn visual_children_count_impl(_this: &Rc<dyn IsView>) -> usize {
        0
    }

    pub fn visual_child_impl(_this: &Rc<dyn IsView>, _index: usize) -> Rc<dyn IsView> {
        panic!("visual child index out of bounds")
    }

    pub fn render_impl(_this: &Rc<dyn IsView>, _rp: &mut RenderPort) { }

    pub fn is_focused_changed_impl(_this: &Rc<dyn IsView>, _primary_focus: bool) { }

    pub fn is_focused_impl(this: &Rc<dyn IsView>, primary_focus: Option<bool>) -> bool {
        let data = this.view().data.borrow();
        match primary_focus {
            Some(true) => data.is_focused_primary,
            Some(false) => data.is_focused_secondary,
            None => data.is_focused_primary || data.is_focused_secondary,
        }
    }

    pub fn _set_is_focused_impl(this: &Rc<dyn IsView>, primary_focus: bool, value: bool) {
        {
            let mut data = this.view().data.borrow_mut();
            if primary_focus {
                data.is_focused_primary = value;
            } else {
                data.is_focused_secondary = value;
            }
        }
        this.is_focused_changed(primary_focus);
    }

    pub fn pre_post_process_impl(_this: &Rc<dyn IsView>) -> PrePostProcess {
        PrePostProcess::empty()
    }

    fn raise_preview<F: Fn(&Rc<dyn IsView>) -> bool>(this: &Rc<dyn IsView>, f: F) -> (bool, F) {
        let (handled, f) = if let Some(parent) = this.visual_parent() {
            Self::raise_preview(&parent, f)
        } else {
            (false, f)
        };
        if handled { return (true, f); }
        let handled = f(this);
        (handled, f)
    }

    fn raise(this: &Rc<dyn IsView>, f: impl Fn(&Rc<dyn IsView>) -> bool) -> bool {
        let handled = f(this);
        if handled { return true; }
        this.visual_parent().map_or(false, |x| Self::raise(&x, f))
    }

    pub fn _raise_key_impl(this: &Rc<dyn IsView>, key: Key) -> bool {
        let handled = Self::raise_preview(this, |x| x.preview_key(key, this)).0;
        if handled { return true; }
        Self::raise(this, |x| x.key(key, this))
    }

    pub fn preview_key_impl(this: &Rc<dyn IsView>, key: Key, original_source: &Rc<dyn IsView>) -> bool {
        let mut invoke = this.view().data.borrow_mut().preview_key_handler.begin_invoke();
        let handled = invoke.as_mut().map_or(false, |x| x(key, original_source));
        this.view().data.borrow_mut().preview_key_handler.end_invoke(invoke);
        handled
    }

    pub fn key_impl(this: &Rc<dyn IsView>, key: Key, original_source: &Rc<dyn IsView>) -> bool {
        let mut invoke = this.view().data.borrow_mut().key_handler.begin_invoke();
        let handled = invoke.as_mut().map_or(false, |x| x(key, original_source));
        this.view().data.borrow_mut().key_handler.end_invoke(invoke);
        handled
    }

    pub fn pre_process_key_impl(_this: &Rc<dyn IsView>, _key: Key) -> bool {
        false
    }

    pub fn post_process_key_impl(_this: &Rc<dyn IsView>, _key: Key) -> bool {
        false
    }

    pub fn handle_preview_key_impl(
        this: &Rc<dyn IsView>, 
        handler: Option<Box<dyn FnMut(Key, &Rc<dyn IsView>) -> bool>>
    ) {
        this.view().data.borrow_mut().preview_key_handler.set(handler);
    }

    pub fn handle_key_impl(
        this: &Rc<dyn IsView>, 
        handler: Option<Box<dyn FnMut(Key, &Rc<dyn IsView>) -> bool>>
    ) {
        this.view().data.borrow_mut().key_handler.set(handler);
    }

    pub fn secondary_focus_keys_impl(this: &Rc<dyn IsView>) -> SecondaryFocusKeys {
        this.view().data.borrow().secondary_focus_keys
    }

    pub fn set_secondary_focus_keys_impl(this: &Rc<dyn IsView>, value: SecondaryFocusKeys) {
        {
            let mut data = this.view().data.borrow_mut();
            if data.secondary_focus_keys == value { return; }
            data.secondary_focus_keys = value;
        }
        if value == SecondaryFocusKeys::None {
            Self::reset_secondary_focus_root(this.clone(), this);
        } else {
            Self::set_secondary_focus_root(this.clone(), this);
        }
    }

    fn set_secondary_focus_root(mut view: Rc<dyn IsView>, sfr: &Rc<dyn IsView>) {
        loop {
            let parent = {
                let mut data = view.view().data.borrow_mut();
                data.secondary_focus_root = Rc::downgrade(sfr);
                data.visual_parent.upgrade()
            };
            if let Some(parent) = parent {
                view = parent;
            } else {
                break;
            }
        }
    }

    fn reset_secondary_focus_root(mut view: Rc<dyn IsView>, sfr: &Rc<dyn IsView>) {
        loop {
            let parent = {
                let mut data = view.view().data.borrow_mut();
                let secondary_focus_root = data.secondary_focus_root.upgrade();
                if !option_addr_eq(
                    secondary_focus_root.as_ref().map(Rc::as_ptr),
                    Some(Rc::as_ptr(sfr))
                ) { break; }
                data.secondary_focus_root = <rc::Weak::<View>>::new();
                data.visual_parent.upgrade()
            };
            if let Some(parent) = parent {
                view = parent;
            } else {
                break;
            }
        }
    }
}
