use contextual::*;

thread_local! {
    static CTX: Context<u32> = Context::new();
}

fn main() {}
