use anyhow::Result;
use sanakirja::btree::page::Page;
use sanakirja::btree::{Iter, RevIter};
use sanakirja::{btree, AllocPage, Env, LoadPage, MutTxn, RootDb, Txn};
pub use sanakirja::{direct_repr, Commit, Error, Storable, UnsizedStorable};
use std::convert::Into;
use std::fs::create_dir_all;
use std::marker::PhantomData;
use std::mem;
use std::mem::ManuallyDrop;
use std::path::PathBuf;

type MutTxnEnv<'a> = MutTxn<&'a Env, ()>;
type TxnEnv<'a> = Txn<&'a Env>;

pub struct Tx {
  pub(crate) env: Env,
}

pub struct Db<'a, K: Storable, V: Storable> {
  tx: &'a Tx,
  pub id: usize,
  _kv: PhantomData<(K, V)>,
}

pub struct TxDb<K: Storable, V: Storable, T: Sized + LoadPage> {
  db: btree::Db<K, V>,
  tx: *mut T,
}

pub struct WriteTx<'a>(ManuallyDrop<MutTxnEnv<'a>>);
pub struct ReadTx<'a>(TxnEnv<'a>);

macro_rules! tx {
  ($cls:ident, $tx:tt) => {
    impl<'a> $cls<'a> {
      pub fn db<'b, K: 'b + Storable, V: 'b + Storable>(
        &self,
        db: &Db<'b, K, V>,
      ) -> TxDb<K, V, $tx<'a>> {
        TxDb {
          db: self.btree(db.id),
          tx: self.ptr() as *mut $tx<'a>,
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
  pub fn btree<K: Storable, V: Storable>(&self, id: usize) -> btree::Db<K, V> {
    let tx = &self.0;
    match tx.root_db::<K, V, Page<K, V>>(id) {
      None => {
        let mut w = Env::mut_txn_begin(tx.env_borrow()).unwrap();
        let tree = btree::create_db::<_, K, V>(&mut w).unwrap();
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

  pub fn btree<K: Storable, V: Storable>(&self, id: usize) -> btree::Db<K, V> {
    let tx = self.ptr() as *mut MutTxnEnv<'a>;
    let tx = unsafe { &mut *tx };

    match tx.root_db::<K, V, Page<K, V>>(id) {
      None => {
        let tree = btree::create_db::<_, K, V>(tx).unwrap();
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
  ($fn:ident, $real:ident, $cls:ident) => {
    pub fn $fn<OptionK: Into<Option<&'a K>>, OptionV: Into<Option<&'a V>>>(
      &self,
      key: OptionK,
      value: OptionV,
    ) -> Result<$cls<T, K, V, Page<K, V>>, <T as LoadPage>::Error> {
      let tx = unsafe { &*self.tx };
      btree::$real(
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
impl<'a, K: 'a + Storable, V: 'a + Storable, T: Sized + LoadPage> TxDb<K, V, T> {
  iter!(iter, iter, Iter);
  iter!(riter, rev_iter, RevIter);
}

// write tx TxDb
impl<'a, K: Storable, V: Storable, T: Sized + AllocPage> TxDb<K, V, T> {
  pub fn put(&mut self, k: &K, v: &V) -> std::result::Result<bool, <T as LoadPage>::Error> {
    let tx = unsafe { &mut *self.tx };
    btree::put(tx, &mut self.db, k, v)
  }
}

pub enum TxArgs<'a> {
  Filename(&'a str),
  InitSize(u64),
  MaxTx(usize),
}

impl Tx {
  pub fn w(&self) -> Result<WriteTx> {
    Ok(WriteTx(ManuallyDrop::new(Env::mut_txn_begin(&self.env)?)))
  }

  pub fn r(&self) -> Result<ReadTx> {
    Ok(ReadTx(Env::txn_begin(&self.env)?))
  }

  pub fn db<K: Storable, V: Storable>(&self, id: usize) -> Db<K, V> {
    Db {
      tx: &self,
      id: id,
      _kv: PhantomData,
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
    Tx {
      env: Env::new(dir.join(filename), init_size, max_tx).unwrap(),
    }
  }
}

/*
use sanakirja::btree::{BTreePage, Db_, Iter, RevIter};
use sanakirja::{btree, AllocPage, Env, LoadPage, WriteTxn, RootDb, ReadTxn};
pub use sanakirja::{direct_repr, Commit, Storable, UnsizedStorable};

//use static_init::lazy::LesserLazy;





/*
impl<K: 'static + Storable, V: 'static + Storable> From<LesserLazy<Db<'static, K, V>>>
  for Db<'static, K, V>
{
  fn move from(w: LesserLazy<Db<'static, K, V>>) -> Db<'static, K, V> {
    *w
  }
}
*/


pub struct ReadTxDb<'a, K: Storable, V: Storable, TX: R> {
  tx: &'a TX,
  pub db: btree::Db<K, V>,
}

impl<'a, K: Storable, V: Storable, TX: R> From<&'a ReadTxDb<'a, K, V, TX>> for &'a btree::Db<K, V> {
  fn from(w: &'a ReadTxDb<K, V, TX>) -> &'a btree::Db<K, V> {
    &w.db
  }
}

impl<'a, K: Storable + PartialEq, V: Storable, TX: R> ReadTxDb<'a, K, V, TX> {
  pub fn get(&self, k: &K) -> Result<Option<&V>, <TX as LoadPage>::Error> {
    self.tx.get(&self.db, k)
  }

  pub fn put<T: AllocPage + Sized + Commit>(
    &mut self,
    tx: &mut T,
    k: &K,
    v: &V,
  ) -> std::result::Result<bool, <T as LoadPage>::Error> {
    btree::put(tx, &mut self.db, k, v)
  }
}

pub trait R: Sized + LoadPage + RootDb {
  iter!(iter, iter, Iter);
  iter!(riter, rev_iter, RevIter);

  fn db<'a, K: Storable, V: Storable>(&'a self, db: &'a Db<K, V>) -> ReadTxDb<'a, K, V, Self> {
    let tx = self;
    let db = tx.root_db(db.id).unwrap();
    ReadTxDb { db, tx }
  }

  fn exist<'a, K: 'a + PartialEq + Storable, V: 'a + PartialEq + Storable, P: BTreePage<K, V>>(
    &self,
    db: &Db_<K, V, P>,
    k: &K,
    v: &V,
  ) -> Result<bool, <Self as LoadPage>::Error> {
    match btree::get(self, db, k, v.into())? {
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

  fn get<
    'a,
    K: 'a + PartialEq + Storable,
    V: Storable,
    P: 'a + BTreePage<K, V>,
    DB: Into<&'a Db_<K, V, P>>,
  >(
    &'a self,
    db: DB,
    k: &K,
  ) -> Result<Option<&V>, <Self as LoadPage>::Error> {
    match btree::get(self, db.into(), k, None)? {
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


*/
