use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use std::cell::Cell;
use crate::template::{Template, NameResolver};
use crate::view_vec::ViewVecExt;

import! { pub canvas_layout:
    use [layout crate::view];
    use int_vec_2d::Point;
}

#[class_unsafe(inherits_Layout)]
pub struct CanvasLayout {
    tl: Cell<Point>,
    #[non_virt]
    tl: fn() -> Point,
    #[non_virt]
    set_tl: fn(value: Point),
}

impl CanvasLayout {
    pub fn new() -> Rc<dyn IsCanvasLayout> {
        Rc::new(unsafe { Self::new_raw(CANVAS_LAYOUT_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        CanvasLayout {
            layout: unsafe { Layout::new_raw(vtable) },
            tl: Cell::new(Point { x: 0, y: 0 }),
        }
    }

    pub fn tl_impl(this: &Rc<dyn IsCanvasLayout>) -> Point {
        this.canvas_layout().tl.get()
    }

    pub fn set_tl_impl(this: &Rc<dyn IsCanvasLayout>, value: Point) {
        this.canvas_layout().tl.set(value);
        this.owner().and_then(|x| x.layout_parent()).map(|x| x.invalidate_arrange());
    }
}

#[macro_export]
macro_rules! canvas_layout_template {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $($(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        $crate::layout_template! {
            $(#[$attr])*
            $vis struct $name {
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub tl: Option<$crate::int_vec_2d_Point>,
                $($(
                    $(#[$field_attr])*
                    $field_vis $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! canvas_layout_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::layout_apply_template!($this, $instance, $names);
        {
            use $crate::canvas::CanvasLayoutExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::canvas::IsCanvasLayout>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.tl.map(|x| obj.set_tl(x));
        }
    };
}

canvas_layout_template! {
    #[derive(Serialize, Deserialize, Default)]
    #[serde(rename="CanvasLayout@Tl")]
    pub struct CanvasLayoutTemplate { }
}

#[typetag::serde(name="CanvasLayout")]
impl Template for CanvasLayoutTemplate {
    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = CanvasLayout::new();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        canvas_layout_apply_template!(this, instance, names);
    }
}

import! { pub canvas:
    use [panel crate::panel];
}

#[class_unsafe(inherits_Panel)]
pub struct Canvas {
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
}

impl Canvas {
    pub fn new() -> Rc<dyn IsCanvas> {
        let res: Rc<dyn IsCanvas> = Rc::new(unsafe { Self::new_raw(CANVAS_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Canvas {
            panel: unsafe { Panel::new_raw(vtable) },
        }
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        for child in this.children().iter() {
            child.measure(None, None);
        }
        Vector { x: w.unwrap_or(1), y: h.unwrap_or(1) }
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        for child in this.children().iter() {
            let child_size = child.desired_size();
            let layout: Option<Rc<dyn IsCanvasLayout>> = dyn_cast_rc(child.layout());
            let tl = layout.map_or(Point { x: 0, y: 0 }, |x| x.tl());
            child.arrange(Rect { tl, size: child_size });
        }
        bounds.size
    }
}

#[macro_export]
macro_rules! canvas_template {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $($(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        $crate::panel_template! {
            $(#[$attr])*
            $vis struct $name {
                $($(
                    $(#[$field_attr])*
                    $field_vis $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! canvas_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::panel_apply_template!($this, $instance, $names);
    };
}

canvas_template! {
    #[derive(Serialize, Deserialize, Default)]
    #[serde(rename="Canvas@Children")]
    pub struct CanvasTemplate { }
}

#[typetag::serde(name="Canvas")]
impl Template for CanvasTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        dyn_cast_rc(Canvas::new()).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        canvas_apply_template!(this, instance, names);
    }
}
