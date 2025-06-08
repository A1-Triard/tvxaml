#![feature(iter_advance_by)]
#![feature(macro_metavar_expr_concat)]
#![feature(slice_from_ptr_range)]
#![feature(trusted_len)]

#[doc(hidden)]
pub use std::rc::Rc as alloc_rc_Rc;
#[doc(hidden)]
pub use dynamic_cast::dyn_cast_rc as dynamic_cast_dyn_cast_rc;
#[doc(hidden)]
pub use int_vec_2d::Vector as int_vec_2d_Vector;
#[doc(hidden)]
pub use int_vec_2d::Thickness as int_vec_2d_Thickness;
#[doc(hidden)]
pub use int_vec_2d::Point as int_vec_2d_Point;
#[doc(hidden)]
pub use int_vec_2d::HAlign as int_vec_2d_HAlign;
#[doc(hidden)]
pub use tvxaml_screen_base::Fg as tvxaml_screen_base_Fg;
#[doc(hidden)]
pub use tvxaml_screen_base::Bg as tvxaml_screen_base_Bg;

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
