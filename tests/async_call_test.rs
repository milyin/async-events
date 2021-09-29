use futures::{
    executor::{LocalPool, LocalSpawner},
    task::LocalSpawnExt,
};
use loopa::{self, EventSubscribers, Handle, HandleSupport};
use std::{
    any::Any,
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock, Weak},
    task::Waker,
};

#[derive(Clone)]
enum CounterEvent {
    Incremented,
}
struct Counter {
    value: usize,
    handle_support: HandleSupport<Self>,
}

impl Counter {
    pub fn new() -> Arc<RwLock<Self>> {
        let this = Arc::new(RwLock::new(Self {
            value: 0,
            handle_support: HandleSupport::new(),
        }));
        this.write().unwrap().handle_support.set_object(&this);
        this
    }
    pub fn handle(&self) -> HCounter {
        HCounter(self.handle_support.handle())
    }
    pub fn inc(&mut self) {
        self.value += 1;
    }
    pub fn value(&self) -> usize {
        self.value
    }
}

#[derive(Clone)]
struct HCounter(Handle);

impl HCounter {
    pub async fn inc(&self) -> Result<(), loopa::Error> {
        self.0.call_mut(|counter: &mut Counter| counter.inc()).await
    }
    pub async fn value(&self) -> Result<usize, loopa::Error> {
        self.0.call(|counter: &Counter| counter.value()).await
    }
}

#[test]
fn test_handle_call() {
    let value = Rc::new(RefCell::new(None));
    let value_r = value.clone();
    let counter = Counter::new();
    let hcounter = counter.read().unwrap().handle();
    let future = async move {
        let v = hcounter.value().await.unwrap();
        *(value.borrow_mut()) = Some(v);
    };
    let mut pool = LocalPool::new();
    pool.spawner().spawn_local(future).unwrap();
    pool.run_until_stalled();
    assert!(value_r.borrow().is_some())
}

// #[test]
// fn test_handle_call_mut() {
//     let value = Rc::new(RefCell::new(0));
//     let value_r = value.clone();
//     let mut pool = Pool::new();
//     let hcounter = HCounter::new(&mut pool);
//     let future = async move {
//         hcounter.inc().await?;
//         let v = hcounter.value().await?;
//         *(value.borrow_mut()) = v;
//         Ok(())
//     };
//     loopa::spawn::<loopa::Error, _>(pool.spawner(), future);
//     pool.run_until_stalled();
//     assert!(*value_r.borrow() == 1)
// }

// fn test_send_receive_event() {}
