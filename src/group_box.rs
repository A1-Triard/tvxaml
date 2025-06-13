use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use crate::base::TextWrapping;
use crate::border::{IsBorder, BorderExt, BorderTemplate};
use crate::content_presenter::{IsContentPresenter, ContentPresenterExt, ContentPresenterTemplate};
use crate::pile_panel::PilePanelTemplate;
use crate::template::{Template, NameResolver};

import! { pub group_box:
    use [headered_content_control crate::headered_content_control];
}

struct GroupBoxData {
    double: bool,
    color: (Fg, Bg),
    header_align: ViewHAlign,
}

#[class_unsafe(inherits_HeaderedContentControl)]
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
    #[non_virt]
    header_align: fn() -> ViewHAlign,
    #[non_virt]
    set_header_align: fn(value: ViewHAlign),
    #[over]
    update_override: (),
    #[over]
    template: (),
}

impl GroupBox {
    pub fn new() -> Rc<dyn IsGroupBox> {
        let res: Rc<dyn IsGroupBox> = Rc::new(unsafe { Self::new_raw(GROUP_BOX_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        GroupBox {
            headered_content_control: unsafe { HeaderedContentControl::new_raw(vtable) },
            data: RefCell::new(GroupBoxData {
                double: false,
                color: (Fg::LightGray, Bg::None),
                header_align: ViewHAlign::Left,
            }),
        }
    }

    pub fn double_impl(this: &Rc<dyn IsGroupBox>) -> bool {
        this.group_box().data.borrow().double
    }

    pub fn set_double_impl(this: &Rc<dyn IsGroupBox>, value: bool) {
        {
            let mut data = this.group_box().data.borrow_mut();
            if data.double == value { return; }
            data.double = value;
        }
        this.update();
    }

    pub fn color_impl(this: &Rc<dyn IsGroupBox>) -> (Fg, Bg) {
        this.group_box().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsGroupBox>, value: (Fg, Bg)) {
        {
            let mut data = this.group_box().data.borrow_mut();
            if data.color == value { return; }
            data.color = value;
        }
        this.update();
    }

    pub fn header_align_impl(this: &Rc<dyn IsGroupBox>) -> ViewHAlign {
        this.group_box().data.borrow().header_align
    }

    pub fn set_header_align_impl(this: &Rc<dyn IsGroupBox>, value: ViewHAlign) {
        {
            let mut data = this.group_box().data.borrow_mut();
            if data.header_align == value { return; }
            data.header_align = value;
        }
        this.update();
    }

    pub fn update_override_impl(this: &Rc<dyn IsControl>, template: &Names) {
        HeaderedContentControl::update_override_impl(this, template);
        let this: Rc<dyn IsGroupBox> = dyn_cast_rc(this.clone()).unwrap();
        let part_border: Rc<dyn IsBorder>
            = dyn_cast_rc(
                template.find("PART_Border").expect("PART_Border").clone()
            ).expect("PART_Border: Border");
        let part_header_presenter: Rc<dyn IsContentPresenter>
            = dyn_cast_rc(
                template.find("PART_HeaderPresenter").expect("PART_HeaderPresenter").clone()
            ).expect("PART_HeaderPresenter: ContentPresenter");
        let (header_align, double, color) = {
            let data = this.group_box().data.borrow();
            (data.header_align, data.double, data.color)
        };
        part_border.set_color(color);
        part_border.set_double(double);
        part_header_presenter.set_h_align(header_align);
        part_header_presenter.set_text_color(color);
    }

    pub fn template_impl(_this: &Rc<dyn IsControl>) -> Box<dyn Template> {
        Box::new(PilePanelTemplate {
            children: vec![
                Box::new(BorderTemplate {
                    name: "PART_Border".to_string(),
                    child: Some(Box::new(ContentPresenterTemplate {
                        name: "PART_ContentPresenter".to_string(),
                        text_wrapping: Some(TextWrapping::Wrap),
                        .. Default::default()
                    })),
                    .. Default::default()
                }),
                Box::new(ContentPresenterTemplate {
                    name: "PART_HeaderPresenter".to_string(),
                    show_text_trimming_marker: Some(true),
                    margin: Some(Thickness::new(1, 0, 1, 0)),
                    .. Default::default()
                }),
            ],
            .. Default::default()
        })
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
        $crate::headered_content_control_template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                $(use $path as $import;)*

                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub double: Option<bool>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color: Option<($crate::base::Fg, $crate::base::Bg)>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub header_align: Option<$crate::view::ViewHAlign>,
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
        $crate::headered_content_control_apply_template!($this, $instance, $names);
        {
            use $crate::group_box::GroupBoxExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::group_box::IsGroupBox>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.double.map(|x| obj.set_double(x));
            $this.color.map(|x| obj.set_color(x));
            $this.header_align.map(|x| obj.set_header_align(x));
        }
    };
}

group_box_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="GroupBox@Content")]
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
