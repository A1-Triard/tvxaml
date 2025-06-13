#![feature(iter_advance_by)]
#![feature(macro_metavar_expr_concat)]
#![feature(slice_from_ptr_range)]
#![feature(trusted_len)]

#[doc(hidden)]
pub use std::rc::Rc as alloc_rc_Rc;
#[doc(hidden)]
pub use dynamic_cast::dyn_cast_rc as dynamic_cast_dyn_cast_rc;

mod arena;

pub mod base;
pub mod xaml;
pub mod obj_col;
pub mod template;
pub mod render_port;
pub mod app;
pub mod event_handler;
pub mod view;
pub mod view_vec;
pub mod panel;
pub mod decorator;
pub mod stack_panel;
pub mod canvas;
pub mod dock_panel;
pub mod static_text;
pub mod frame;
pub mod check_box;
pub mod radio_button;
pub mod background;
pub mod button;
pub mod headered_decorator;
pub mod group_box;
pub mod content_presenter;
pub mod control;
pub mod content_control;
pub mod headered_content_control;
