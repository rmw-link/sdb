#![feature(decl_macro)]
mod dbpage;
pub use dbpage::{DbPage, Encode};
mod tx;
pub use tx::{MutTxnEnv, Tx, TxnEnv};
mod iter;
use iter::key_iter;

extern crate sdb_macro;
pub use sdb_macro::Sdb;

pub use sanakirja::btree::page::Page;
use sanakirja::btree::{create_db_, BTreeMutPage, BTreePage, Db_, Iter, RevIter};
pub use sanakirja::{btree, direct_repr, Commit, Error, Storable, UnsizedStorable};
use sanakirja::{Env, LoadPage, RootDb};
use std::convert::Into;
use std::fs::create_dir_all;
use std::marker::PhantomData;
use std::mem;
use std::mem::ManuallyDrop;
use std::path::PathBuf;
use std::result::Result;

#[macro_export]
pub macro repr($cls:ident) {
  use sdb::direct_repr;
  direct_repr!($cls);

  use sdb::encode;
  encode!($cls);
}

pub struct TxDb<
  'a,
  K: Storable + PartialEq + ?Sized,
  V: Storable + PartialEq + ?Sized,
  T: LoadPage,
  P: BTreeMutPage<K, V> + BTreePage<K, V>,
  RK: ?Sized + Encode<K>,
  RV: ?Sized + Encode<V>,
> {
  db: Db_<K, V, P>,
  id: usize,
  tx: *mut T,
  pub(crate) _rkv: PhantomData<(&'a RK, &'a RV)>,
}

type UP<K, V> = btree::page_unsized::Page<K, V>;

pub type Db<'a, K, V> = DbPage<'a, K, V, Page<K, V>, K, V>;
pub type DbU<'a, K, V> = DbPage<'a, K, V, UP<K, V>, K, V>;
pub type DbEk<'a, K, V, RK> = DbPage<'a, K, V, UP<K, V>, RK, V>;
pub type DbEv<'a, K, V, RV> = DbPage<'a, K, V, UP<K, V>, K, RV>;
pub type DbEkv<'a, K, V, RK, RV> = DbPage<'a, K, V, UP<K, V>, RK, RV>;

pub struct WriteTx<'a>(ManuallyDrop<MutTxnEnv<'a>>);
pub struct ReadTx<'a>(TxnEnv<'a>);

#[cfg(feature = "desse")]
#[macro_export]
macro_rules! desse {
  ($cls:ident, $desse:ident) => {
    #[derive(
      Default, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug, DesseSized, Desse,
    )]
    pub struct $desse([u8; $cls::SIZE]);

    sdb::direct_repr!($desse);

    impl Encode<$desse> for $cls {
      #[inline]
      fn encode<R: Sized>(&self, next: &mut dyn FnMut(&$desse) -> R) -> R {
        next(&$desse(self.serialize()))
      }
    }

    impl From<&$desse> for $cls {
      fn from(v: &$desse) -> Self {
        Data2::deserialize_from(&v.0)
      }
    }
  };
}

macro_rules! tx {
  ($cls:ident, $tx:tt) => {
    impl<'a> $cls<'a> {
      pub fn db<
        K: ?Sized + Storable + PartialEq,
        V: ?Sized + Storable + PartialEq,
        P: BTreeMutPage<K, V> + BTreePage<K, V>,
        RK: ?Sized + Encode<K>,
        RV: ?Sized + Encode<V>,
      >(
        &self,
        db: &DbPage<K, V, P, RK, RV>,
      ) -> TxDb<K, V, $tx<'a>, P, RK, RV> {
        TxDb {
          id: db.id,
          db: self.btree(db.id),
          tx: self.ptr() as *mut $tx,
          _rkv: PhantomData {},
        }
      }
    }
  };
}

tx!(WriteTx, MutTxnEnv);
tx!(ReadTx, TxnEnv);

impl<'a> ReadTx<'a> {
  pub fn ptr(&self) -> *const TxnEnv<'a> {
    &self.0
  }
  pub fn btree<
    K: Storable + ?Sized,
    V: Storable + ?Sized,
    P: BTreeMutPage<K, V> + BTreePage<K, V>,
  >(
    &self,
    id: usize,
  ) -> Db_<K, V, P> {
    let tx = &self.0;
    tx.root_db::<K, V, P>(id).unwrap()
  }
}

impl<'a> WriteTx<'a> {
  pub fn ptr(&self) -> *const MutTxnEnv<'a> {
    &*self.0
  }

  pub fn btree<
    K: Storable + ?Sized,
    V: Storable + ?Sized,
    P: BTreeMutPage<K, V> + BTreePage<K, V>,
  >(
    &self,
    id: usize,
  ) -> Db_<K, V, P> {
    let tx = self.ptr() as *mut MutTxnEnv<'a>;
    let tx = unsafe { &mut *tx };

    match tx.root_db::<K, V, P>(id) {
      None => {
        let tree = create_db_::<_, K, V, P>(tx).unwrap();
        tx.set_root(id, tree.db);
        tree
      }
      Some(tree) => tree,
    }
  }
}

impl<'a> Drop for WriteTx<'a> {
  fn drop(&mut self) {
    let tx = mem::MaybeUninit::<MutTxnEnv>::uninit();
    let tx = mem::replace(&mut *self.0, unsafe { tx.assume_init() });
    tx.commit().unwrap();
  }
}

macro_rules! encode {
  ($i:ident, $fn:expr) => {
    $i.encode(&mut |$i| $fn)
  };
}

macro_rules! encode_k_v {
  ($k:ident, $v:ident, $fn:expr) => {
    encode!($k, encode!($v, $fn))
  };
}

macro_rules! iter {
  ($cls:ident, $fn:ident, $real:expr) => {
    #[inline]
    pub fn $fn<OptionK: Into<Option<&'a RK>>, OptionV: Into<Option<&'a RV>>>(
      &self,
      k: OptionK,
      v: OptionV,
    ) -> Result<$cls<'a, T, K, V, P>, <T as LoadPage>::Error> {
      match k.into() {
        None => {
          let tx = unsafe { &*self.tx };
          $real(tx, &self.db, None)
        }
        Some(k) => match v.into() {
          Some(v) => {
            encode_k_v!(k, v, {
              let tx = unsafe { &*self.tx };
              $real(tx, &self.db, Some((k, Some(v))))
            })
          }
          None => {
            encode!(k, {
              let tx = unsafe { &*self.tx };
              $real(tx, &self.db, Some((k, None)))
            })
          }
        },
      }
    }
  };
}

// all TxDb
impl<
    'a,
    'b,
    K: 'a + PartialEq + Storable + ?Sized,
    V: 'a + PartialEq + Storable + ?Sized,
    T: 'a + LoadPage,
    P: 'a + BTreeMutPage<K, V> + BTreePage<K, V>,
    RK: 'a + ?Sized + Encode<K>,
    RV: 'a + ?Sized + Encode<V>,
  > TxDb<'b, K, V, T, P, RK, RV>
{
  iter!(Iter, iter, btree::iter);
  iter!(RevIter, riter, btree::rev_iter);

  #[inline]
  pub fn key_iter<'c>(
    &self,
    k: &'a RK,
  ) -> Result<Box<dyn Iterator<Item = Result<(&'a K, &'a V), T::Error>> + 'a>, T::Error> {
    let tx = unsafe { &*self.tx };
    encode!(k, key_iter(tx, &self.db, k))
  }

  #[inline]
  pub fn exist(&self, k: &RK, v: &RV) -> Result<bool, <T as LoadPage>::Error> {
    encode_k_v!(k, v, {
      let tx = unsafe { &*self.tx };
      match btree::get(tx, &self.db, k, Some(v))? {
        None => Ok(false),
        Some((key, val)) => {
          if key == k {
            if val == v {
              Ok(true)
            } else {
              Ok(false)
            }
          } else {
            Ok(false)
          }
        }
      }
    })
  }

  #[inline]
  pub fn one(&self, k: &RK) -> Result<Option<&'a V>, <T as LoadPage>::Error> {
    encode!(k, {
      let tx = unsafe { &*self.tx };
      match btree::get(tx, &self.db, k, None)? {
        None => Ok(None),
        Some((key, v)) => {
          if key == k {
            Ok(Some(v))
          } else {
            Ok(None)
          }
        }
      }
    })
  }
}

macro_rules! set_root {
  ($fn: expr, $self:ident, $tx:ident ) => {{
    let $tx = unsafe { &mut *$self.tx };
    let db = $self.db.db;
    let r = $fn;
    let db_now = $self.db.db;
    if db_now != db {
      $tx.set_root($self.id, db_now);
    }
    r
  }};
}

// write tx TxDb
impl<
    'a,
    'b,
    K: 'a + Storable + PartialEq + ?Sized,
    V: 'a + Storable + PartialEq + ?Sized,
    P: BTreeMutPage<K, V> + BTreePage<K, V>,
    RK: ?Sized + Encode<K>,
    RV: ?Sized + Encode<V>,
  > TxDb<'b, K, V, MutTxnEnv<'b>, P, RK, RV>
{
  #[inline]
  pub fn put(&mut self, k: &RK, v: &RV) -> std::result::Result<bool, Error> {
    encode_k_v!(
      k,
      v,
      set_root!(btree::put(tx, &mut self.db, k, v), self, tx)
    )
  }

  #[inline]
  pub fn rm1<IntoV: Into<Option<&'b RV>>>(&mut self, k: &RK, v: IntoV) -> Result<bool, Error> {
    match v.into() {
      Some(v) => {
        encode_k_v!(
          k,
          v,
          set_root!(btree::del(tx, &mut self.db, k, Some(v)), self, tx)
        )
      }
      None => {
        encode!(
          k,
          set_root!(btree::del(tx, &mut self.db, k, None), self, tx)
        )
      }
    }
  }

  #[inline]
  pub fn upsert(&mut self, k: &RK, v: &RV) -> std::result::Result<bool, Error> {
    self.rm(k)?;
    self.put(k, v)
  }

  #[inline]
  pub fn rm(&mut self, k: &RK) -> Result<usize, Error> {
    encode!(
      k,
      set_root!(
        {
          let db = &mut self.db;
          let mut n = 0usize;
          while btree::del(tx, db, k, None)? {
            n += 1
          }
          Ok(n)
        },
        self,
        tx
      )
    )
  }
}

pub enum TxArgs<'a> {
  Filename(&'a str),
  InitSize(u64),
  MaxTx(usize),
}

impl Tx {
  pub fn w(&self) -> Result<WriteTx, Error> {
    Ok(WriteTx(ManuallyDrop::new(Env::mut_txn_begin(&self.env)?)))
  }

  pub fn r(&self) -> Result<ReadTx, Error> {
    Ok(ReadTx(Env::txn_begin(&self.env)?))
  }

  pub fn db<
    K: ?Sized + Storable + PartialEq,
    V: ?Sized + Storable + PartialEq,
    P: BTreeMutPage<K, V> + BTreePage<K, V>,
    RK: ?Sized + Encode<K>,
    RV: ?Sized + Encode<V>,
  >(
    &self,
    id: usize,
  ) -> DbPage<K, V, P, RK, RV> {
    if let None = {
      let tx = Env::txn_begin(&self.env).unwrap();
      tx.root_db::<K, V, P>(id)
    } {
      let mut w = Env::mut_txn_begin(&self.env).unwrap();
      let tree = create_db_::<_, K, V, P>(&mut w).unwrap();
      w.set_root(id, tree.db);
      w.commit().unwrap();
    }

    DbPage {
      tx: self,
      id,
      _kvp: PhantomData,
    }
  }

  pub fn new<P: Into<PathBuf>>(dir: P, args: &[TxArgs]) -> Tx {
    let mut filename = None;
    let mut init_size = None;
    let mut max_tx = None;
    use TxArgs::*;

    for arg in args {
      match arg {
        Filename(i) => filename = i.to_string().into(),
        InitSize(i) => init_size = (*i).into(),
        MaxTx(i) => max_tx = (*i).into(),
      }
    }

    let dir: PathBuf = dir.into();
    let filename = filename.unwrap_or_else(|| "sdb".into());
    let init_size = init_size.unwrap_or(1 << 21);
    let max_tx = max_tx.unwrap_or(3);

    create_dir_all(&dir).unwrap();

    let file = dir.join(filename);

    Tx {
      env: Env::new(&file, init_size, max_tx).unwrap(),
    }
  }
}
