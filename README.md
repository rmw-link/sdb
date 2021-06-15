<!-- 本文件由 ./readme.make.md 自动生成，请不要直接修改此文件 -->

# sdb

a user-friendly wrapper for [sanakirja](https://docs.rs/sanakirja/latest/sanakirja/) database

## about

sanakirja author is [Pierre-Étienne Meunier](https://twitter.com/pijul_org) .

I wrapper it for easy use .

First global define db , see [tests/db.rs](./tests/db.rs)

```rust
use sdb::{direct_repr, Db, Storable, Tx, UnsizedStorable};
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
pub struct Hash([u8; 32]);

direct_repr!(Hash);

#[dynamic]
pub static DB1: Db<'static, u64, Hash> = TX.db(1);

```

Second use it , see [tests/main.rs](./tests/main.rs)

```rust
mod db;
use anyhow::Result;
use db::{DB0, TX};

#[test]
fn main() -> Result<()> {
  {
    println!("# write transaction");
    let tx = TX.w()?;
    let mut db0 = tx.db(&DB0);

    db0.put(&1, &5)?;
    db0.put(&1, &3)?;

    db0.put(&2, &2)?;
    db0.put(&2, &3)?;
    db0.put(&2, &1)?;

    /*
    db0.put(&2, &5)?;
    db0.put(&3, &5)?;
    db0.put(&3, &5)?;
    */

    println!("## print all key");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    //write tx will auto commit when drop
  }

  {
    println!("# read transaction");

    let tx = TX.r()?; //
    let db0 = tx.db(&DB0);

    println!("# exist key 2 value 3 > {:?}", db0.exist(&2, &1)?);
    println!("# get key 1 > {:?}", db0.get(&1)?);
    println!("# get key 2 > {:?}", db0.get(&2)?);
    println!("# get key 3 > {:?}", db0.get(&3)?);

    println!("# print all key");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }
    println!("# print key greater or equal 2");
    for entry in db0.iter(&2, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("# print key greater or equal 2 and value greater or equal 1");
    for entry in db0.iter(&2, &1)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("# print greatest key");
    for entry in db0.riter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v);
      break;
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






