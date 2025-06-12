use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use either::{Either, Left, Right};
use std::cell::RefCell;
use std::ptr::addr_eq;
use crate::base::option_addr_eq;
use crate::static_text::{StaticText, IsStaticText, StaticTextExt};
use crate::template::{Template, NameResolver};

import! { pub headered_decorator:
    use [decorator crate::decorator];
    use crate::base::{HAlign, Fg, Bg};
}

struct HeaderedDecoratorData {
    header: Option<Rc<dyn IsView>>,
    header_text: Rc<String>,
    header_text_align: HAlign,
    header_text_color: (Fg, Bg),
    actual_header: Option<Either<Rc<dyn IsStaticText>, Rc<dyn IsView>>>,
}

#[class_unsafe(inherits_Decorator)]
pub struct HeaderedDecorator {
    data: RefCell<HeaderedDecoratorData>,
    #[non_virt]
    actual_header: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    header: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    set_header: fn(value: Option<Rc<dyn IsView>>),
    #[non_virt]
    header_text: fn() -> Rc<String>,
    #[non_virt]
    set_header_text: fn(value: Rc<String>),
    #[non_virt]
    header_text_align: fn() -> HAlign,
    #[non_virt]
    set_header_text_align: fn(value: HAlign),
    #[non_virt]
    _header_text_color: fn() -> (Fg, Bg),
    #[non_virt]
    _set_header_text_color: fn(value: (Fg, Bg)),
    #[over]
    visual_children_count: (),
    #[over]
    visual_child: (),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
}

impl HeaderedDecorator {
    pub fn new() -> Rc<dyn IsHeaderedDecorator> {
        let res: Rc<dyn IsHeaderedDecorator>
            = Rc::new(unsafe { Self::new_raw(HEADERED_DECORATOR_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        HeaderedDecorator {
            decorator: unsafe { Decorator::new_raw(vtable) },
            data: RefCell::new(HeaderedDecoratorData {
                header: None,
                header_text: Rc::new(String::new()),
                header_text_align: HAlign::Left,
                header_text_color: (Fg::Red, Bg::Green),
                actual_header: None,
            }),
        }
    }

    pub fn actual_header_impl(this: &Rc<dyn IsHeaderedDecorator>) -> Option<Rc<dyn IsView>> {
        let header = this.headered_decorator().data.borrow().actual_header.clone();
        header.map(|x| x.either(|x| { let y: Rc<dyn IsView> = x; y }, |x| x))
    }

    pub fn header_impl(this: &Rc<dyn IsHeaderedDecorator>) -> Option<Rc<dyn IsView>> {
        this.headered_decorator().data.borrow().header.clone()
    }

    pub fn set_header_impl(this: &Rc<dyn IsHeaderedDecorator>, value: Option<Rc<dyn IsView>>) {
        let new_actual_header = {
            let mut data = this.headered_decorator().data.borrow_mut();
            if option_addr_eq(data.header.as_ref().map(Rc::as_ptr), value.as_ref().map(Rc::as_ptr)) { return; }
            data.header = value.clone();
            if data.actual_header.as_ref().map_or(false, |x| x.is_right()) != value.is_some() {
                if let Some(value) = value {
                    Some(Some(Right(value)))
                } else if data.header_text.is_empty() {
                    Some(None)
                } else {
                    let text = StaticText::new();
                    text.set_text(data.header_text.clone());
                    text.set_h_align(Some(data.header_text_align).into());
                    text.set_color(data.header_text_color);
                    Some(Some(Left(text)))
                }
            } else {
                if let Some(value) = value {
                    Some(Some(Right(value)))
                } else {
                    None
                }
            }
        };
        if let Some(new_actual_header) = new_actual_header {
            Self::set_actual_header(this, new_actual_header);
        }
    }

    pub fn header_text_impl(this: &Rc<dyn IsHeaderedDecorator>) -> Rc<String> {
        this.headered_decorator().data.borrow().header_text.clone()
    }

    pub fn set_header_text_impl(this: &Rc<dyn IsHeaderedDecorator>, value: Rc<String>) {
        let new_actual_header = {
            let mut data = this.headered_decorator().data.borrow_mut();
            if addr_eq(Rc::as_ptr(&data.header_text), Rc::as_ptr(&value)) { return; }
            data.header_text = value.clone();
            if data.actual_header.as_ref().map_or(false, |x| x.is_left()) != value.is_empty() {
                if value.is_empty() {
                    None
                } else {
                    Some(Some(Right(())))
                }
            } else {
                if value.is_empty() {
                    Some(None)
                } else if data.actual_header.is_some() {
                    None
                } else {
                    let text = StaticText::new();
                    text.set_text(data.header_text.clone());
                    text.set_h_align(Some(data.header_text_align).into());
                    text.set_color(data.header_text_color);
                    Some(Some(Left(text)))
                }
            }
        };
        if let Some(new_actual_header) = new_actual_header {
            if let Some(new_actual_header) = new_actual_header {
                match new_actual_header {
                    Left(text) => Self::set_actual_header(this, Some(Left(text))),
                    Right(()) => {
                        let data = this.headered_decorator().data.borrow();
                        data.actual_header.as_ref().unwrap().as_ref().left().unwrap().set_text(value);
                    },
                }
            } else {
                Self::set_actual_header(this, None);
            }
        }
    }

    fn set_actual_header(
        this: &Rc<dyn IsHeaderedDecorator>,
        value: Option<Either<Rc<dyn IsStaticText>, Rc<dyn IsView>>>,
    ) {
        let header = this.headered_decorator().data.borrow().actual_header.clone();
        if let Some(header) = header {
            let header = header.either(|x| { let y: Rc<dyn IsView> = x; y }, |x| x);
            this.remove_visual_child(&header);
            header._set_visual_parent(None);
            header._set_layout_parent(None);
        }
        this.headered_decorator().data.borrow_mut().actual_header = value.clone();
        let this: Rc<dyn IsView> = this.clone();
        if let Some(header) = value {
            let header = header.either(|x| { let y: Rc<dyn IsView> = x; y }, |x| x);
            header._set_layout_parent(Some(&this));
            header._set_visual_parent(Some(&this));
            this.add_visual_child(&header);
        }
        this.invalidate_measure();
    }

    pub fn header_text_align_impl(this: &Rc<dyn IsHeaderedDecorator>) -> HAlign {
        this.headered_decorator().data.borrow().header_text_align
    }

    pub fn set_header_text_align_impl(this: &Rc<dyn IsHeaderedDecorator>, value: HAlign) {
        let header = {
            let mut data = this.headered_decorator().data.borrow_mut();
            data.header_text_align = value;
            if data.header_text_align == value { return; }
            data.actual_header.as_ref().and_then(|x| x.as_ref().left()).map(|x| x.clone())
        };
        if let Some(header) = header {
            header.set_h_align(Some(value).into());
        }
    }

    pub fn _header_text_color_impl(this: &Rc<dyn IsHeaderedDecorator>) -> (Fg, Bg) {
        this.headered_decorator().data.borrow().header_text_color
    }

    pub fn _set_header_text_color_impl(this: &Rc<dyn IsHeaderedDecorator>, value: (Fg, Bg)) {
        let mut data = this.headered_decorator().data.borrow_mut();
        if data.header_text_color == value { return; }
        data.header_text_color = value;
        let header = data.actual_header.as_ref().and_then(|x| x.as_ref().left());
        if let Some(header) = header {
            header.set_color(value);
        }
    }

    pub fn visual_children_count_impl(this: &Rc<dyn IsView>) -> usize {
        let this: Rc<dyn IsHeaderedDecorator> = dyn_cast_rc(this.clone()).unwrap();
        let mut children_count = 0;
        if this.headered_decorator().data.borrow().actual_header.is_some() { children_count += 1;  }
        if this.child().is_some() { children_count += 1;  }
        children_count
    }

    pub fn visual_child_impl(this: &Rc<dyn IsView>, index: usize) -> Rc<dyn IsView> {
        let this: Rc<dyn IsHeaderedDecorator> = dyn_cast_rc(this.clone()).unwrap();
        match index {
            0 => if let Some(header) = this.headered_decorator().data.borrow().actual_header.clone() {
                header.either(|x| { let y: Rc<dyn IsView> = x; y }, |x| x)
            } else {
                this.child().unwrap()
            },
            1 => this.child().unwrap(),
            _ => panic!(),
        }
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, mut h: Option<i16>) -> Vector {
        let this: Rc<dyn IsHeaderedDecorator> = dyn_cast_rc(this.clone()).unwrap();
        let has_header = if let Some(header) = this.headered_decorator().data.borrow().actual_header.clone() {
            let header = header.either(|x| { let y: Rc<dyn IsView> = x; y }, |x| x);
            header.measure(w, Some(1));
            h.as_mut().map(|x| *x = (*x as u16).saturating_sub(1) as i16);
            true
        } else {
            false
        };
        let size = if let Some(child) = this.child() {
            eprintln!("has child");
            child.measure(w, h);
            child.desired_size()
        } else {
            eprintln!("no child");
            Vector::null()
        };
        let r = if has_header {
            Thickness::new(0, 1, 0, 0).expand_rect_size(size)
        } else {
            size
        };
        eprintln!("m = {r:?}");
        r
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, mut bounds: Rect) -> Vector {
        let this: Rc<dyn IsHeaderedDecorator> = dyn_cast_rc(this.clone()).unwrap();
        let has_header = if let Some(header) = this.headered_decorator().data.borrow().actual_header.clone() {
            let header = header.either(|x| { let y: Rc<dyn IsView> = x; y }, |x| x);
            header.arrange(bounds.t_line());
            bounds = Thickness::new(0, 1, 0, 0).shrink_rect(bounds);
            true
        } else {
            false
        };
        let size = if let Some(child) = this.child() {
            child.arrange(bounds);
            child.render_bounds().size
        } else {
            Vector::null()
        };
        if has_header {
            Thickness::new(0, 1, 0, 0).expand_rect_size(size)
        } else {
            size
        }
    }
}

#[macro_export]
macro_rules! headered_decorator_template {
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
                pub header: Option<Box<dyn $crate::template::Template>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub header_text: Option<String>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub header_text_align: Option<$crate::base::HAlign>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! headered_decorator_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::decorator_apply_template!($this, $instance, $names);
        {
            use $crate::headered_decorator::HeaderedDecoratorExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::headered_decorator::IsHeaderedDecorator>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.header.as_ref().map(|x|
                obj.set_header(Some($crate::dynamic_cast_dyn_cast_rc(x.load_content($names)).unwrap()))
            );
            $this.header_text.as_ref().map(|x| obj.set_header_text($crate::alloc_rc_Rc::new(x.clone())));
            $this.header_text_align.map(|x| obj.set_header_text_align(x))
        }
    };
}

headered_decorator_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="HeaderedDecorator@Child")]
    pub struct HeaderedDecoratorTemplate in template { }
}

#[typetag::serde(name="HeaderedDecorator")]
impl Template for HeaderedDecoratorTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        HeaderedDecorator::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        headered_decorator_apply_template!(this, instance, names);
    }
}
