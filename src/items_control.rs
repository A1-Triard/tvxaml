use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use std::mem::replace;
use std::ptr::addr_eq;
use crate::decorator::{IsDecorator, DecoratorExt, DecoratorTemplate};
use crate::panel::{IsPanel, PanelExt};
use crate::stack_panel::StackPanelTemplate;
use crate::template::{NameResolver, Names};
use crate::view_vec::ViewVecExt;

import! { pub items_control:
    use [control crate::control];
}

struct ItemsControlData {
    items_count: usize,
    item_template: Rc<dyn Template>,
    loaded_item_templates: Vec<Rc<dyn IsView>>,
    panel_template: Rc<dyn Template>,
}

#[class_unsafe(inherits_Control)]
pub struct ItemsControl {
    data: RefCell<ItemsControlData>,
    #[non_virt]
    items_count: fn() -> usize,
    #[non_virt]
    set_items_count: fn(value: usize),
    #[non_virt]
    item_template: fn() -> Rc<dyn Template>,
    #[non_virt]
    set_item_template: fn(value: Rc<dyn Template>),
    #[non_virt]
    panel_template: fn() -> Rc<dyn Template>,
    #[non_virt]
    set_panel_template: fn(value: Rc<dyn Template>),
    #[over]
    template: (),
    #[over]
    update_override: (),
}

impl ItemsControl {
    pub fn new() -> Rc<dyn IsItemsControl> {
        let res: Rc<dyn IsItemsControl>
            = Rc::new(unsafe { Self::new_raw(ITEMS_CONTROL_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        ItemsControl {
            control: unsafe { Control::new_raw(vtable) },
            data: RefCell::new(ItemsControlData {
                items_count: 0,
                item_template: Rc::new(ViewTemplate::default()),
                loaded_item_templates: Vec::new(),
                panel_template: Rc::new(StackPanelTemplate::default()),
            }),
        }
    }

    pub fn update_override_impl(this: &Rc<dyn IsControl>, template: &Names) {
        let this: Rc<dyn IsItemsControl> = dyn_cast_rc(this.clone()).unwrap();
        let part_items_presenter: Rc<dyn IsDecorator>
            = dyn_cast_rc(
                template.find("PART_ItemsPresenter").expect("PART_ItemsPresenter").clone()
            ).expect("PART_ItemsPresenter: Decorator");
        let (panel, old_loaded_item_templates, new_loaded_item_templates) = {
            let mut data = this.items_control().data.borrow_mut();
            let panel: Rc<dyn IsPanel> = dyn_cast_rc(data.panel_template.load_root().0).expect("Panel");
            let new_loaded_item_templates: Vec<Rc<dyn IsView>>
                = (0 .. data.items_count)
                    .map(|_| dyn_cast_rc(data.item_template.load_root().0).expect("View")).collect()
                ;
            let old_loaded_item_templates
                = replace(&mut data.loaded_item_templates, new_loaded_item_templates.clone());
            (panel, old_loaded_item_templates, new_loaded_item_templates)
        };
        for (i, old_loaded_item_template) in old_loaded_item_templates.into_iter().enumerate() {
            this._raise_unbind(&old_loaded_item_template, i);
        }
        let panel_children = panel.children();
        part_items_presenter.set_child(Some(panel));
        for (i, new_loaded_item_template) in new_loaded_item_templates.into_iter().enumerate() {
            panel_children.push(new_loaded_item_template.clone());
            this._raise_bind(&new_loaded_item_template, i);
        }
    }

    pub fn items_count_impl(this: &Rc<dyn IsItemsControl>) -> usize {
        this.items_control().data.borrow().items_count
    }

    pub fn set_items_count_impl(this: &Rc<dyn IsItemsControl>, value: usize) {
        {
            let mut data = this.items_control().data.borrow_mut();
            if data.items_count == value { return; }
            data.items_count = value;
        };
        this.update();
    }

    pub fn item_template_impl(this: &Rc<dyn IsItemsControl>) -> Rc<dyn Template> {
        this.items_control().data.borrow().item_template.clone()
    }

    pub fn set_item_template_impl(this: &Rc<dyn IsItemsControl>, value: Rc<dyn Template>) {
        {
            let mut data = this.items_control().data.borrow_mut();
            if addr_eq(Rc::as_ptr(&data.item_template), Rc::as_ptr(&value)) { return; }
            data.item_template = value;
        }
        this.update();
    }

    pub fn panel_template_impl(this: &Rc<dyn IsItemsControl>) -> Rc<dyn Template> {
        this.items_control().data.borrow().panel_template.clone()
    }

    pub fn set_panel_template_impl(this: &Rc<dyn IsItemsControl>, value: Rc<dyn Template>) {
        {
            let mut data = this.items_control().data.borrow_mut();
            if addr_eq(Rc::as_ptr(&data.panel_template), Rc::as_ptr(&value)) { return; }
            data.panel_template = value;
        }
        this.update();
    }

    pub fn template_impl(_this: &Rc<dyn IsControl>) -> Box<dyn Template> {
        Box::new(DecoratorTemplate {
            name: "PART_ItemsPresenter".to_string(),
            .. Default::default()
        })
    }
}

#[macro_export]
macro_rules! items_control_template {
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
                pub items_count: Option<usize>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub item_template: Option<Box<dyn $crate::template::Template>>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub panel_template: Option<Box<dyn $crate::template::Template>>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! items_control_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::control_apply_template!($this, $instance, $names);
        {
            use $crate::items_control::ItemsControlExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::items_control::IsItemsControl>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.items_count.map(|x| obj.set_items_count(x));
            $this.item_template.as_ref().map(|x| obj.set_item_template($crate::alloc_rc_Rc::from(x.clone())));
            $this.panel_template.as_ref().map(|x| obj.set_panel_template($crate::alloc_rc_Rc::from(x.clone())));
        }
    };
}

items_control_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
    #[serde(rename="ItemsControl@ItemTemplate")]
    pub struct ItemsControlTemplate in template { }
}

#[typetag::serde(name="ItemsControl")]
impl Template for ItemsControlTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        ItemsControl::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        items_control_apply_template!(this, instance, names);
    }
}
