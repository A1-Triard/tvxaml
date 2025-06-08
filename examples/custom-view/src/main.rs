use dynamic_cast::dyn_cast_rc;
use std::process::ExitCode;
use std::rc::Rc;
use tvxaml_screen_base::Key;
use tvxaml::app::{App, AppExt};
use tvxaml::template::Template;
use tvxaml::view::{IsView, ViewExt};
use tvxaml::xaml::{self};

fn main() -> ExitCode {
    ExitCode::from(start_and_print_err())
}

fn start_and_print_err() -> u8 {
    match start() {
        Err(e) => {
           eprintln!("{e}");
            1
        },
        Ok(exit_code) => exit_code
    }
}

fn start() -> Result<u8, tvxaml_screen_base::Error> {
    let screen = unsafe { tvxaml_screen_ncurses::init(None, None) }?;
    let xaml = include_str!("ui.xaml");
    let ui: Box<dyn Template> = xaml::from_str(xaml).unwrap();
    let (root, _) = ui.load_root();
    let app = App::new(screen);
    let root: Rc<dyn IsView> = dyn_cast_rc(root).unwrap();
    {
        let app = app.clone();
        root.handle_key(Some(Box::new(move |key, _| { if key == Key::Escape { app.quit(); }; true })));
    }
    app.run(root.clone(), Some(&mut || app.focus(Some(&root), true)))
}
