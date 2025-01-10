use std::any::{type_name, Any};
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

struct AnyTyped {
    any: Box<dyn Any>,
    type_name: fn() -> &'static str,
}

impl AnyTyped {
    fn new<T: Any>(value: T) -> AnyTyped {
        AnyTyped {
            any: Box::new(value),
            type_name: || type_name::<T>(),
        }
    }
}

impl Deref for AnyTyped {
    type Target = Box<dyn Any>;

    fn deref(&self) -> &Self::Target {
        &self.any
    }
}

impl Debug for AnyTyped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", (self.type_name)())
    }
}

#[derive(Debug)]
struct Expect(AnyTyped, AnyTyped);

impl Expect {
    fn new<T: Any, U: Any>((when, then): (fn(T), fn() -> U)) -> Expect {
        Self(AnyTyped::new(when), AnyTyped::new(then))
    }

    fn mock<T: Any + Debug, U: Any>(&self, args: T) -> Result<U, String> {
        let Self(when, then) = self;

        let when = when.downcast_ref::<fn(T)>().ok_or_else(|| {
            format!("args type mismatch: expected type {when:?}, received value {args:?}")
        })?;

        when(args);

        let then = then.downcast_ref::<fn() -> U>().ok_or_else(|| {
            let result = type_name::<U>();
            format!("return type mismatch: expected type {result:?}, returning value {then:?}")
        })?;

        Ok(then())
    }
}

#[derive(Clone, Debug, Default)]
pub struct MockStore(Arc<RefCell<Vec<Expect>>>);

impl MockStore {
    fn add_expect(&self, mock: Expect) {
        self.0.borrow_mut().insert(0, mock)
    }

    fn next_expect(&self) -> Option<Expect> {
        self.0.borrow_mut().pop()
    }

    fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }
}

#[derive(Clone, Default)]
pub struct Mock(MockStore);

impl Deref for Mock {
    type Target = MockStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for Mock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Drop for Mock {
    fn drop(&mut self) {
        if !self.is_empty() {
            panic!("pending expects: {:?}", self.0)
        }
    }
}

impl Mock {
    pub fn on_mock<T: Any + Debug, U: Any>(&self, args: T) -> Result<U, String> {
        let expect = self.next_expect().ok_or(format!(
            "args type mismatch: expecting nothing, received value {args:?}"
        ))?;

        expect.mock(args)
    }
}

pub trait MockExpect
where
    Self: Sized,
{
    fn mock(&self) -> &Mock;

    fn expect<T: Any, U: Any>(self, expect: (fn(T), fn() -> U)) -> Self {
        self.mock().add_expect(Expect::new(expect));
        self
    }
}
