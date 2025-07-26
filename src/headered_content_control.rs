use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use std::mem::replace;
use std::ptr::addr_eq;
use crate::base::{option_addr_eq, TextWrapping};
use crate::content_presenter::{IsContentPresenter, ContentPresenterExt, ContentPresenterTemplate};
use crate::dock_panel::{DockLayoutTemplate, DockPanelTemplate, Dock};
use crate::template::{Template, NameResolver, Names};

import! { pub headered_content_control:
    use [content_control crate::content_control];
}

struct HeaderedContentControlData {
    header: Option<Rc<dyn IsView>>,
    header_text: Rc<String>,
    header_template: Option<Rc<dyn Template>>,
    loaded_header_template: Option<Rc<dyn IsView>>,
}

#[class_unsafe(inherits_ContentControl)]
pub struct HeaderedContentControl {
    data: RefCell<HeaderedContentControlData>,
    #[non_virt]
    header: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    set_header: fn(value: Option<Rc<dyn IsView>>),
    #[non_virt]
    header_text: fn() -> Rc<String>,
    #[non_virt]
    set_header_text: fn(value: Rc<String>),
    #[non_virt]
    header_template: fn() -> Option<Rc<dyn Template>>,
    #[non_virt]
    set_header_template: fn(value: Option<Rc<dyn Template>>),
    #[over]
    template: (),
    #[over]
    update_override: (),
}

impl HeaderedContentControl {
    pub fn new() -> Rc<dyn IsHeaderedContentControl> {
        let res: Rc<dyn IsHeaderedContentControl>
            = Rc::new(unsafe { Self::new_raw(HEADERED_CONTENT_CONTROL_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        HeaderedContentControl {
            content_control: unsafe { ContentControl::new_raw(vtable) },
            data: RefCell::new(HeaderedContentControlData {
                header: None,
                header_text: Rc::new(String::new()),
                header_template: None,
                loaded_header_template: None,
            }),
        }
    }

    pub fn header_impl(this: &Rc<dyn IsHeaderedContentControl>) -> Option<Rc<dyn IsView>> {
        this.headered_content_control().data.borrow().header.clone()
    }

    pub fn set_header_impl(this: &Rc<dyn IsHeaderedContentControl>, value: Option<Rc<dyn IsView>>) {
        {
            let mut data = this.headered_content_control().data.borrow_mut();
            if option_addr_eq(data.header.as_ref().map(Rc::as_ptr), value.as_ref().map(Rc::as_ptr)) { return; }
            data.header = value;
        }
        this.update();
    }

    pub fn header_text_impl(this: &Rc<dyn IsHeaderedContentControl>) -> Rc<String> {
        this.headered_content_control().data.borrow().header_text.clone()
    }

    pub fn set_header_text_impl(this: &Rc<dyn IsHeaderedContentControl>, value: Rc<String>) {
        {
            let mut data = this.headered_content_control().data.borrow_mut();
            if addr_eq(Rc::as_ptr(&data.header_text), Rc::as_ptr(&value)) { return; }
            data.header_text = value;
        }
        this.update();
    }

    pub fn header_template_impl(this: &Rc<dyn IsHeaderedContentControl>) -> Option<Rc<dyn Template>> {
        this.headered_content_control().data.borrow().header_template.clone()
    }

    pub fn set_header_template_impl(this: &Rc<dyn IsHeaderedContentControl>, value: Option<Rc<dyn Template>>) {
        {
            let mut data = this.headered_content_control().data.borrow_mut();
            if option_addr_eq(data.header_template.as_ref().map(Rc::as_ptr), value.as_ref().map(Rc::as_ptr)) {
                return;
            }
            data.header_template = value;
        }
        this.update();
    }

    pub fn template_impl(_this: &Rc<dyn IsControl>) -> Box<dyn Template> {
        Box::new(DockPanelTemplate {
            children: vec![
                Box::new(ContentPresenterTemplate {
                    name: "PART_HeaderPresenter".to_string(),
                    layout: Some(Box::new(DockLayoutTemplate {
                        dock: Some(Dock::Top),
                        .. Default::default()
                    })),
                    .. Default::default()
                }),
                Box::new(ContentPresenterTemplate {
                    name: "PART_ContentPresenter".to_string(),
                    text_wrapping: Some(TextWrapping::Wrap),
                    .. Default::default()
                }),
            ],
            .. Default::default()
        })
    }

    pub fn update_override_impl(this: &Rc<dyn IsControl>, template: &Names) {
        ContentControl::update_override_impl(this, template);
        let this: Rc<dyn IsHeaderedContentControl> = dyn_cast_rc(this.clone()).unwrap();
        let part_header_presenter: Rc<dyn IsContentPresenter>
            = dyn_cast_rc(
                template.find("PART_HeaderPresenter").expect("PART_HeaderPresenter").clone()
            ).expect("PART_HeaderPresenter: ContentPresenter");
        let (header, text, old_loaded_header_template, new_loaded_header_template) = {
            let mut data = this.headered_content_control().data.borrow_mut();
            let new_loaded_header_template: Option<Rc<dyn IsView>>
                = data.header_template.as_ref().map(|x| dyn_cast_rc(x.load_root().0).expect("View"));
            let old_loaded_header_template
                = replace(&mut data.loaded_header_template, new_loaded_header_template.clone());
            (
                data.header.clone(),
                data.header_text.clone(),
                old_loaded_header_template,
                new_loaded_header_template,
            )
        };
        if let Some(old_loaded_header_template) = old_loaded_header_template {
            this._raise_unbind(&old_loaded_header_template, 1);
        }
        part_header_presenter.set_content(new_loaded_header_template.clone().or(header));
        part_header_presenter.set_text(text);
        if let Some(new_loaded_header_template) = new_loaded_header_template {
            this._raise_bind(&new_loaded_header_template, 1);
        }
    }
}

#[macro_export]
macro_rules! headered_content_control_template {
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
        $crate::content_control_template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                $(use $path as $import;)*

                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub header: Option<Box<dyn $crate::template::Template>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub header_text: Option<String>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! headered_content_control_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::content_control_apply_template!($this, $instance, $names);
        {
            use $crate::headered_content_control::HeaderedContentControlExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::headered_content_control::IsHeaderedContentControl>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.header.as_ref().map(|x|
                obj.set_header(Some($crate::dynamic_cast_dyn_cast_rc(x.load_content($names)).unwrap()))
            );
            $this.header_text.as_ref().map(|x| obj.set_header_text($crate::alloc_rc_Rc::new(x.clone())));
        }
    };
}

headered_content_control_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="HeaderedContentControl@Content")]
    pub struct HeaderedContentControlTemplate in template { }
}

#[typetag::serde(name="HeaderedContentControl")]
impl Template for HeaderedContentControlTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        HeaderedContentControl::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        headered_content_control_apply_template!(this, instance, names);
    }
}
