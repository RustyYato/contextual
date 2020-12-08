use contextual::{get, push, Context, ContextExt as _};

thread_local! {
    static CTX: Context<u32> = Context::new(16);
}

fn main() {
    CTX.push(0);
    // println!("{}", main);
}

fn fib() {
    // println!("{}", main);
}
