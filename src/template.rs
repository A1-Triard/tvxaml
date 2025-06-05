use basic_oop::obj::IsObj;
use hashbrown::HashMap;
use std::rc::Rc;

pub struct Names {
    names: HashMap<String, Rc<dyn IsObj>>,
    clients: Option<Vec<(String, Box<dyn FnOnce(Rc<dyn IsObj>)>)>>,
}

impl Names {
    fn new() -> Self {
        Names {
            names: HashMap::new(),
            clients: Some(Vec::new()),
        }
    }

    fn register(&mut self, name: &String, obj: Rc<dyn IsObj>) {
        if self.names.insert(name.clone(), obj).is_some() {
            eprintln!("Warning: conflicting names ('{name}')");
        }
    }

    pub fn resolve(&mut self, name: String, client: Box<dyn FnOnce(Rc<dyn IsObj>)>) {
        if !name.is_empty() {
            self.clients.as_mut().unwrap().push((name, client));
        }
    }
}

impl Drop for Names {
    fn drop(&mut self) {
        for (name, client) in self.clients.take().unwrap() {
            let Some(named_obj) = self.names.get(&name) else {
                eprintln!("Warning: name not found ('{name}')");
                continue;
            };
            client(named_obj.clone())
        }
    }
}

#[typetag::serde]
pub trait Template {
    fn is_name_scope(&self) -> bool {
        false
    }

    fn name(&self) -> Option<&String> {
        None
    }

    fn create_instance(&self) -> Rc<dyn IsObj>;

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut Names);

    fn load_content(&self, names: &mut Names) -> Rc<dyn IsObj> {
        let mut local_names = if self.is_name_scope() { Some(Names::new()) } else { None };
        let names = local_names.as_mut().unwrap_or(names);
        let instance = self.create_instance();
        if let Some(name) = self.name() && !name.is_empty() {
            names.register(name, instance.clone());
        }
        self.apply(&instance, names);
        instance
    }

    fn load_root(&self) -> Rc<dyn IsObj> {
        self.load_content(&mut Names::new())
    }
}
