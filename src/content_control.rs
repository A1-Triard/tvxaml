use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use std::ptr::addr_eq;
use crate::base::{option_addr_eq, TextWrapping};
use crate::content_presenter::{IsContentPresenter, ContentPresenterExt, ContentPresenterTemplate};
use crate::template::{NameResolver, Names};

import! { pub content_control:
    use [control crate::control];
    use crate::base::{Fg, Bg};
}

struct ContentControlData {
    content: Option<Rc<dyn IsView>>,
    text: Rc<String>,
    text_color: (Fg, Bg),
}

#[class_unsafe(inherits_Control)]
pub struct ContentControl {
    data: RefCell<ContentControlData>,
    #[non_virt]
    content: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    set_content: fn(value: Option<Rc<dyn IsView>>),
    #[non_virt]
    text: fn() -> Rc<String>,
    #[non_virt]
    set_text: fn(value: Rc<String>),
    #[non_virt]
    text_color: fn() -> (Fg, Bg),
    #[non_virt]
    set_text_color: fn(value: (Fg, Bg)),
    #[over]
    template: (),
    #[over]
    update_override: (),
}

impl ContentControl {
    pub fn new() -> Rc<dyn IsContentControl> {
        let res: Rc<dyn IsContentControl>
            = Rc::new(unsafe { Self::new_raw(CONTENT_CONTROL_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        ContentControl {
            control: unsafe { Control::new_raw(vtable) },
            data: RefCell::new(ContentControlData {
                content: None,
                text: Rc::new(String::new()),
                text_color: (Fg::LightGray, Bg::None),
            }),
        }
    }

    pub fn update_override_impl(this: &Rc<dyn IsControl>, template: &Names) {
        let this: Rc<dyn IsContentControl> = dyn_cast_rc(this.clone()).unwrap();
        let part_content_presenter: Rc<dyn IsContentPresenter>
            = dyn_cast_rc(
                template.find("PART_ContentPresenter").expect("PART_ContentPresenter").clone()
            ).expect("PART_ContentPresenter: ContentPresenter");
        let (content, text, text_color) = {
            let data = this.content_control().data.borrow();
            (data.content.clone(), data.text.clone(), data.text_color)
        };
        part_content_presenter.set_text(text);
        part_content_presenter.set_text_color(text_color);
        part_content_presenter.set_content(content);
    }

    pub fn content_impl(this: &Rc<dyn IsContentControl>) -> Option<Rc<dyn IsView>> {
        this.content_control().data.borrow().content.clone()
    }

    pub fn set_content_impl(this: &Rc<dyn IsContentControl>, value: Option<Rc<dyn IsView>>) {
        {
            let mut data = this.content_control().data.borrow_mut();
            if option_addr_eq(data.content.as_ref().map(Rc::as_ptr), value.as_ref().map(Rc::as_ptr)) {
                return;
            }
            data.content = value;
        }
        this.update();
    }

    pub fn text_impl(this: &Rc<dyn IsContentControl>) -> Rc<String> {
        this.content_control().data.borrow().text.clone()
    }

    pub fn set_text_impl(this: &Rc<dyn IsContentControl>, value: Rc<String>) {
        {
            let mut data = this.content_control().data.borrow_mut();
            if addr_eq(Rc::as_ptr(&data.text), Rc::as_ptr(&value)) { return; }
            data.text = value;
        }
        this.update();
    }

    pub fn text_color_impl(this: &Rc<dyn IsContentControl>) -> (Fg, Bg) {
        this.content_control().data.borrow().text_color
    }

    pub fn set_text_color_impl(this: &Rc<dyn IsContentControl>, value: (Fg, Bg)) {
        {
            let mut data = this.content_control().data.borrow_mut();
            if data.text_color == value { return; }
            data.text_color = value;
        };
        this.update();
    }

    pub fn template_impl(_this: &Rc<dyn IsControl>) -> Box<dyn Template> {
        Box::new(ContentPresenterTemplate {
            name: "PART_ContentPresenter".to_string(),
            text_wrapping: Some(TextWrapping::Wrap),
            .. Default::default()
        })
    }
}

#[macro_export]
macro_rules! content_control_template {
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
        $crate::control_template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                $(use $path as $import;)*

                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub content: Option<Box<dyn $crate::template::Template>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub text: Option<String>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub text_color: Option<($crate::base::Fg, $crate::base::Bg)>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! content_control_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::control_apply_template!($this, $instance, $names);
        {
            use $crate::content_control::ContentControlExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::content_control::IsContentControl>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.content.as_ref().map(|x|
                obj.set_content(Some($crate::dynamic_cast_dyn_cast_rc(x.load_content($names)).unwrap()))
            );
            $this.text.as_ref().map(|x| obj.set_text($crate::alloc_rc_Rc::new(x.clone())));
            $this.text_color.map(|x| obj.set_text_color(x))
        }
    };
}

content_control_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="ContentControl@Content")]
    pub struct ContentControlTemplate in template { }
}

#[typetag::serde(name="ContentControl")]
impl Template for ContentControlTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        ContentControl::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        content_control_apply_template!(this, instance, names);
    }
}
