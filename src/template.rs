use basic_oop::obj::IsObj;
use std::rc::Rc;

#[typetag::serde]
pub trait Template {
    fn create_instance(&self) -> Rc<dyn IsObj>;

    fn apply(&self, instance: &Rc<dyn IsObj>);

    fn load_content(&self) -> Rc<dyn IsObj> {
        let instance = self.create_instance();
        self.apply(&instance);
        instance
    }
}
