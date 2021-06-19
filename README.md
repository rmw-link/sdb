<!-- 本文件由 ./readme.make.md 自动生成，请不要直接修改此文件 -->

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
use sdb::{encode_decode, sdb, Db, DbU, EncodeDecode, Storable, Tx, UnsizedStorable};
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

sdb!(Hash);

#[dynamic]
pub static DB1: Db<'static, u64, Hash> = TX.db(1);

#[dynamic] // DbU = Db & btree::page_unsized::Page
pub static DB2: DbU<'static, u64, [u8]> = TX.db(2);

#[dynamic]
pub static DB3: DbU<'static, [u8], [u8]> = TX.db(3);

#[derive(Default, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Data {
  pub hash: [u8; 3],
  pub id: u64,
}

sdb!(Data);

#[dynamic]
pub static DB4: Db<'static, u64, Data> = TX.db(4);

```

Second step : use it , see [tests/main.rs](./tests/main.rs)

```rust
mod db;
use anyhow::Result;
use db::{Data, Hash, DB0, DB1, DB2, DB3, DB4, TX};
use sdb::UnsizedStorable;

#[test]
fn main() -> Result<()> {
  println!("direct put");
  DB0.put(&0, &0)?;
  DB0.put(&0, &1)?;
  DB0.put(&6, &1)?;
  DB0.put(&6, &2)?;
  DB0.put(&6, &3)?;
  println!("- print val where key = 6");
  for entry in DB0.key_iter(&6)? {
    let (k, v) = entry?;
    println!("> {:?} {:?}", k, v)
  }
  DB0.rm1(&6, None)?;

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
    println!("- delete key=2 and value=5 : {}", db0.rm1(&2, &5)?);
    println!("- delete key 3 all value : delete number = {}", db0.rm(&3)?);
    println!("- delete key 5 the first value : {}", db0.rm1(&5, None)?);

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

    let mut db4 = tx.db(&DB4);
    let data = Data {
      id: 1234,
      hash: [3, 2, 1],
    };
    println!("data size : {}", data.size());
    db4.put(&1, &data)?;
    println!("- print all key db4");
    for entry in db4.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    //write tx will auto commit when drop
  }
  {
    println!("# read transaction");

    let tx = TX.r()?; //
    let db0 = tx.db(&DB0);

    dbg!(db0.exist(&2, &1)?);
    for i in [1, 2, 5] {
      dbg!(i, db0.one(&i)?);
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

    let db4 = tx.db(&DB4);
    println!("- db4 : print key in revese order");
    for entry in db4.riter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v);
    }
  }

  Ok(())
}

```

db method you can see [src/dbpage.rs](./src/dbpage.rs)

```rust
use crate::iter::KeyIter;
use crate::tx::{Tx, TxnEnv};
pub use sanakirja::btree::page::Page;
use sanakirja::btree::{BTreeMutPage, BTreePage, Iter, RevIter};
use sanakirja::{Error, LoadPage, Storable};
use std::marker::PhantomData;

macro_rules! db_page_r {
  ($self:ident, $db:ident, $fn:expr) => {{
    let tx = $self.tx.r()?;
    let $db = tx.db($self);
    $fn
  }};
}

macro_rules! db_page_w {
  ($self:ident, $db:ident, $fn:expr) => {{
    let tx = $self.tx.w()?;
    let mut $db = tx.db($self);
    $fn
  }};
}

impl<
    'a,
    K: ?Sized + Storable + PartialEq,
    V: ?Sized + Storable + PartialEq,
    P: BTreeMutPage<K, V> + BTreePage<K, V>,
    RK: ?Sized + EncodeDecode<K>,
    RV: ?Sized + EncodeDecode<V>,
  > DbPage<'a, K, V, P, RK, RV>
{
  pub fn put(&self, k: &K, v: &V) -> Result<bool, Error> {
    db_page_w!(self, db, db.put(k, v))
  }

  pub fn rm(&self, k: &K) -> Result<usize, Error> {
    db_page_w!(self, db, db.rm(k))
  }

  pub fn one(&self, k: &'a K) -> Result<Option<&'a V>, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.one(k))
  }

  pub fn exist(&self, k: &K, v: &V) -> Result<bool, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.exist(k, v))
  }

  pub fn rm1<IntoV: Into<Option<&'a V>>>(&self, k: &K, v: IntoV) -> Result<bool, Error> {
    db_page_w!(self, db, db.rm1(k, v.into()))
  }

  pub fn key_iter(
    &self,
    k: &'a K,
  ) -> Result<KeyIter<TxnEnv, K, V, P>, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.key_iter(k))
  }

  pub fn iter<OptionK: Into<Option<&'a K>>, OptionV: Into<Option<&'a V>>>(
    &self,
    k: OptionK,
    v: OptionV,
  ) -> Result<Iter<TxnEnv, K, V, P>, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.iter(k.into(), v.into()))
  }

  pub fn riter<OptionK: Into<Option<&'a K>>, OptionV: Into<Option<&'a V>>>(
    &self,
    k: OptionK,
    v: OptionV,
  ) -> Result<RevIter<TxnEnv, K, V, P>, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.riter(k.into(), v.into()))
  }
}

pub struct DbPage<
  'a,
  K: ?Sized + Storable + PartialEq,
  V: ?Sized + Storable + PartialEq,
  P: BTreeMutPage<K, V> + BTreePage<K, V>,
  RK: ?Sized + EncodeDecode<K>,
  RV: ?Sized + EncodeDecode<V>,
> {
  pub(crate) tx: &'a Tx,
  pub id: usize,
  pub(crate) _kvp: PhantomData<(&'a K, &'a V, &'a P, &'a RK, &'a RV)>,
}
pub trait EncodeDecode<T: ?Sized> {
  fn encode<R: Sized>(&self, next: &dyn Fn(&T) -> R) -> R;
  /*
  fn decode(val: &T) -> Self;
  */
}

#[macro_export]
macro_rules! encode_decode {
  ($cls:ty, $t:ty) => {
    impl EncodeDecode<$t> for $cls {
      #[inline]
      fn encode<R: Sized>(&self, next: &dyn Fn(&$t) -> R) -> R {
        next(self)
      }
    }
  };
  ($cls:ty) => {
    encode_decode!($cls, $cls);
  };
}
encode_decode!([u8]);
encode_decode!(u64);

```


## sanakirja

Copy-on-write datastructures, storable on disk (or elsewhere) with a stable format.

* [Rethinking Sanakirja, a Rust database engine with fast clones (first released in 2015)](https://www.reddit.com/r/rust/comments/ld6k2h/rethinking_sanakirja_a_rust_database_engine_with/)
* [Implementing cross-process Sanakirja locks](https://pijul.org/posts/2021-01-15-sanakirja-locks/)
* [Sanakirja gets its full concurrency model](https://pijul.org/posts/2019-03-20-sanakirja-0.10/)
* [News of Sanakirja](https://pijul.org/2017/03/16/sanakirja/)
* [Sanakirja 0.2](https://pijul.org/2016/03/24/sanakirja-0.2/)
* [Functional semantics in Sanakirja](https://pijul.org/2016/03/03/functional-database/)






