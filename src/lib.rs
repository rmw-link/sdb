mod dbpage;
pub use dbpage::{DbPage, EncodeDecode};
mod tx;
pub use tx::{MutTxnEnv, Tx, TxnEnv};
mod iter;
use iter::{key_iter, KeyIter};

pub use sanakirja::btree::page::Page;
use sanakirja::btree::{create_db_, BTreeMutPage, BTreePage, Db_, Iter, RevIter};
pub use sanakirja::{btree, Commit, Error, Storable, UnsizedStorable};
use sanakirja::{Env, LoadPage, RootDb};
use std::convert::Into;
use std::fs::create_dir_all;
use std::marker::PhantomData;
use std::mem;
use std::mem::ManuallyDrop;
use std::path::PathBuf;
use std::result::Result;

#[macro_export]
macro_rules! sdb {
  ($cls:ident) => {
    sanakirja::direct_repr!($cls);
    impl sdb::EncodeDecode<$cls> for $cls {}
  };
}

pub struct TxDb<
  K: Storable + PartialEq + ?Sized,
  V: Storable + PartialEq + ?Sized,
  T: LoadPage,
  P: BTreeMutPage<K, V> + BTreePage<K, V>,
> {
  db: Db_<K, V, P>,
  id: usize,
  tx: *mut T,
}

type UP<K, V> = btree::page_unsized::Page<K, V>;

pub type Db<'a, K, V> = DbPage<'a, K, V, Page<K, V>, K, V>;
pub type DbU<'a, K, V> = DbPage<'a, K, V, UP<K, V>, K, V>;

pub struct WriteTx<'a>(ManuallyDrop<MutTxnEnv<'a>>);
pub struct ReadTx<'a>(TxnEnv<'a>);

macro_rules! tx {
  ($cls:ident, $tx:tt) => {
    impl<'a> $cls<'a> {
      pub fn db<
        K: ?Sized + Storable + PartialEq,
        V: ?Sized + Storable + PartialEq,
        P: BTreeMutPage<K, V> + BTreePage<K, V>,
        RK: ?Sized + EncodeDecode<K>,
        RV: ?Sized + EncodeDecode<V>,
      >(
        &self,
        db: &DbPage<K, V, P, RK, RV>,
      ) -> TxDb<K, V, $tx<'a>, P> {
        TxDb {
          id: db.id,
          db: self.btree(db.id),
          tx: self.ptr() as *mut $tx,
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
    match tx.root_db::<K, V, P>(id) {
      None => {
        let mut w = Env::mut_txn_begin(tx.env_borrow()).unwrap();
        let tree = create_db_::<_, K, V, P>(&mut w).unwrap();
        w.set_root(id, tree.db);
        w.commit().unwrap();
        tree
      }
      Some(tree) => tree,
    }
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

macro_rules! iter {
  ($cls:ident, $fn:ident, $real:expr) => {
    pub fn $fn<OptionK: Into<Option<&'a K>>, OptionV: Into<Option<&'a V>>>(
      &self,
      key: OptionK,
      value: OptionV,
    ) -> Result<$cls<'a, T, K, V, P>, <T as LoadPage>::Error> {
      let tx = unsafe { &*self.tx };
      $real(
        tx,
        &self.db,
        match key.into() {
          None => None,
          Some(k) => match value.into() {
            None => Some((k, None)),
            Some(v) => Some((k, Some(v))),
          },
        },
      )
    }
  };
}

// all TxDb
impl<
    'a,
    K: 'a + PartialEq + Storable + ?Sized,
    V: 'a + PartialEq + Storable + ?Sized,
    T: 'a + LoadPage,
    P: BTreeMutPage<K, V> + BTreePage<K, V>,
  > TxDb<K, V, T, P>
{
  iter!(Iter, iter, btree::iter);
  iter!(RevIter, riter, btree::rev_iter);

  pub fn key_iter(&self, key: &'a K) -> Result<KeyIter<'a, T, K, V, P>, T::Error> {
    let tx = unsafe { &*self.tx };
    key_iter(tx, &self.db, key)
  }

  pub fn exist(&self, k: &K, v: &V) -> Result<bool, <T as LoadPage>::Error> {
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
  }

  pub fn one(&self, k: &K) -> Result<Option<&'a V>, <T as LoadPage>::Error> {
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
    K: 'a + Storable + PartialEq + ?Sized,
    V: 'a + Storable + PartialEq + ?Sized,
    P: BTreeMutPage<K, V> + BTreePage<K, V>,
  > TxDb<K, V, MutTxnEnv<'a>, P>
{
  pub fn put(&mut self, k: &K, v: &V) -> std::result::Result<bool, Error> {
    set_root!(btree::put(tx, &mut self.db, k, v), self, tx)
  }

  pub fn rm1<IntoV: Into<Option<&'a V>>>(&mut self, k: &K, v: IntoV) -> Result<bool, Error> {
    set_root!(btree::del(tx, &mut self.db, k, v.into()), self, tx)
  }

  pub fn rm(&mut self, k: &K) -> Result<usize, Error> {
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
    RK: ?Sized + EncodeDecode<K>,
    RV: ?Sized + EncodeDecode<V>,
  >(
    &self,
    id: usize,
  ) -> DbPage<K, V, P, RK, RV> {
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
