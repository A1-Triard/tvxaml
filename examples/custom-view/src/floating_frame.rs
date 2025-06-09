use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use tvxaml_screen_base::{Point, Fg, Bg};
use tvxaml::template::{Template, NameResolver};

import! { pub floating_frame:
    use [view tvxaml::view];
}

#[class_unsafe(inherits_View)]
pub struct FloatingFrame {
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
    #[over]
    render: (),
}

impl FloatingFrame {
    pub fn new() -> Rc<dyn IsFloatingFrame> {
        let res: Rc<dyn IsFloatingFrame> = Rc::new(unsafe { Self::new_raw(FLOATING_FRAME_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        FloatingFrame {
            view: unsafe { View::new_raw(vtable) },
        }
    }

    pub fn measure_override_impl(_this: &Rc<dyn IsView>, _w: Option<i16>, _h: Option<i16>) -> Vector {
        Vector { x: 13, y: 7 }
    }

    pub fn arrange_override_impl(_this: &Rc<dyn IsView>, _bounds: Rect) -> Vector {
        Vector { x: 13, y: 7 }
    }

    pub fn render_impl(_this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        rp.text(Point { x: 0, y: 0 }, (Fg::Green, Bg::None), "╔═══════════╗");
        rp.text(Point { x: 0, y: 1 }, (Fg::Green, Bg::None), "║     ↑     ║");
        rp.text(Point { x: 0, y: 2 }, (Fg::Green, Bg::None), "║     k     ║");
        rp.text(Point { x: 0, y: 3 }, (Fg::Green, Bg::None), "║ ←h     l→ ║");
        rp.text(Point { x: 0, y: 4 }, (Fg::Green, Bg::None), "║     j     ║");
        rp.text(Point { x: 0, y: 5 }, (Fg::Green, Bg::None), "║     ↓     ║");
        rp.text(Point { x: 0, y: 6 }, (Fg::Green, Bg::None), "╚═══════════╝");
    }
}

macro_rules! floating_frame_template {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $($(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        ::tvxaml::view_template! {
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
 
macro_rules! floating_frame_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        ::tvxaml::view_apply_template!($this, $instance, $names);
    };
}

floating_frame_template! {
    #[derive(Serialize, Deserialize, Default)]
    #[serde(rename="FloatingFrame")]
    pub struct FloatingFrameTemplate { }
}

#[typetag::serde(name="FloatingFrame")]
impl Template for FloatingFrameTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        dyn_cast_rc(FloatingFrame::new()).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        floating_frame_apply_template!(this, instance, names);
    }
}
