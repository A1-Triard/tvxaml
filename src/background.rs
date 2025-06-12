use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use crate::template::{Template, NameResolver};

import! { pub background:
    use [decorator crate::decorator];
    use crate::base::{Fg, Bg};
}

struct BackgroundData {
    pattern: Rc<String>,
    color: (Fg, Bg),
}

#[class_unsafe(inherits_Decorator)]
pub struct Background {
    data: RefCell<BackgroundData>,
    #[non_virt]
    pattern: fn() -> Rc<String>,
    #[non_virt]
    set_pattern: fn(value: Rc<String>),
    #[non_virt]
    color: fn() -> (Fg, Bg),
    #[non_virt]
    set_color: fn(value: (Fg, Bg)),
    #[over]
    render: (),
    #[over]
    arrange_override: (),
}

impl Background {
    pub fn new() -> Rc<dyn IsBackground> {
        let res: Rc<dyn IsBackground> = Rc::new(unsafe { Self::new_raw(BACKGROUND_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Background {
            decorator: unsafe { Decorator::new_raw(vtable) },
            data: RefCell::new(BackgroundData {
                pattern: Rc::new("░".to_string()),
                color: (Fg::LightGray, Bg::Black),
            }),
        }
    }

    pub fn pattern_impl(this: &Rc<dyn IsBackground>) -> Rc<String> {
        this.background().data.borrow().pattern.clone()
    }

    pub fn set_pattern_impl(this: &Rc<dyn IsBackground>, value: Rc<String>) {
        this.background().data.borrow_mut().pattern = value;
        this.invalidate_render();
    }

    pub fn color_impl(this: &Rc<dyn IsBackground>) -> (Fg, Bg) {
        this.background().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsBackground>, value: (Fg, Bg)) {
        this.background().data.borrow_mut().color = value;
        this.invalidate_render();
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        Decorator::arrange_override_impl(this, bounds);
        bounds.size
    }

    pub fn render_impl(this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        let this: Rc<dyn IsBackground> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.background().data.borrow();
        rp.fill(|rp, p| rp.text(p, data.color, &data.pattern));
    }
}

#[macro_export]
macro_rules! background_template {
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
                pub pattern: Option<String>,
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
macro_rules! background_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::decorator_apply_template!($this, $instance, $names);
        {
            use $crate::background::BackgroundExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::background::IsBackground>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.pattern.as_ref().map(|x| obj.set_pattern(Rc::new(x.clone())));
            $this.color.map(|x| obj.set_color(x));
        }
    };
}

background_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="Background@Child")]
    pub struct BackgroundTemplate in template { }
}

#[typetag::serde(name="Background")]
impl Template for BackgroundTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        Background::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        background_apply_template!(this, instance, names);
    }
}
