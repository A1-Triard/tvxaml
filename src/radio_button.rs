use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use std::mem::replace;
use std::ptr::addr_eq;
use std::rc::{self};
use crate::base::{label_width, option_addr_eq};
use crate::template::{Template, NameResolver};

import! { pub radio_group:
    use [obj basic_oop::obj];
}

#[class_unsafe(inherits_Obj)]
pub struct RadioGroup {
    buttons: RefCell<Vec<rc::Weak<dyn IsRadioButton>>>,
}

impl RadioGroup {
    pub fn new() -> Rc<dyn IsRadioGroup> {
        Rc::new(unsafe { Self::new_raw(RADIO_GROUP_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        RadioGroup {
            obj: unsafe { Obj::new_raw(vtable) },
            buttons: RefCell::new(Vec::new()),
        }
    }
}

#[macro_export]
macro_rules! radio_group_template {
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
        $crate::view_template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                $(use $path as $import;)*

                #[serde(default)]
                #[serde(skip_serializing_if="String::is_empty")]
                pub text: String,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}
 
#[macro_export]
macro_rules! radio_group_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        let _ = $this;
        let _ = $instance;
        let _ = $names;
    };
}

radio_group_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="RadioGroup")]
    pub struct RadioGroupTemplate in radio_group_template { }
}

#[typetag::serde(name="RadioGroup")]
impl Template for RadioGroupTemplate {
    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        RadioGroup::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        radio_group_apply_template!(this, instance, names);
    }
}

import! { pub radio_button:
    use [check_box crate::check_box];
}

struct RadioButtonData {
    allow_uncheck: bool,
    group: rc::Weak<dyn IsRadioGroup>,
}

#[class_unsafe(inherits_CheckBox)]
pub struct RadioButton {
    data: RefCell<RadioButtonData>,
    #[non_virt]
    allow_uncheck: fn() -> bool,
    #[non_virt]
    set_allow_uncheck: fn(value: bool),
    #[non_virt]
    group: fn() -> Option<Rc<dyn IsRadioGroup>>,
    #[non_virt]
    set_group: fn(value: Option<&Rc<dyn IsRadioGroup>>),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
    #[over]
    render: (),
    #[over]
    _attach_to_app: (),
    #[over]
    _detach_from_app: (),
    #[over]
    allow_click: (),
    #[over]
    is_checked_changed: (),
}

impl RadioButton {
    pub fn new() -> Rc<dyn IsRadioButton> {
        let res: Rc<dyn IsRadioButton> = Rc::new(unsafe { Self::new_raw(RADIO_BUTTON_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        RadioButton {
            check_box: unsafe { CheckBox::new_raw(vtable) },
            data: RefCell::new(RadioButtonData {
                group: <rc::Weak<RadioGroup>>::new(),
                allow_uncheck: false,
            }),
        }
    }

    pub fn allow_uncheck_impl(this: &Rc<dyn IsRadioButton>) -> bool {
        this.radio_button().data.borrow().allow_uncheck
    }

    pub fn set_allow_uncheck_impl(this: &Rc<dyn IsRadioButton>, value: bool) {
        this.radio_button().data.borrow_mut().allow_uncheck = value;
    }

    pub fn group_impl(this: &Rc<dyn IsRadioButton>) -> Option<Rc<dyn IsRadioGroup>> {
        this.radio_button().data.borrow().group.upgrade()
    }

    pub fn set_group_impl(this: &Rc<dyn IsRadioButton>, value: Option<&Rc<dyn IsRadioGroup>>) {
        let old_group = replace(
            &mut this.radio_button().data.borrow_mut().group,
            value.map_or_else(|| <rc::Weak::<RadioGroup>>::new(), Rc::downgrade)
        ).upgrade();
        if option_addr_eq(old_group.as_ref().map(Rc::as_ptr), value.map(Rc::as_ptr)) { return; }
        if this.app().is_some() {
            if let Some(old_group) = old_group {
                let mut buttons = old_group.radio_group().buttons.borrow_mut();
                let index = buttons.iter().position(|x|
                    addr_eq(Rc::as_ptr(&x.upgrade().unwrap()), Rc::as_ptr(this))
                ).unwrap();
                buttons.swap_remove(index);
            }
            if let Some(new_group) = value {
                let mut buttons = new_group.radio_group().buttons.borrow_mut();
                buttons.push(Rc::downgrade(this));
            }
        }
    }

    pub fn _attach_to_app_impl(this: &Rc<dyn IsView>, value: &Rc<dyn IsApp>) {
        CheckBox::_attach_to_app_impl(this, value);
        let this: Rc<dyn IsRadioButton> = dyn_cast_rc(this.clone()).unwrap();
        if let Some(group) = this.group() {
            let mut buttons = group.radio_group().buttons.borrow_mut();
            buttons.push(Rc::downgrade(&this));
        }
    }

    pub fn _detach_from_app_impl(this: &Rc<dyn IsView>) {
        {
            let this: Rc<dyn IsRadioButton> = dyn_cast_rc(this.clone()).unwrap();
            if let Some(group) = this.group() {
                let mut buttons = group.radio_group().buttons.borrow_mut();
                let index = buttons.iter().position(|x|
                    addr_eq(Rc::as_ptr(&x.upgrade().unwrap()), Rc::as_ptr(&this))
                ).unwrap();
                buttons.swap_remove(index);
            }
        }
        CheckBox::_detach_from_app_impl(this);
    }

    pub fn is_checked_changed_impl(this: &Rc<dyn IsCheckBox>) {
        CheckBox::is_checked_changed_impl(this);
        if !this.is_checked() || this.app().is_none() { return; }
        let this: Rc<dyn IsRadioButton> = dyn_cast_rc(this.clone()).unwrap();
        if let Some(group) = this.group() {
            let buttons = group.radio_group().buttons.borrow().clone();
            for
                button
            in
                buttons.iter()
                    .map(|x| x.upgrade().unwrap())
                    .filter(|x| !addr_eq(Rc::as_ptr(&x), Rc::as_ptr(&this)))
            {
                button.set_is_checked(false);
            }
        }
    }

    pub fn allow_click_impl(this: &Rc<dyn IsCheckBox>) -> bool {
        if !this.is_checked() { return true; }
        let this: Rc<dyn IsRadioButton> = dyn_cast_rc(this.clone()).unwrap();
        this.allow_uncheck()
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, _w: Option<i16>, _h: Option<i16>) -> Vector {
        let this: Rc<dyn IsCheckBox> = dyn_cast_rc(this.clone()).unwrap();
        let text = this.text();
        if text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            Vector { x: label_width(&text).wrapping_add(4), y: 1 }
        }
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, _bounds: Rect) -> Vector {
        let this: Rc<dyn IsCheckBox> = dyn_cast_rc(this.clone()).unwrap();
        let text = this.text();
        if text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            Vector { x: label_width(&text).wrapping_add(4), y: 1 }
        }
    }

    pub fn render_impl(this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        let bounds = this.inner_render_bounds();
        let is_enabled = this.is_enabled();
        let is_focused = this.is_focused(None);
        let is_focused_primary = this.is_focused(Some(true));
        let this: Rc<dyn IsCheckBox> = dyn_cast_rc(this.clone()).unwrap();
        let is_checked = this.is_checked();
        let (color, color_hotkey) = match (is_enabled, is_focused) {
            (true, false) => (this.color(), this.color_hotkey()),
            (true, true) => (this.color_focused(), this.color_focused()),
            (false, false) => (this.color_disabled(), this.color_disabled()),
            (false, true) => (
                (this.color_disabled().0, this.color_focused().1),
                (this.color_disabled().0, this.color_focused().1)
            ),
        };
        rp.text(Point { x: 1, y: 0 }, color, if is_checked { "*" } else { " " });
        rp.text(Point { x: 0, y: 0 }, color, "(");
        rp.text(Point { x: 2, y: 0 }, color, ")");
        let text = this.text();
        if !text.is_empty() {
            rp.text(Point { x: 3, y: 0 }, color, " ");
            rp.label(Point { x: 4, y: 0 }, color, color_hotkey, &text);
            if (label_width(&text) as u16) > (bounds.w() as u16).saturating_sub(4) {
                rp.text(bounds.br_inner(), color, "►");
            }
        }
        if is_focused_primary { rp.cursor(Point { x: 1, y: 0 }); }
    }
}

#[macro_export]
macro_rules! radio_button_template {
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
        $crate::check_box_template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                $(use $path as $import;)*

                #[serde(default)]
                #[serde(skip_serializing_if="String::is_empty")]
                pub group: String,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}
 
#[macro_export]
macro_rules! radio_button_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::check_box_apply_template!($this, $instance, $names);
        {
            use crate::obj_col::ObjColExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::radio_button::IsRadioButton>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            let obj_ref = obj.clone();
            $names.resolve_or_create(
                $this.group.clone(),
                Box::new(move |x| obj.set_group(Some(&$crate::dynamic_cast_dyn_cast_rc(x).unwrap()))),
                Box::new(move || {
                    let group = RadioGroup::new();
                    obj_ref.resources().insert(group.clone());
                    group
                })
            );
        }
    };
}

radio_button_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="RadioButton@Text")]
    pub struct RadioButtonTemplate in radio_button_template { }
}

#[typetag::serde(name="RadioButton")]
impl Template for RadioButtonTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        RadioButton::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        radio_button_apply_template!(this, instance, names);
    }
}
