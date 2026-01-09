/// Wraps anything into ``Rc::new(RefCell::new())``
#[macro_export]
macro_rules! rc_refcell {
    ($value:expr) => {
        ::std::rc::Rc::new(::std::cell::RefCell::new($value))
    };
}

/// Casts object to concrete type
#[macro_export]
macro_rules! cast {
    ($obj:expr => $type:ty) => {{
        if !isinstance!($obj, $type) {
            let borrowed = $obj.borrow();
            Err(RuntimeErrorKind::TypeError {
                expected: stringify!($type).to_string(),
                provided: borrowed.type_name(),
            })
        } else {
            Ok(std::cell::Ref::map($obj.borrow(), |obj| {
                (obj as &dyn std::any::Any).downcast_ref::<$type>().unwrap()
            }))
        }
    }};
}

// Checks that object is an instance of type
#[macro_export]
macro_rules! isinstance {
    ($obj:expr, $type:ty) => {{
        let borrowed = $obj.borrow();
        (&*borrowed as &dyn std::any::Any).is::<$type>()
    }};
}

#[macro_export]
macro_rules! calc {
    ($a:expr, $b:expr, $op:expr) => {{
        match $op {
            "+" => $a + $b,
            "-" => $a - $b,
            "*" => $a * $b,
            "/" => $a / $b,
            _ => panic!("Unsupported operator: {}", $op),
        }
    }};
}
