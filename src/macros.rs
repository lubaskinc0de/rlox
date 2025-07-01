#[macro_export]
macro_rules! rc_refcell {
    ($value:expr) => {
        ::std::rc::Rc::new(::std::cell::RefCell::new($value))
    };
}
