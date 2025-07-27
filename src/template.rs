use basic_oop::obj::IsObj;
use dyn_clone::{DynClone, clone_trait_object};
use hashbrown::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct Names {
    map: HashMap<String, Rc<dyn IsObj>>,
}

impl Names {
    fn new() -> Self {
        Names { map: HashMap::new() }
    }

    fn register(&mut self, name: &str, obj: Rc<dyn IsObj>) {
        if self.map.insert(name.to_string(), obj).is_some() {
            eprintln!("Warning: conflicting names ('{name}')");
        }
    }

    pub fn find(&self, name: &str) -> Option<&Rc<dyn IsObj>> {
        self.map.get(name)
    }
}

pub struct NameResolver {
    names: Names,
    clients: Vec<(String, Box<dyn FnOnce(Rc<dyn IsObj>)>, Option<Box<dyn FnOnce() -> Rc<dyn IsObj>>>)>,
}

impl NameResolver {
    fn new() -> Self {
        NameResolver {
            names: Names::new(),
            clients: Vec::new(),
        }
    }

    pub fn resolve(&mut self, name: String, client: Box<dyn FnOnce(Rc<dyn IsObj>)>) {
        if !name.is_empty() {
            self.clients.push((name, client, None));
        }
    }

    pub fn resolve_or_create(
        &mut self,
        name: String,
        client: Box<dyn FnOnce(Rc<dyn IsObj>)>,
        create: Box<dyn FnOnce() -> Rc<dyn IsObj>>,
    ) {
        if !name.is_empty() {
            self.clients.push((name, client, Some(create)));
        }
    }

    fn finish(mut self) -> Names {
        for (name, client, factory) in self.clients {
            let named_obj = if let Some(named_obj) = self.names.map.get(&name) {
                named_obj.clone()
            } else {
                if let Some(factory) = factory {
                    let named_obj = factory();
                    self.names.register(&name, named_obj.clone());
                    named_obj
                } else {
                    eprintln!("Warning: name not found ('{name}')");
                    continue;
                }
            };
            client(named_obj)
        }
        self.names
    }
}

#[typetag::serde]
pub trait Template: DynClone {
    fn is_name_scope(&self) -> bool {
        false
    }

    fn name(&self) -> Option<&String> {
        None
    }

    fn create_instance(&self) -> Rc<dyn IsObj>;

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver);

    fn load_content(&self, names: &mut NameResolver) -> Rc<dyn IsObj> {
        let mut local_names = if self.is_name_scope() { Some(NameResolver::new()) } else { None };
        let names = local_names.as_mut().unwrap_or(names);
        let instance = self.create_instance();
        if let Some(name) = self.name() && !name.is_empty() {
            names.names.register(name, instance.clone());
        }
        self.apply(&instance, names);
        local_names.map(|x| x.finish());
        instance
    }

    fn load_root(&self) -> (Rc<dyn IsObj>, Names) {
        let mut name_resolver = NameResolver::new();
        let root = self.load_content(&mut name_resolver);
        let names = name_resolver.finish();
        (root, names)
    }
}

clone_trait_object!(Template);

#[macro_export]
macro_rules! template {
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
        mod $mod {
            $(use $path as $import;)*

            $(#[$attr])*
            pub struct $name {
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
        $vis use $mod::$name;
    };
}
