use lunatic::{spawn_link, Mailbox};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let child = spawn_link!(@task || {
        println!("hello, world!");
    });
    let _ = child.result();
}
