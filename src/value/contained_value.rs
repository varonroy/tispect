use std::{cell::UnsafeCell, marker::PhantomPinned, pin::Pin, ptr::NonNull};

use super::{parse, Value};

pub struct ContainedValue<'a> {
    source: String,
    slice: NonNull<String>,
    value: UnsafeCell<Value<'a>>,
    _pin: PhantomPinned,
}

impl<'a> ContainedValue<'a> {
    pub fn parse(source: String) -> Pin<Box<Self>> {
        let this = ContainedValue {
            source,
            slice: NonNull::dangling(),
            value: Value::Null.into(),
            _pin: PhantomPinned,
        };
        let mut pin = Box::pin(this);

        let slice = NonNull::from(&pin.source);

        unsafe {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut pin);
            let mut_ref = Pin::get_unchecked_mut(mut_ref);
            mut_ref.slice = slice;
            mut_ref.value = parse(mut_ref.slice.as_ref().as_str()).unwrap().into();
        }

        pin
    }

    pub fn get(&self) -> &Value<'a> {
        unsafe { &*(self.value.get()) }
    }

    pub fn get_mut(&self) -> &mut Value<'a> {
        unsafe { &mut *(self.value.get()) }
    }
}
