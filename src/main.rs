use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

async fn do_thing_1() -> i32 {
    2
}

async fn do_thing_2() -> i32 {
    10
}

async fn do_stuff() -> i32 {
    let x = do_thing_1().await;
    println!("x = {}", x);

    let y = do_thing_2().await;
    println!("y = {}", y);

    x * y
}

enum DoStuffFutureState {
    Start,
    CallingThing1(Pin<Box<dyn Future<Output = i32>>>),
    AfterThing1(i32),
    CallingThing2(i32, Pin<Box<dyn Future<Output = i32>>>),
    AfterThing2(i32, i32),
    ReturnStuff(i32),
}

struct DoStuffFuture {
    state: DoStuffFutureState,
}

impl DoStuffFuture {
    fn new() -> Self {
        Self {
            state: DoStuffFutureState::Start,
        }
    }
}

impl Future for DoStuffFuture {
    type Output = i32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.state {
            DoStuffFutureState::Start => {
                println!("DoStuffFutureState::Start");
                self.state = DoStuffFutureState::CallingThing1(Box::pin(do_thing_1()));
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            DoStuffFutureState::CallingThing1(ref mut thing1_future) => {
                println!("DoStuffFutureState::CallingThing1");
                match thing1_future.as_mut().poll(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(x) => {
                        self.state = DoStuffFutureState::AfterThing1(x);
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            DoStuffFutureState::AfterThing1(x) => {
                println!("DoStuffFutureState::AfterThing1");
                println!("x = {}", x);
                self.state = DoStuffFutureState::CallingThing2(*x, Box::pin(do_thing_2()));
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            DoStuffFutureState::CallingThing2(x, thing2_future) => {
                println!("DoStuffFutureState::CallingThing2");
                match thing2_future.as_mut().poll(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(y) => {
                        self.state = DoStuffFutureState::AfterThing2(*x, y);
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            DoStuffFutureState::AfterThing2(x, y) => {
                println!("DoStuffFutureState::AfterThing2");
                println!("y = {}", y);
                self.state = DoStuffFutureState::ReturnStuff(*x * *y);
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            DoStuffFutureState::ReturnStuff(result) => {
                println!("DoStuffFutureState::ReturnStuff");
                Poll::Ready(*result)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let sugared = do_stuff().await;
    println!("Sugared = {}", sugared);

    let desugared = DoStuffFuture::new().await;
    println!("Desugared = {}", desugared);
}
