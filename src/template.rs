use basic_oop::obj::TObj;
use std::rc::Rc;

#[typetag::serde]
pub trait Template {
    fn create_instance(&self) -> Rc<dyn TObj>;

    fn apply(&self, instance: &Rc<dyn TObj>);

    fn load_content(&self) -> Rc<dyn TObj> {
        let instance = self.create_instance();
        self.apply(&instance);
        instance
    }
}
