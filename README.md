<!-- 本文件由 ./readme.make.md 自动生成，请不要直接修改此文件 -->

# sdb

a user-friendly wrapper for [sanakirja](https://docs.rs/sanakirja/latest/sanakirja/) database

## about

sanakirja author is [Pierre-Étienne Meunier](https://twitter.com/pijul_org) .


see [Sanakirja 1.0 (pure Rust transactional on-disk key-value store) released!](https://www.reddit.com/r/rust/comments/lp5jez/sanakirja_10_pure_rust_transactional_ondisk/)

> Sanakirja is at least 10 times faster than Sled in my (sequential) benchmarks, and even 20%-50% faster than LMDB (the fastest C equivalent) in the same benchmarks. Also, I started it when there was no real alternative (Sled didn't exist at the time).
>
> I wrote about the cool features of Sled in another comment in this thread. Features unique to Sanakirja are fast zero-copy clone of tables, arbitrary nesting of datastructures (my main use cases uses tables of tuples of tables of statically-typed values, something like Db<String, (Db<A, B>, Db<C, D>, u64>)> in Rust terms). In addition to that, like LMDB (but unlike Sled), Sanakirja is robust to inter-process concurrency (multiple process reading and writing concurrently).



[I](https://github.com/rmw-link/sdb) wrapper it for easy use .

First step : static define db , see [tests/db.rs](./tests/db.rs)

```rust
use sdb::{direct_repr, Db, DbU, Storable, Tx, UnsizedStorable};
use static_init::dynamic;
use std::env;
use std::path::Path;

#[dynamic]
pub static DIR: String = env::current_exe()
  .unwrap()
  .parent()
  .unwrap()
  .parent()
  .unwrap()
  .display()
  .to_string();

#[dynamic]
pub static TX: Tx = {
  let dir = Path::new(&*DIR).join("db");

  println!("DATABASE DIR {}", dir.display().to_string());

  //use sdb::SdbArgs::{InitSize, MaxTx, Filename};

  Tx::new(
    &dir,
    &[
      //MaxTx(3),
      //Filename("sdb"),
      //InitSize(1<<21),
    ],
  )
};

#[dynamic]
pub static DB0: Db<'static, u64, u64> = TX.db(0);

#[derive(Default, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Hash(pub [u8; 2]);

direct_repr!(Hash);

#[dynamic]
pub static DB1: Db<'static, u64, Hash> = TX.db(1);

#[dynamic] // DbU = Db & btree::page_unsized::Page
pub static DB2: DbU<'static, u64, [u8]> = TX.db(2);

#[dynamic]
pub static DB3: DbU<'static, [u8], [u8]> = TX.db(3);

```

Second step : use it , see [tests/main.rs](./tests/main.rs)

```rust
mod db;
use anyhow::Result;
use db::{Hash, DB0, DB1, DB2, DB3, TX};

#[test]
fn main() -> Result<()> {
  println!("direct put");
  DB0.put(&0, &0)?;

  {
    println!("# write transaction");
    let tx = TX.w()?;
    let mut db0 = tx.db(&DB0);

    db0.put(&1, &5)?;
    db0.put(&1, &3)?;
    db0.put(&2, &2)?;
    db0.put(&2, &1)?;
    db0.put(&3, &7)?;
    db0.put(&3, &9)?;

    println!("- print all key db0");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- print db1 where key is 2");
    for entry in db0.key_iter(&2)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- delete key 2 the first value : {}", db0.rm1(&2, None)?);
    println!("- delete 2,5 : {}", db0.rm1(&2, &5)?);
    println!("- delete key 5 the first value : {}", db0.rm1(&5, None)?);
    println!("- delete key 3 all value : delete number = {}", db0.rm(&3)?);

    println!("- print all key");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    let mut db1 = tx.db(&DB1);
    db1.put(&1, &Hash([1, 2]))?;
    println!("- print all key db1");
    for entry in db1.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    let mut db2 = tx.db(&DB2);
    db2.put(&1, &[1, 2, 3][..])?;
    db2.put(&2, &[4, 6][..])?;

    println!("- print all key db2");
    for entry in db2.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    let mut db3 = tx.db(&DB3);
    db3.put(&[1, 2][..], &[1, 2, 3][..])?;

    println!("- print all key db3");
    for entry in db3.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    //write tx will auto commit when drop
  }
  {
    println!("# read transaction");

    let tx = TX.r()?; //
    let db0 = tx.db(&DB0);

    println!("- exist key 2 value 1 > {:?}", db0.exist(&2, &1)?);
    for i in [1, 2, 5] {
      println!("- get key {} > {:?}", i, db0.one(&i)?);
    }

    println!("- print all key");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }
    println!("- print key greater or equal 2");
    for entry in db0.iter(&2, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- print key greater or equal 2 and value greater or equal 1");
    for entry in db0.iter(&2, &1)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- print key greater or equal 2 and value greater or equal 1");
    for entry in db0.iter(&2, &1)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- print key in revese order");
    for entry in db0.riter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v);
    }
  }

  Ok(())
}

```

## sanakirja

Copy-on-write datastructures, storable on disk (or elsewhere) with a stable format.

* [Rethinking Sanakirja, a Rust database engine with fast clones (first released in 2015)](https://www.reddit.com/r/rust/comments/ld6k2h/rethinking_sanakirja_a_rust_database_engine_with/)
* [Implementing cross-process Sanakirja locks](https://pijul.org/posts/2021-01-15-sanakirja-locks/)
* [Sanakirja gets its full concurrency model](https://pijul.org/posts/2019-03-20-sanakirja-0.10/)
* [News of Sanakirja](https://pijul.org/2017/03/16/sanakirja/)
* [Sanakirja 0.2](https://pijul.org/2016/03/24/sanakirja-0.2/)
* [Functional semantics in Sanakirja](https://pijul.org/2016/03/03/functional-database/)






