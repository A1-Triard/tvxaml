use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use crate::template::{Template, NameResolver};

import! { pub border:
    use [decorator crate::decorator];
    use crate::base::{Fg, Bg};
}

struct BorderData {
    double: bool,
    color: (Fg, Bg),
}

#[class_unsafe(inherits_Decorator)]
pub struct Border {
    data: RefCell<BorderData>,
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
}

impl Border {
    pub fn new() -> Rc<dyn IsBorder> {
        let res: Rc<dyn IsBorder> = Rc::new(unsafe { Self::new_raw(BORDER_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Border {
            decorator: unsafe { Decorator::new_raw(vtable) },
            data: RefCell::new(BorderData {
                double: false,
                color: (Fg::LightGray, Bg::None),
            }),
        }
    }

    pub fn double_impl(this: &Rc<dyn IsBorder>) -> bool {
        this.border().data.borrow().double
    }

    pub fn set_double_impl(this: &Rc<dyn IsBorder>, value: bool) {
        {
            let mut data = this.border().data.borrow_mut();
            if data.double == value { return; }
            data.double = value;
        }
        this.invalidate_render();
    }

    pub fn color_impl(this: &Rc<dyn IsBorder>) -> (Fg, Bg) {
        this.border().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsBorder>, value: (Fg, Bg)) {
        {
            let mut data = this.border().data.borrow_mut();
            if data.color == value { return; }
            data.color = value;
        }
        this.invalidate_render();
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsDecorator> = dyn_cast_rc(this.clone()).unwrap();
        let size = if let Some(child) = this.child() {
            let available_size = Vector { x: w.unwrap_or(0), y: h.unwrap_or(0) };
            let child_size = Thickness::all(1).shrink_rect_size(available_size);
            let child_width = if w.is_none() { None } else { Some(child_size.x) };
            let child_height = if h.is_none() { None } else { Some(child_size.y) };
            child.measure(child_width, child_height);
            child.desired_size()
        } else {
            Vector::null()
        };
        Thickness::all(1).expand_rect_size(size)
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsDecorator> = dyn_cast_rc(this.clone()).unwrap();
        if let Some(child) = this.child() {
            let child_bounds = Thickness::all(1).shrink_rect(bounds);
            child.arrange(child_bounds);
        }
        bounds.size
    }

    pub fn render_impl(this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        let this: Rc<dyn IsBorder> = dyn_cast_rc(this.clone()).unwrap();
        let bounds = this.inner_render_bounds();
        let data = this.border().data.borrow();
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
}

#[macro_export]
macro_rules! border_template {
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
        $crate::decorator_template! {
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
macro_rules! border_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::decorator_apply_template!($this, $instance, $names);
        {
            use $crate::border::BorderExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::border::IsBorder>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.double.map(|x| obj.set_double(x));
            $this.color.map(|x| obj.set_color(x));
        }
    };
}

border_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
    #[serde(rename="Border@Child")]
    pub struct BorderTemplate in template { }
}

#[typetag::serde(name="Border")]
impl Template for BorderTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        Border::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        border_apply_template!(this, instance, names);
    }
}
