use expect::ExpectList;
use std::any::{type_name, Any};
use std::cell::RefCell;
use std::default::Default;
use std::fmt::Debug;
use std::sync::{Arc, LazyLock, Mutex};
use std::thread::LocalKey;

mod expect {
    use std::any::Any;
    use std::fmt::Debug;

    trait AsAny {
        fn as_any(self) -> Box<dyn Any>;
    }

    impl<T: Any> AsAny for T {
        fn as_any(self) -> Box<dyn Any> {
            Box::new(self)
        }
    }

    trait AsType {
        fn as_type<T: Any>(self, expect: &dyn Expect) -> Result<T, &'static str>;
    }

    impl AsType for Box<dyn Any> {
        fn as_type<T: Any>(self, expect: &dyn Expect) -> Result<T, &'static str> {
            self.downcast::<T>()
                .map_err(|_| expect.type_name())
                .map(|value| *value)
        }
    }

    pub(super) trait Expect: Send {
        fn on_mock(&self, when: Box<dyn Any>) -> Result<Box<dyn Any>, &'static str>;
        fn type_name(&self) -> &'static str;
    }

    impl Debug for dyn Expect {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.type_name())
        }
    }

    impl dyn Expect {
        pub(super) fn mock<T: Any, U: Any>(&self, when: T) -> Result<U, &'static str> {
            let then = self.on_mock(when.as_any())?;
            Ok(then.as_type(self)?)
        }
    }

    impl<T: Any, U: Any> Expect for fn(T) -> U {
        fn on_mock(&self, when: Box<dyn Any>) -> Result<Box<dyn Any>, &'static str> {
            let then = self(when.as_type(self)?);
            Ok(then.as_any())
        }

        fn type_name(&self) -> &'static str {
            std::any::type_name::<fn(T) -> U>()
        }
    }

    #[derive(Debug, Default)]
    pub struct ExpectList {
        list: Vec<Box<dyn Expect>>,
    }

    impl ExpectList {
        pub(super) fn clear(&mut self) {
            self.list.clear();
        }

        pub(super) fn add<T: Any, U: Any>(&mut self, expect: fn(T) -> U) {
            self.list.insert(0, Box::new(expect));
        }

        pub(super) fn next(&mut self) -> Option<Box<dyn Expect>> {
            self.list.pop()
        }

        fn is_empty(&self) -> bool {
            self.list.is_empty()
        }
    }

    impl Drop for ExpectList {
        fn drop(&mut self) {
            if !self.is_empty() {
                panic!("Mockdown error, pending expects: {:?}", self.list)
            }
        }
    }
}

#[derive(Default)]
pub struct Mockdown {
    expects: ExpectList,
}

impl Mockdown {
    pub fn new() -> Mockdown {
        Default::default()
    }

    pub fn thread_local() -> RefCell<Mockdown> {
        Default::default()
    }

    pub const fn static_global() -> LazyLock<Arc<Mutex<Mockdown>>> {
        LazyLock::new(|| Default::default())
    }

    pub fn clone(mockdown: &Arc<Mutex<Mockdown>>) -> Arc<Mutex<Mockdown>> {
        Arc::clone(mockdown)
    }

    fn clear(&mut self) {
        self.expects.clear();
    }

    fn expect<T: Any, U: Any>(&mut self, expect: fn(T) -> U) {
        self.expects.add(expect);
    }

    fn type_error<T: Any + Debug, U: Any>(expect: &str) -> String {
        let received = type_name::<fn(T) -> U>();
        format!("Mockdown error, expect type mismatch: expecting {expect:?}, received {received:?}")
    }

    fn mock<T: Any + Debug, U: Any>(&mut self, args: T) -> Result<U, String> {
        let expect = self.expects.next().ok_or_else(|| {
            self.expects.clear();
            Self::type_error::<T, U>("nothing")
        })?;

        let result = expect.mock(args).map_err(|expect| {
            self.expects.clear();
            Self::type_error::<T, U>(expect)
        })?;

        Ok(result)
    }
}

pub trait StaticMockdown {
    fn clear(&'static self) -> &'static Self;
    fn expect<T: Any, U: Any>(&'static self, expect: fn(T) -> U) -> &'static Self;
    fn mock<T: Any + Debug, U: Any>(&'static self, args: T) -> Result<U, String>;
}

impl StaticMockdown for RefCell<Mockdown> {
    fn clear(&'static self) -> &'static Self {
        self.borrow_mut().clear();
        self
    }

    fn expect<T: Any, U: Any>(&'static self, expect: fn(T) -> U) -> &'static Self {
        self.borrow_mut().expect(expect);
        self
    }

    fn mock<T: Any + Debug, U: Any>(&'static self, args: T) -> Result<U, String> {
        self.borrow_mut().mock(args)
    }
}

impl StaticMockdown for LocalKey<RefCell<Mockdown>> {
    fn clear(&'static self) -> &'static Self {
        self.with_borrow_mut(|mock| mock.clear());
        self
    }

    fn expect<T: Any, U: Any>(&'static self, expect: fn(T) -> U) -> &'static Self {
        self.with_borrow_mut(|mock| mock.expect(expect));
        self
    }

    fn mock<T: Any + Debug, U: Any>(&'static self, args: T) -> Result<U, String> {
        self.with_borrow_mut(|mock| mock.mock::<T, U>(args))
    }
}

impl StaticMockdown for LazyLock<Arc<Mutex<Mockdown>>> {
    fn clear(&'static self) -> &'static Self {
        self.lock().unwrap().clear();
        self
    }

    fn expect<T: Any, U: Any>(&'static self, expect: fn(T) -> U) -> &'static Self {
        self.lock().unwrap().expect(expect);
        self
    }

    fn mock<T: Any + Debug, U: Any>(&'static self, args: T) -> Result<U, String> {
        self.lock().unwrap().mock(args)
    }
}
