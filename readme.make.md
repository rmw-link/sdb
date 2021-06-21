# sdb

a user-friendly wrapper for [sanakirja](https://docs.rs/sanakirja/latest/sanakirja/) database

sanakirja a key can have multiple values , this is different from the other k-v databases .

## about

sanakirja author is [Pierre-Étienne Meunier](https://twitter.com/pijul_org) .

see [Sanakirja 1.0 (pure Rust transactional on-disk key-value store) released!](https://www.reddit.com/r/rust/comments/lp5jez/sanakirja_10_pure_rust_transactional_ondisk/)

> Sanakirja is at least 10 times faster than Sled in my (sequential) benchmarks, and even 20%-50% faster than LMDB (the fastest C equivalent) in the same benchmarks. Also, I started it when there was no real alternative (Sled didn't exist at the time).

[I](https://github.com/rmw-link/sdb) wrapper it for easy use .

First step : static define db , see [tests/db.rs](./tests/db.rs)

I use `static_init = {git="https://gitlab.com/vkahl/static_init.git"}` for static init ( use git version because of [bug](https://gitlab.com/okannen/static_init/-/issues/7) ) .  You can use [lazy_static](https://docs.rs/crate/lazy_static) instead .

```rust
#include ./sdb/tests/db.rs
```

Second step : use it , see [tests/main.rs](./tests/main.rs)

```rust
#include ./sdb/tests/main.rs
```

db method you can see [src/dbpage.rs](./src/dbpage.rs)

```rust
#include ./sdb/src/dbpage.rs
```


## sanakirja

Copy-on-write datastructures, storable on disk (or elsewhere) with a stable format.

* [Rethinking Sanakirja, a Rust database engine with fast clones (first released in 2015)](https://www.reddit.com/r/rust/comments/ld6k2h/rethinking_sanakirja_a_rust_database_engine_with/)
* [Implementing cross-process Sanakirja locks](https://pijul.org/posts/2021-01-15-sanakirja-locks/)
* [Sanakirja gets its full concurrency model](https://pijul.org/posts/2019-03-20-sanakirja-0.10/)
* [News of Sanakirja](https://pijul.org/2017/03/16/sanakirja/)
* [Sanakirja 0.2](https://pijul.org/2016/03/24/sanakirja-0.2/)
* [Functional semantics in Sanakirja](https://pijul.org/2016/03/03/functional-database/)






