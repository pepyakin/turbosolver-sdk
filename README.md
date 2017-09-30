# TurboSolver SDK (Research)

> Please note, that this project works only on Android and only on x86_64. The build tested only on macOS.

Mobile SDKs are hard, especially in case of Android because JNI bindings are tedious to write. This project aiming to explore other possibilities.

At the moment there are 2 implementations:

- local HTTP server and
- [Cap'n Proto](https://capnproto.org/)
- JNR

The primary idea of the first two is to define thin layer for the "transport" and then do the actual communication with it, without sharding concrete objects between two. A big advantage of this kind of approaches is decreasing surface of an API. In case of JNI this could be a big win.

But before diving into the guts lets go through the problem.

## The Problem

This SDK provides a way to solve [Sudoku puzzles](https://en.wikipedia.org/wiki/Sudoku).
Here it's [Android API](https://github.com/pepyakin/turbosolver-sdk/blob/master/android-demo/app/src/main/java/me/pepyakin/turbosolver/TurboSolver.kt).

It is a bit silly: solver can be created with only one grid and then solved multiple times. This decision was made to make the problem a bit more difficult.

- [`solver`](https://github.com/pepyakin/turbosolver-sdk/blob/master/libsolver/src/solver.rs). Actually implemented by [`sudoku`](https://crates.io/crates/sudoku) crate
- [`context`](https://github.com/pepyakin/turbosolver-sdk/blob/master/libsolver/src/context.rs) - context of the app. Holds solvers and implements logic of working with solvers without direct access to them. Can be used to implement different concurrency schemes.
- [`executor`](https://github.com/pepyakin/turbosolver-sdk/blob/master/libsolver/src/executor.rs) implements a message-passing style API on top of the `context`.

## HTTP Server

Just start a new HTTP server and do requests to it encoded with JSON. That's it. HTTP is very mature and there are a lot of tools around, JSON is on par. Your library would be useful not only from iOS and Android, but probably anywhere where HTTP is supported. No schema needed, you can start right away.
The biggest drawback is the overhead. Also, for better or worse, HTTP server that bound on localhost can accept connection from the any app running on the device, and I don't found a way to prevent that. If it's not desired you probably can do somekind of authorization (in this project there is none). Also, it is generally for request-response workflows.

Local HTTP server is probably the simplest way to use your library from Android and iOS.

- [Android side](https://github.com/pepyakin/turbosolver-sdk/blob/master/android-demo/app/src/main/java/me/pepyakin/turbosolver/HttpTurboSolver.kt)
- [Rust side](https://github.com/pepyakin/turbosolver-sdk/blob/master/libsolver/src/http.rs)

## Cap'n Proto

The idea is to define [actor](https://en.wikipedia.org/wiki/Actor_model) like system and communicate with it only with messages without having any shared memory. This approach has benefits of reduced complexity and simpler threading model. With proper implementation it could be possible to avoid copying messages on the FFI boundaries.

Messages can be in any serialization format you like, it can be JSON, but preferrably something more efficient like [Flatbuffers](https://google.github.io/flatbuffers/flatbuffers_support.html). I chose [Cap' Proto](https://capnproto.org/) because I had always wanted to play with it.

- [Android side](https://github.com/pepyakin/turbosolver-sdk/blob/master/android-demo/app/src/main/java/me/pepyakin/turbosolver/capnp/CapnpTurboSolver.kt)
- [Rust side](https://github.com/pepyakin/turbosolver-sdk/blob/master/libsolver/src/capnproto.rs)
- [Schema definition](https://github.com/pepyakin/turbosolver-sdk/blob/master/common/api.capnp)

## JNR

Actually JNR is pretty impressive! It let's you to use C functions without requiring you to write JNI bindings. But you should be careful as you exposed to the raw memory.

Big downside of JNR is a lack of Android support. It took for me an entire day to figure this stuff out!

- You need an Android build of jffi. [This repo](https://github.com/pepyakin/jffi)  could be useful,
- Because jnr-ffi generates proxies at runtime, you need that proxies in Dex format (because Android's ART/Dalvik not into JVM bytecode). I chose to generate compile .class files into .dex at runtime with DX and then load it with DexClassLoader. Here is a [commit](https://github.com/pepyakin/jnr-ffi/commit/01ed59708adc19a825d2d4fe19065d1912cfbac8) implementing this approach.
- Some little hacks and workarounds that could be found [here](https://github.com/pepyakin/turbosolver-sdk/blob/39eae1762808de16f44f3213c092691624026074/android-demo/app/src/main/java/me/pepyakin/turbosolver/JnrTurboSolver.kt#L77-L106).

- [Android side](https://github.com/pepyakin/turbosolver-sdk/blob/master/android-demo/app/src/main/java/me/pepyakin/turbosolver/JnrTurboSolver.kt)
- [Rust side](https://github.com/pepyakin/turbosolver-sdk/blob/master/libsolver/src/ffi.rs)
