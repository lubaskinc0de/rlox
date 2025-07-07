/// Wraps anything into ``Rc::new(RefCell::new())``
#[macro_export]
macro_rules! rc_refcell {
    ($value:expr) => {
        ::std::rc::Rc::new(::std::cell::RefCell::new($value))
    };
}

/// Casts object to concrete type, do not forget to check object type before cast otherwise you will get panic
#[macro_export]
macro_rules! cast {
    ($obj:expr => $type:ty) => {
        (&*$obj as &dyn Any)
            .downcast_ref::<$type>()
            .expect(concat!("Failed to downcast to ", stringify!($type)))
    };
}

// Checks that object is an instance of type
#[macro_export]
macro_rules! isinstance {
    ($obj:expr, $type:ty) => {
        (&*$obj as &dyn Any).is::<$type>()
    };
}
