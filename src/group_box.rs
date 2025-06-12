use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use crate::template::{Template, NameResolver};

import! { pub group_box:
    use [headered_decorator crate::headered_decorator];
}

struct GroupBoxData {
    double: bool,
    color: (Fg, Bg),
}

#[class_unsafe(inherits_HeaderedDecorator)]
pub struct GroupBox {
    data: RefCell<GroupBoxData>,
    #[non_virt]
    double: fn() -> bool,
    #[non_virt]
    set_double: fn(value: bool),
    #[non_virt]
    color: fn() -> (Fg, Bg),
    #[non_virt]
    set_color: fn(value: (Fg, Bg)),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
    #[over]
    render: (),
    #[over]
    _init: (),
}

impl GroupBox {
    pub fn new() -> Rc<dyn IsGroupBox> {
        let res: Rc<dyn IsGroupBox> = Rc::new(unsafe { Self::new_raw(GROUP_BOX_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        GroupBox {
            headered_decorator: unsafe { HeaderedDecorator::new_raw(vtable) },
            data: RefCell::new(GroupBoxData {
                double: false,
                color: (Fg::LightGray, Bg::None),
            }),
        }
    }

    pub fn _init_impl(this: &Rc<dyn IsView>) {
        HeaderedDecorator::_init_impl(this);
        let this: Rc<dyn IsGroupBox> = dyn_cast_rc(this.clone()).unwrap();
        this._set_header_text_color(this.color());
    }

    pub fn double_impl(this: &Rc<dyn IsGroupBox>) -> bool {
        this.group_box().data.borrow().double
    }

    pub fn set_double_impl(this: &Rc<dyn IsGroupBox>, value: bool) {
        this.group_box().data.borrow_mut().double = value;
        this.invalidate_render();
    }

    pub fn color_impl(this: &Rc<dyn IsGroupBox>) -> (Fg, Bg) {
        this.group_box().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsGroupBox>, value: (Fg, Bg)) {
        this.group_box().data.borrow_mut().color = value;
        this._set_header_text_color(value);
        this.invalidate_render();
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsHeaderedDecorator> = dyn_cast_rc(this.clone()).unwrap();
        let w = w.map(|x| x.saturating_sub(2));
        let h = h.map(|x| x.saturating_sub(2));
        let size = if let Some(child) = this.child() {
            child.measure(w, h);
            child.desired_size()
        } else {
            Vector::null()
        };
        if let Some(header) = this.actual_header() {
            header.measure(Some(size.x.saturating_sub(2)), Some(1));
        }
        Thickness::all(1).expand_rect_size(size)
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsHeaderedDecorator> = dyn_cast_rc(this.clone()).unwrap();
        if let Some(header) = this.actual_header() {
            let header_bounds = Thickness::new(2, 0, 2, 0).shrink_rect(bounds.t_line());
            header.arrange(header_bounds);
        }
        if let Some(child) = this.child() {
            let child_bounds = Thickness::all(1).shrink_rect(bounds);
            child.arrange(child_bounds);
        }
        bounds.size
    }

    pub fn render_impl(this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        let this: Rc<dyn IsGroupBox> = dyn_cast_rc(this.clone()).unwrap();
        let bounds = this.inner_render_bounds();
        {
            let data = this.group_box().data.borrow();
            rp.fill_bg(data.color);
            rp.h_line(bounds.tl, bounds.w(), data.double, data.color);
            rp.h_line(bounds.bl_inner(), bounds.w(), data.double, data.color);
            rp.v_line(bounds.tl, bounds.h(), data.double, data.color);
            rp.v_line(bounds.tr_inner(), bounds.h(), data.double, data.color);
            rp.tl_edge(bounds.tl, data.double, data.color);
            rp.tr_edge(bounds.tr_inner(), data.double, data.color);
            rp.br_edge(bounds.br_inner(), data.double, data.color);
            rp.bl_edge(bounds.bl_inner(), data.double, data.color);
        }
        if let Some(header) = this.actual_header() {
            let header_bounds = header.render_bounds();
            let data = this.group_box().data.borrow();
            rp.text(header_bounds.tl.offset(Vector { x: -1, y: 0 }), data.color, " ");
            rp.text(header_bounds.tr(), data.color, " ");
        }
    }
}

#[macro_export]
macro_rules! group_box_template {
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
        $crate::headered_decorator_template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                $(use $path as $import;)*

                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub double: Option<bool>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color: Option<($crate::base::Fg, $crate::base::Bg)>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}
 
#[macro_export]
macro_rules! group_box_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::headered_decorator_apply_template!($this, $instance, $names);
        {
            use $crate::group_box::GroupBoxExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::group_box::IsGroupBox>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.double.map(|x| obj.set_double(x));
            $this.color.map(|x| obj.set_color(x));
        }
    };
}

group_box_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="GroupBox@Child")]
    pub struct GroupBoxTemplate in template { }
}

#[typetag::serde(name="GroupBox")]
impl Template for GroupBoxTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        GroupBox::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        group_box_apply_template!(this, instance, names);
    }
}
