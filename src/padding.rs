use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use crate::template::{Template, NameResolver};

import! { pub padding:
    use [decorator crate::decorator];
    use crate::base::{Fg, Bg};
}

struct PaddingData {
    color: (Fg, Bg),
}

#[class_unsafe(inherits_Decorator)]
pub struct Padding {
    data: RefCell<PaddingData>,
    #[non_virt]
    color: fn() -> (Fg, Bg),
    #[non_virt]
    set_color: fn(value: (Fg, Bg)),
    #[over]
    render: (),
}

impl Padding {
    pub fn new() -> Rc<dyn IsPadding> {
        let res: Rc<dyn IsPadding> = Rc::new(unsafe { Self::new_raw(PADDING_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Padding {
            decorator: unsafe { Decorator::new_raw(vtable) },
            data: RefCell::new(PaddingData {
                color: (Fg::LightGray, Bg::Black),
            }),
        }
    }

    pub fn color_impl(this: &Rc<dyn IsPadding>) -> (Fg, Bg) {
        this.padding().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsPadding>, value: (Fg, Bg)) {
        {
            let mut data = this.padding().data.borrow_mut();
            if data.color == value { return; }
            data.color = value;
        }
        this.invalidate_render();
    }

    pub fn render_impl(this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        let this: Rc<dyn IsPadding> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.padding().data.borrow();
        rp.fill_bg(data.color);
    }
}

#[macro_export]
macro_rules! padding_template {
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
macro_rules! padding_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::decorator_apply_template!($this, $instance, $names);
        {
            use $crate::padding::PaddingExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::padding::IsPadding>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.color.map(|x| obj.set_color(x));
        }
    };
}

padding_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="Padding@Child")]
    pub struct PaddingTemplate in template { }
}

#[typetag::serde(name="Padding")]
impl Template for PaddingTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        Padding::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        padding_apply_template!(this, instance, names);
    }
}
