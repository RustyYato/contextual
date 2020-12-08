#[doc(hidden)]
pub mod macros {
    pub use std::thread_local;
}

#[macro_export]
macro_rules! try_get {
    (let $name:ident: $context:ident) => {
        let stack_pin = unsafe { $crate::StackPin::new() };
        let stack_pin = &stack_pin;
        let $name = unsafe { $crate::StackGuard::new(&$context, stack_pin) };
        let $name = $name.as_deref();
    };
}

#[macro_export]
macro_rules! get {
    (let $name:ident: $context:ident) => {
        $crate::try_get!(let $name: $context);
        let $name = $name.expect("Tried to get from an empty context");
    };
}

#[macro_export]
macro_rules! push {
    (let $name:ident: $context:ident = $value:expr) => {
        let stack_pin = unsafe { $crate::StackPin::new() };
        let stack_pin = &stack_pin;
        let $name = unsafe { $crate::Item::new(&$context, stack_pin, $value) };
        let $name = &*$name;
    };
}

use std::{
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    mem::MaybeUninit,
    num::NonZeroUsize,
    ptr::NonNull,
    thread::LocalKey,
};

pub trait ContextExt: Sized {
    type Item;
    fn len(self) -> usize;
    fn is_empty(self) -> bool { self.len() == 0 }
    fn push(self, value: Self::Item);
}

impl<T> ContextExt for &Context<T> {
    type Item = T;

    fn len(self) -> usize { self.len() }

    fn push(self, value: Self::Item) { self.push(value); }
}

impl<T> ContextExt for &'static LocalKey<Context<T>> {
    type Item = T;

    fn len(self) -> usize { self.with(|x| x.len()) }

    fn push(self, value: Self::Item) { self.with(|x| x.push(value)); }
}

pub struct Context<T> {
    blocks: UnsafeCell<Vec<*mut T>>,
    block_capacity: NonZeroUsize,
    len: Cell<usize>,
}

#[doc(hidden)]
pub struct StackPin(());

#[doc(hidden)]
pub struct StackGuard<'a, T> {
    value: NonNull<T>,
    stack_pin: PhantomData<&'a mut &'a StackPin>,
}

#[doc(hidden)]
pub struct Item<'ctx, 'a, T> {
    value: NonNull<T>,
    ctx: &'ctx Context<T>,
    stack_pin: PhantomData<&'a mut &'a StackPin>,
}

thread_local! {
    static CONTEXT: Context<i32> = Context::new(16);
}

impl<T> Drop for Context<T> {
    fn drop(&mut self) {
        struct DropContext<'a, I: Iterator<Item = (*mut T, (usize, usize))>, T> {
            blocks: &'a mut I,
        }

        impl<I: Iterator<Item = (*mut T, (usize, usize))>, T> Drop for DropContext<'_, I, T> {
            fn drop(&mut self) {
                self.blocks.by_ref().for_each(move |(ptr, (capacity, len))| unsafe {
                    Vec::from_raw_parts(ptr, len, capacity);
                })
            }
        }

        let len = self.len.get();
        let capacity = self.block_capacity.get();

        let init_blocks = len / capacity;
        let len = len % capacity;

        let blocks = unsafe { &*self.blocks.get() };

        let mut init_len = len;
        let block_sizes = (0..init_blocks)
            .map(|_| (capacity, capacity))
            .chain((init_blocks..blocks.len()).map(|_| (capacity, core::mem::take(&mut init_len))));
        let mut blocks = blocks.iter().copied().zip(block_sizes);

        let on_panic = DropContext { blocks: &mut blocks };

        drop(DropContext {
            blocks: on_panic.blocks,
        });

        core::mem::forget(on_panic);
    }
}

impl StackPin {
    pub unsafe fn new() -> Self { Self(()) }
}

impl<'a, T> StackGuard<'a, T> {
    pub unsafe fn from_ref(context: &Context<T>, _: &'a StackPin) -> Option<Self> {
        Some(Self {
            value: context.top()?,
            stack_pin: PhantomData,
        })
    }

    #[doc(hidden)]
    pub unsafe fn new(context: &'static LocalKey<Context<T>>, pin: &'a StackPin) -> Option<Self> {
        context.with(move |ctx| Self::from_ref(ctx, pin))
    }
}

impl<'ctx, 'a, T> Item<'ctx, 'a, T> {
    pub unsafe fn from_ref(ctx: &'ctx Context<T>, _: &'a StackPin, value: T) -> Self {
        Self {
            value: ctx.push(value),
            ctx,
            stack_pin: PhantomData,
        }
    }

    #[doc(hidden)]
    pub unsafe fn new(context: &'static LocalKey<Context<T>>, pin: &'a StackPin, value: T) -> Self {
        context.with(move |ctx| Self::from_ref(&*(ctx as *const Context<T>), pin, value))
    }

    pub fn guard(&self) -> StackGuard<'_, T> {
        StackGuard {
            value: self.value,
            stack_pin: PhantomData,
        }
    }
}

impl<T> core::ops::Deref for StackGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target { unsafe { self.value.as_ref() } }
}

impl<T> core::ops::Deref for Item<'_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target { unsafe { self.value.as_ref() } }
}

impl<T> Drop for Item<'_, '_, T> {
    fn drop(&mut self) {
        unsafe {
            #[cfg(miri)]
            self.value
                .as_ptr()
                .cast::<MaybeUninit<T>>()
                .write(MaybeUninit::uninit());
            self.ctx.pop();
        }
    }
}

impl<T> Context<T> {
    pub fn new(block_capacity: usize) -> Self {
        Self {
            blocks: Default::default(),
            block_capacity: NonZeroUsize::new(block_capacity).expect("The block capacity must be non-zero"),
            len: Cell::new(0),
        }
    }

    pub fn len(&self) -> usize {
        let blocks = unsafe { &*self.blocks.get() };
        blocks.len()
    }

    #[cold]
    #[inline(never)]
    fn reserve_block(&self) {
        let block_capacity = self.block_capacity.get();
        let mut block = Vec::<MaybeUninit<T>>::with_capacity(block_capacity);
        unsafe {
            block.set_len(block_capacity);
        }
        let blocks = unsafe { &mut *self.blocks.get() };
        blocks.push(Box::into_raw(block.into_boxed_slice()).cast::<T>());
    }

    pub fn push(&self, value: T) -> NonNull<T> {
        let block_capacity = self.block_capacity.get();
        let len = self.len.get();
        let block = len / block_capacity;
        let slot = len % block_capacity;

        if slot == 0 && block >= self.len() {
            self.reserve_block();
        }

        unsafe {
            self.len.set(len + 1);
            let blocks = &*self.blocks.get();
            let slot = blocks.get_unchecked(block).add(slot);
            slot.write(value);
            NonNull::new_unchecked(slot)
        }
    }

    pub fn top(&self) -> Option<NonNull<T>> {
        let block_capacity = self.block_capacity.get();
        let len = self.len.get().checked_sub(1)?;
        let block = len / block_capacity;
        let slot = len % block_capacity;

        unsafe {
            let blocks = &*self.blocks.get();
            let slot = blocks.get_unchecked(block).add(slot);
            Some(NonNull::new_unchecked(slot))
        }
    }

    pub unsafe fn pop(&self) { self.len.set(self.len.get().wrapping_sub(1)); }
}

#[test]
fn push_pop() {
    thread_local! {
        static CONTEXT: Context<i32> = Context::new(16);
    }

    push!(let item: CONTEXT = 10);
    get!(let guard: CONTEXT);
    assert_eq!(*item, 10);
    assert_eq!(*guard, 10);
}

#[test]
fn push_lots() {
    let ctx = Context::new(16);

    for _ in 0..100 {
        ctx.push(Box::new(10));
    }
}
