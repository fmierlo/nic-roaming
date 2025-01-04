use std::{any::type_name, fmt::Debug, ops::Deref};
use std::{any::Any, cell::RefCell, cmp::PartialEq, rc::Rc};

#[derive(Default, Clone)]
pub struct Mock(Rc<RefCell<Vec<(Box<dyn Any>, &'static str)>>>);

impl Deref for Mock {
    type Target = RefCell<Vec<(Box<dyn Any>, &'static str)>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for Mock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl Mock {
    pub fn on<T: Any + Clone>(&self, value: T) {
        self.borrow_mut()
            .insert(0, (Box::new(value), type_name::<T>()));
    }

    pub fn next<T: Any + Clone>(&self) -> T {
        let (next, next_type_name) = match self.borrow_mut().pop() {
            Some(next) => next,
            None => panic!(
                "{:?}: type not found, predicate list is empty",
                type_name::<T>()
            ),
        };

        match next.downcast::<T>() {
            Ok(next) => *next,
            Err(_) => panic!(
                "{:?}: type not compatible with {:?}",
                type_name::<T>(),
                next_type_name
            ),
        }
    }

    pub fn assert<T, V, U, P>(&self, destructure: P) -> U
    where
        P: Fn(&T) -> (V, (&V, &U)),
        T: Any + Clone,
        V: Clone + PartialEq + Debug,
        U: Clone + Debug,
    {
        let next = self.next();

        let (lhs, (rhs, ret)) = destructure(&next);

        if &lhs == rhs {
            eprintln!("{}({lhs:?}) -> ret={ret:?}", type_name::<T>());
            ret.clone()
        } else {
            panic!(
                "{:?}: type value {:?} don't match value {:?}",
                type_name::<T>(),
                lhs,
                rhs
            )
        }
    }
}
