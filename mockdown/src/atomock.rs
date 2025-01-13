use std::{
    any::{type_name, Any},
    cell::RefCell,
    fmt::Debug,
    ops::Deref,
    sync::Arc,
};

type AnyTyped = (Box<dyn Any>, fn() -> &'static str);

fn new_any_typed<T: Any>(value: T) -> AnyTyped {
    (Box::new(value), || type_name::<T>())
}

type Expect = (
    (Box<dyn Any>, fn() -> &'static str),
    (Box<dyn Any>, fn() -> &'static str),
);

fn new_expect<T: Any, U: Any>((when, then): (fn(T), fn() -> U)) -> Expect {
    (new_any_typed(when), new_any_typed(then))
}

fn mock<T: Any + Debug, U: Any>(expect: &Expect, args: T) -> Result<U, String> {
    let (when, then) = expect;

    let when = when.0.downcast_ref::<fn(T)>().ok_or_else(|| {
        let when = (when.1)();
        format!("args type mismatch: expected type {when:?}, received value {args:?}")
    })?;

    when(args);

    let then = then.0.downcast_ref::<fn() -> U>().ok_or_else(|| {
        let result = type_name::<U>();
        let then = (then.1)();
        format!("return type mismatch: expected type {result:?}, returning value {then:?}")
    })?;

    Ok(then())
}

type MockStore = Arc<RefCell<Vec<Expect>>>;

struct SafeMockStore(MockStore);

impl Deref for SafeMockStore {
    type Target = MockStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for SafeMockStore {
    fn drop(&mut self) {
        on_drop(&self);
    }
}

fn add_expect(store: &MockStore, mock: Expect) {
    store.borrow_mut().insert(0, mock)
}

fn next_expect(store: &MockStore) -> Option<Expect> {
    store.borrow_mut().pop()
}

fn is_empty(store: &MockStore) -> bool {
    store.borrow().is_empty()
}

fn on_drop(store: &MockStore) {
    if !is_empty(store) {
        panic!("pending expects: {:?}", store)
    }
}

fn expect<T: Any, U: Any>(store: &MockStore, expect: (fn(T), fn() -> U)) {
    add_expect(store, new_expect(expect));
}

pub fn on_mock<T: Any + Debug, U: Any>(store: &MockStore, args: T) -> Result<U, String> {
    let expect = next_expect(store).ok_or(format!(
        "args type mismatch: expecting nothing, received value {args:?}"
    ))?;

    mock(&expect, args)
}
