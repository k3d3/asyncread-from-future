This is a demonstration showing how to implement the `AsyncRead` trait (from Tokio, but the futures version should work similarly) from an existing async-defined `Future`.

Because of lifetimes and the fact that `AsyncRead`'s `poll_read()` method may be called multiple times, this is a bit tricky to get right.
Not only do we need to make sure the source `Future` lives across multiple `Poll::Pending` results, it also means we need to create new `Future`s
after each time `poll_read()` returns `Poll::Ready`.

As this is just meant to be demonstration code, I'm release this to the public domain, or under the CC0 licence.
