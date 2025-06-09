use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use crate::base::text_width;
use crate::template::{Template, NameResolver};

import! { pub frame:
    use [decorator crate::decorator];
    use crate::base::{Fg, Bg};
}

struct FrameData {
    text: Rc<String>,
    text_align: HAlign,
    double: bool,
    color: (Fg, Bg),
}

#[class_unsafe(inherits_Decorator)]
pub struct Frame {
    data: RefCell<FrameData>,
    #[non_virt]
    text: fn() -> Rc<String>,
    #[non_virt]
    set_text: fn(value: Rc<String>),
    #[non_virt]
    text_align: fn() -> HAlign,
    #[non_virt]
    set_text_align: fn(value: HAlign),
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

impl Frame {
    pub fn new() -> Rc<dyn IsFrame> {
        let res: Rc<dyn IsFrame> = Rc::new(unsafe { Self::new_raw(FRAME_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Frame {
            decorator: unsafe { Decorator::new_raw(vtable) },
            data: RefCell::new(FrameData {
                text: Rc::new(String::new()),
                text_align: HAlign::Left,
                double: false,
                color: (Fg::LightGray, Bg::None),
            }),
        }
    }

    pub fn text_impl(this: &Rc<dyn IsFrame>) -> Rc<String> {
        this.frame().data.borrow().text.clone()
    }

    pub fn set_text_impl(this: &Rc<dyn IsFrame>, value: Rc<String>) {
        this.frame().data.borrow_mut().text = value;
        this.invalidate_render();
    }

    pub fn text_align_impl(this: &Rc<dyn IsFrame>) -> HAlign {
        this.frame().data.borrow().text_align
    }

    pub fn set_text_align_impl(this: &Rc<dyn IsFrame>, value: HAlign) {
        this.frame().data.borrow_mut().text_align = value;
        this.invalidate_render();
    }

    pub fn double_impl(this: &Rc<dyn IsFrame>) -> bool {
        this.frame().data.borrow().double
    }

    pub fn set_double_impl(this: &Rc<dyn IsFrame>, value: bool) {
        this.frame().data.borrow_mut().double = value;
        this.invalidate_render();
    }

    pub fn color_impl(this: &Rc<dyn IsFrame>) -> (Fg, Bg) {
        this.frame().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsFrame>, value: (Fg, Bg)) {
        this.frame().data.borrow_mut().color = value;
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
        let this: Rc<dyn IsFrame> = dyn_cast_rc(this.clone()).unwrap();
        let bounds = this.inner_render_bounds();
        let data = this.frame().data.borrow();
        rp.fill_bg(data.color);
        rp.h_line(bounds.tl, bounds.w(), data.double, data.color);
        rp.h_line(bounds.bl_inner(), bounds.w(), data.double, data.color);
        rp.v_line(bounds.tl, bounds.h(), data.double, data.color);
        rp.v_line(bounds.tr_inner(), bounds.h(), data.double, data.color);
        rp.tl_edge(bounds.tl, data.double, data.color);
        rp.tr_edge(bounds.tr_inner(), data.double, data.color);
        rp.br_edge(bounds.br_inner(), data.double, data.color);
        rp.bl_edge(bounds.bl_inner(), data.double, data.color);
        if !data.text.is_empty() {
            let text_area_bounds = Thickness::new(2, 0, 2, 0).shrink_rect(bounds.t_line());
            let text_width = text_width(&data.text);
            if text_width <= text_area_bounds.w() {
                let margin = Thickness::align(
                    Vector { x: text_width, y: 1 },
                    text_area_bounds.size,
                    data.text_align,
                    VAlign::Top
                );
                let text_bounds = margin.shrink_rect(text_area_bounds);
                rp.text(text_bounds.tl.offset(Vector { x: -1, y: 0 }), data.color, " ");
                rp.text(text_bounds.tl, data.color, &data.text);
                rp.text(text_bounds.tr(), data.color, " ");
            } else {
                rp.text(text_area_bounds.tl.offset(Vector { x: -1, y: 0 }), data.color, " ");
                rp.text(text_area_bounds.tl, data.color, &data.text);
                rp.text(text_area_bounds.tr(), data.color, "►");
                rp.tr_edge(bounds.tr_inner(), data.double, data.color);
            }
        }
    }
}

#[macro_export]
macro_rules! frame_template {
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
                pub text: Option<String>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub text_align: Option<$crate::base::HAlign>,
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
macro_rules! frame_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::decorator_apply_template!($this, $instance, $names);
        {
            use $crate::frame::FrameExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::frame::IsFrame>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.text.as_ref().map(|x| obj.set_text(Rc::new(x.clone())));
            $this.text_align.map(|x| obj.set_text_align(x));
            $this.double.map(|x| obj.set_double(x));
            $this.color.map(|x| obj.set_color(x));
        }
    };
}

frame_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="Frame@Child")]
    pub struct FrameTemplate in template { }
}

#[typetag::serde(name="Frame")]
impl Template for FrameTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        Frame::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        frame_apply_template!(this, instance, names);
    }
}
