# sdb

a user-friendly wrapper for [sanakirja](https://docs.rs/sanakirja/latest/sanakirja/) database

## about

sanakirja author is [Pierre-Étienne Meunier](https://twitter.com/pijul_org) .


see [Sanakirja 1.0 (pure Rust transactional on-disk key-value store) released!](https://www.reddit.com/r/rust/comments/lp5jez/sanakirja_10_pure_rust_transactional_ondisk/)

> Sanakirja is at least 10 times faster than Sled in my (sequential) benchmarks, and even 20%-50% faster than LMDB (the fastest C equivalent) in the same benchmarks. Also, I started it when there was no real alternative (Sled didn't exist at the time).

> I wrote about the cool features of Sled in another comment in this thread. Features unique to Sanakirja are fast zero-copy clone of tables, arbitrary nesting of datastructures (my main use cases uses tables of tuples of tables of statically-typed values, something like Db<String, (Db<A, B>, Db<C, D>, u64>)> in Rust terms). In addition to that, like LMDB (but unlike Sled), Sanakirja is robust to inter-process concurrency (multiple process reading and writing concurrently).



I wrapper it for easy use .

First step : static define db , see [tests/db.rs](./tests/db.rs)

```rust
#include ./tests/db.rs
```

Second step : use it , see [tests/main.rs](./tests/main.rs)

```rust
#include ./tests/main.rs
```

## sanakirja

Copy-on-write datastructures, storable on disk (or elsewhere) with a stable format.

* [Rethinking Sanakirja, a Rust database engine with fast clones (first released in 2015)](https://www.reddit.com/r/rust/comments/ld6k2h/rethinking_sanakirja_a_rust_database_engine_with/)
* [Implementing cross-process Sanakirja locks](https://pijul.org/posts/2021-01-15-sanakirja-locks/)
* [Sanakirja gets its full concurrency model](https://pijul.org/posts/2019-03-20-sanakirja-0.10/)
* [News of Sanakirja](https://pijul.org/2017/03/16/sanakirja/)
* [Sanakirja 0.2](https://pijul.org/2016/03/24/sanakirja-0.2/)
* [Functional semantics in Sanakirja](https://pijul.org/2016/03/03/functional-database/)






