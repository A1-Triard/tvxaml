use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use either::{Either, Left, Right};
use std::cell::RefCell;
use std::ptr::addr_eq;
use crate::base::{option_addr_eq, TextWrapping};
use crate::background::{IsBackground, BackgroundExt, BackgroundTemplate};
use crate::dock_panel::{Dock, DockLayoutTemplate, DockPanelTemplate};
use crate::static_text::{IsStaticText, StaticTextExt, StaticTextTemplate, TextTrimmingMarkerTemplate};
use crate::template::{NameResolver, Names};

import! { pub content_presenter:
    use [view crate::view];
    use crate::base::{Fg, Bg};
    use crate::template::Template;
}

struct ContentPresenterData {
    content: Option<Rc<dyn IsView>>,
    text: Rc<String>,
    text_color: (Fg, Bg),
    text_wrapping: TextWrapping,
    show_text_trimming_marker: bool,
    actual_content: Option<Either<(Rc<dyn IsView>, Names), Rc<dyn IsView>>>,
}

#[class_unsafe(inherits_View)]
pub struct ContentPresenter {
    data: RefCell<ContentPresenterData>,
    #[non_virt]
    content: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    set_content: fn(value: Option<Rc<dyn IsView>>),
    #[non_virt]
    text: fn() -> Rc<String>,
    #[non_virt]
    set_text: fn(value: Rc<String>),
    #[non_virt]
    text_wrapping: fn() -> TextWrapping,
    #[non_virt]
    set_text_wrapping: fn(value: TextWrapping),
    #[non_virt]
    show_text_trimming_marker: fn() -> bool,
    #[non_virt]
    set_show_text_trimming_marker: fn(value: bool),
    #[non_virt]
    text_color: fn() -> (Fg, Bg),
    #[non_virt]
    set_text_color: fn(value: (Fg, Bg)),
    #[over]
    visual_children_count: (),
    #[over]
    visual_child: (),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
    #[virt]
    text_template: fn() -> Box<dyn Template>,
    #[non_virt]
    update: fn(),
    #[virt]
    update_override: fn(text_template: &Names),
}

impl ContentPresenter {
    pub fn new() -> Rc<dyn IsContentPresenter> {
        let res: Rc<dyn IsContentPresenter>
            = Rc::new(unsafe { Self::new_raw(CONTENT_PRESENTER_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        ContentPresenter {
            view: unsafe { View::new_raw(vtable) },
            data: RefCell::new(ContentPresenterData {
                content: None,
                text: Rc::new(String::new()),
                text_color: (Fg::LightGray, Bg::None),
                text_wrapping: TextWrapping::NoWrap,
                show_text_trimming_marker: false,
                actual_content: None,
            }),
        }
    }

    pub fn text_template_impl(_this: &Rc<dyn IsContentPresenter>) -> Box<dyn Template> {
        Box::new(DockPanelTemplate {
            children: vec![
                Box::new(BackgroundTemplate {
                    name: "PART_Padding".to_string(),
                    pattern: Some(" ".to_string()),
                    width: Some(Some(1)),
                    height: Some(Some(1)),
                    layout: Some(Box::new(DockLayoutTemplate {
                        dock: Some(Dock::Left),
                        .. Default::default()
                    })),
                    .. Default::default()
                }),
                Box::new(TextTrimmingMarkerTemplate {
                    name: "PART_Trimming".to_string(),
                    text: "PART_Text".to_string(),
                    layout: Some(Box::new(DockLayoutTemplate {
                        dock: Some(Dock::Right),
                        .. Default::default()
                    })),
                    .. Default::default()
                }),
                Box::new(StaticTextTemplate {
                    name: "PART_Text".to_string(),
                    .. Default::default()
                }),
            ],
            .. Default::default()
        })
    }

    pub fn update_impl(this: &Rc<dyn IsContentPresenter>) {
        let text_template = {
            let data = this.content_presenter().data.borrow();
            data.actual_content.as_ref().and_then(|x| x.as_ref().left()).map(|x| x.1.clone())
        };
        if let Some(text_template) = text_template {
            this.update_override(&text_template);
        }
    }

    pub fn update_override_impl(this: &Rc<dyn IsContentPresenter>, text_template: &Names) {
        let part_text: Rc<dyn IsStaticText>
            = dyn_cast_rc(
                text_template.find("PART_Text").expect("PART_Text").clone()
            ).expect("PART_Text: StaticText");
        let part_padding: Rc<dyn IsBackground>
            = dyn_cast_rc(
                text_template.find("PART_Padding").expect("PART_Padding").clone()
            ).expect("PART_Padding: Background");
        let part_trimming: Rc<dyn IsView>
            = dyn_cast_rc(
                text_template.find("PART_Trimming").expect("PART_Trimming").clone()
            ).expect("PART_Trimming: View");
        let (text, text_color, text_wrapping, show_trimming_marker) = {
            let data = this.content_presenter().data.borrow();
            (data.text.clone(), data.text_color, data.text_wrapping, data.show_text_trimming_marker)
        };
        part_text.set_text(text);
        part_text.set_color(text_color);
        part_text.set_text_wrapping(text_wrapping);
        part_padding.set_color(text_color);
        part_padding.set_visibility(
            if show_trimming_marker { Visibility:: Visible } else { Visibility::Collapsed }
        );
        part_trimming.set_visibility(
            if show_trimming_marker { Visibility:: Visible } else { Visibility::Collapsed }
        );
    }

    pub fn content_impl(this: &Rc<dyn IsContentPresenter>) -> Option<Rc<dyn IsView>> {
        this.content_presenter().data.borrow().content.clone()
    }

    pub fn set_content_impl(this: &Rc<dyn IsContentPresenter>, value: Option<Rc<dyn IsView>>) {
        let new_actual_content = {
            let mut data = this.content_presenter().data.borrow_mut();
            if option_addr_eq(data.content.as_ref().map(Rc::as_ptr), value.as_ref().map(Rc::as_ptr)) { return; }
            data.content = value.clone();
            if data.actual_content.as_ref().map_or(false, |x| x.is_right()) != value.is_some() {
                if let Some(value) = value {
                    Some(Some(Right(value)))
                } else if data.text.is_empty() {
                    Some(None)
                } else {
                    Some(Some(Left(())))
                }
            } else {
                if let Some(value) = value {
                    Some(Some(Right(value)))
                } else {
                    None
                }
            }
        };
        if let Some(new_actual_content) = new_actual_content {
            if let Some(new_actual_content) = new_actual_content {
                match new_actual_content {
                    Left(()) => {
                        let (text, names) = this.text_template().load_root();
                        let text: Rc<dyn IsView> = dyn_cast_rc(text).expect("View");
                        Self::set_actual_content(this, Some(Left((text, names.clone()))));
                        this.update_override(&names);
                    },
                    Right(x) => Self::set_actual_content(this, Some(Right(x))),
                }
            } else {
                Self::set_actual_content(this, None);
            }
        }
    }

    pub fn text_impl(this: &Rc<dyn IsContentPresenter>) -> Rc<String> {
        this.content_presenter().data.borrow().text.clone()
    }

    pub fn set_text_impl(this: &Rc<dyn IsContentPresenter>, value: Rc<String>) {
        let new_actual_content = {
            let mut data = this.content_presenter().data.borrow_mut();
            if addr_eq(Rc::as_ptr(&data.text), Rc::as_ptr(&value)) { return; }
            data.text = value.clone();
            if data.actual_content.as_ref().map_or(false, |x| x.is_left()) != value.is_empty() {
                if value.is_empty() {
                    None
                } else {
                    Some(Some(Right(())))
                }
            } else {
                if value.is_empty() {
                    Some(None)
                } else if data.actual_content.is_some() {
                    None
                } else {
                    Some(Some(Left(())))
                }
            }
        };
        if let Some(new_actual_content) = new_actual_content {
            if let Some(new_actual_content) = new_actual_content {
                match new_actual_content {
                    Left(()) => {
                        let (text, names) = this.text_template().load_root();
                        let text: Rc<dyn IsView> = dyn_cast_rc(text).expect("View");
                        Self::set_actual_content(this, Some(Left((text, names.clone()))));
                        this.update_override(&names);
                    },
                    Right(()) => {
                        let names = {
                            let data = this.content_presenter().data.borrow();
                            data.actual_content.as_ref().unwrap().as_ref().left().unwrap().1.clone()
                        };
                        this.update_override(&names);
                    },
                }
            } else {
                Self::set_actual_content(this, None);
            }
        }
    }

    fn set_actual_content(
        this: &Rc<dyn IsContentPresenter>,
        value: Option<Either<(Rc<dyn IsView>, Names), Rc<dyn IsView>>>,
    ) {
        let content = {
            let data = this.content_presenter().data.borrow();
            data.actual_content.as_ref().map(|x| x.as_ref().either(|x| x.0.clone(), |x| x.clone()))
        };
        if let Some(content) = content {
            this.remove_visual_child(&content);
            content._set_visual_parent(None);
            content._set_layout_parent(None);
        }
        let content = value.as_ref().map(|x| x.as_ref().either(|x| x.0.clone(), |x| x.clone()));
        this.content_presenter().data.borrow_mut().actual_content = value;
        let this: Rc<dyn IsView> = this.clone();
        if let Some(content) = content {
            content._set_layout_parent(Some(&this));
            content._set_visual_parent(Some(&this));
            this.add_visual_child(&content);
        }
        this.invalidate_measure();
    }

    pub fn text_color_impl(this: &Rc<dyn IsContentPresenter>) -> (Fg, Bg) {
        this.content_presenter().data.borrow().text_color
    }

    pub fn set_text_color_impl(this: &Rc<dyn IsContentPresenter>, value: (Fg, Bg)) {
        {
            let mut data = this.content_presenter().data.borrow_mut();
            if data.text_color == value { return; }
            data.text_color = value;
        };
        this.update();
    }

    pub fn text_wrapping_impl(this: &Rc<dyn IsContentPresenter>) -> TextWrapping {
        this.content_presenter().data.borrow().text_wrapping
    }

    pub fn set_text_wrapping_impl(this: &Rc<dyn IsContentPresenter>, value: TextWrapping) {
        {
            let mut data = this.content_presenter().data.borrow_mut();
            if data.text_wrapping == value { return; }
            data.text_wrapping = value;
        };
        this.update();
    }

    pub fn show_text_trimming_marker_impl(this: &Rc<dyn IsContentPresenter>) -> bool {
        this.content_presenter().data.borrow().show_text_trimming_marker
    }

    pub fn set_show_text_trimming_marker_impl(this: &Rc<dyn IsContentPresenter>, value: bool) {
        {
            let mut data = this.content_presenter().data.borrow_mut();
            if data.show_text_trimming_marker == value { return; }
            data.show_text_trimming_marker = value;
        };
        this.update();
    }

    pub fn visual_children_count_impl(this: &Rc<dyn IsView>) -> usize {
        let this: Rc<dyn IsContentPresenter> = dyn_cast_rc(this.clone()).unwrap();
        if this.content_presenter().data.borrow().actual_content.is_some() {
            1
        } else {
            0
        }
    }

    pub fn visual_child_impl(this: &Rc<dyn IsView>, index: usize) -> Rc<dyn IsView> {
        let this: Rc<dyn IsContentPresenter> = dyn_cast_rc(this.clone()).unwrap();
        assert_eq!(index, 0);
        this.content_presenter().data.borrow().actual_content
            .as_ref().unwrap().as_ref().either(|x| x.0.clone(), |x| x.clone())
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsContentPresenter> = dyn_cast_rc(this.clone()).unwrap();
        let content
            = this.content_presenter().data.borrow().actual_content
            .as_ref().map(|x| x.as_ref().either(|x| x.0.clone(), |x| x.clone()));
        if let Some(content) = content {
            content.measure(w, h);
            content.desired_size()
        } else {
            Vector::null()
        }
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsContentPresenter> = dyn_cast_rc(this.clone()).unwrap();
        let content
            = this.content_presenter().data.borrow().actual_content
            .as_ref().map(|x| x.as_ref().either(|x| x.0.clone(), |x| x.clone()));
        if let Some(content) = content {
            content.arrange(bounds);
            content.render_bounds().size
        } else {
            Vector::null()
        }
    }
}

#[macro_export]
macro_rules! content_presenter_template {
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
        $crate::view_template! {
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
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub text_wrapping: Option<$crate::base::TextWrapping>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub show_text_trimming_marker: Option<bool>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! content_presenter_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::view_apply_template!($this, $instance, $names);
        {
            use $crate::content_presenter::ContentPresenterExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::content_presenter::IsContentPresenter>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.content.as_ref().map(|x|
                obj.set_content(Some($crate::dynamic_cast_dyn_cast_rc(x.load_content($names)).unwrap()))
            );
            $this.text.as_ref().map(|x| obj.set_text($crate::alloc_rc_Rc::new(x.clone())));
            $this.text_color.map(|x| obj.set_text_color(x));
            $this.text_wrapping.map(|x| obj.set_text_wrapping(x));
            $this.show_text_trimming_marker.map(|x| obj.set_show_text_trimming_marker(x));
        }
    };
}

content_presenter_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="ContentPresenter@Content")]
    pub struct ContentPresenterTemplate in template { }
}

#[typetag::serde(name="ContentPresenter")]
impl Template for ContentPresenterTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        ContentPresenter::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        content_presenter_apply_template!(this, instance, names);
    }
}
