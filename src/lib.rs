use anyhow::Result;
pub use sanakirja::btree::page::Page;
use sanakirja::btree::{BTreeMutPage, BTreePage, Iter, RevIter};
pub use sanakirja::{btree, direct_repr, Commit, Error, SetRoot, Storable, UnsizedStorable};
use sanakirja::{AllocPage, Env, LoadPage, MutTxn, RootDb, Txn};
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

pub struct Db_<'a, K: Storable, V: Storable, P: BTreeMutPage<K, V> + BTreePage<K, V>> {
  tx: &'a Tx,
  pub id: usize,
  _kvp: PhantomData<(K, V, P)>,
}

pub type Db<'a, K, V> = Db_<'a, K, V, Page<K, V>>;
pub type DbU<'a, K, V> = Db_<'a, K, V, btree::page_unsized::Page<K, V>>;

pub struct TxDb<
  K: Storable + PartialEq,
  V: PartialEq + Storable,
  T: Sized + LoadPage,
  P: BTreeMutPage<K, V> + BTreePage<K, V>,
> {
  db: btree::Db_<K, V, P>,
  id: usize,
  tx: *mut T,
}

pub struct WriteTx<'a>(ManuallyDrop<MutTxnEnv<'a>>);
pub struct ReadTx<'a>(TxnEnv<'a>);

macro_rules! tx {
  ($cls:ident, $tx:tt) => {
    impl<'a> $cls<'a> {
      pub fn db<
        K: Storable + PartialEq,
        V: Storable + PartialEq,
        P: BTreeMutPage<K, V> + BTreePage<K, V>,
      >(
        &self,
        db: &Db_<K, V, P>,
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
  pub fn btree<K: Storable, V: Storable, P: BTreeMutPage<K, V> + BTreePage<K, V>>(
    &self,
    id: usize,
  ) -> btree::Db_<K, V, P> {
    let tx = &self.0;
    match tx.root_db::<K, V, P>(id) {
      None => {
        let mut w = Env::mut_txn_begin(tx.env_borrow()).unwrap();
        let tree = btree::create_db_::<_, K, V, P>(&mut w).unwrap();
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

  pub fn btree<K: Storable, V: Storable, P: BTreeMutPage<K, V> + BTreePage<K, V>>(
    &self,
    id: usize,
  ) -> btree::Db_<K, V, P> {
    let tx = self.ptr() as *mut MutTxnEnv<'a>;
    let tx = unsafe { &mut *tx };

    match tx.root_db::<K, V, P>(id) {
      None => {
        let tree = btree::create_db_::<_, K, V, P>(tx).unwrap();
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
    println!("auto commited");
  }
}

macro_rules! iter {
  ($fn:ident, $real:ident, $cls:ident) => {
    pub fn $fn<OptionK: Into<Option<&'a K>>, OptionV: Into<Option<&'a V>>>(
      &self,
      key: OptionK,
      value: OptionV,
    ) -> Result<$cls<T, K, V, P>, <T as LoadPage>::Error> {
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
impl<
    'a,
    K: 'a + PartialEq + Storable,
    V: 'a + PartialEq + Storable,
    T: 'a + Sized + LoadPage,
    P: BTreeMutPage<K, V> + BTreePage<K, V>,
  > TxDb<K, V, T, P>
{
  iter!(iter, iter, Iter);
  iter!(riter, rev_iter, RevIter);

  pub fn exist<IntoK: Into<&'a K>, IntoV: Into<&'a V>>(
    &self,
    k: IntoK,
    v: IntoV,
  ) -> Result<bool, <T as LoadPage>::Error> {
    let tx = unsafe { &*self.tx };
    let k = k.into();
    let v = v.into();
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

  pub fn get<IntoK: Into<&'a K>>(&self, k: IntoK) -> Result<Option<&'a V>, <T as LoadPage>::Error> {
    let tx = unsafe { &*self.tx };
    let k = k.into();
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

// write tx TxDb
impl<
    'a,
    K: 'a + Storable + PartialEq,
    V: 'a + PartialEq + Storable,
    T: Sized + AllocPage + core::fmt::Debug + SetRoot,
    P: BTreeMutPage<K, V> + BTreePage<K, V>,
  > TxDb<K, V, T, P>
{
  pub fn put<IntoK: Into<&'a K>, IntoV: Into<&'a V>>(
    &mut self,
    k: IntoK,
    v: IntoV,
  ) -> std::result::Result<bool, <T as LoadPage>::Error> {
    let mut tx = unsafe { &mut *self.tx };
    let r = btree::put(tx, &mut self.db, k.into(), v.into());
    tx.set_root(self.id, self.db.db);
    r
  }

  pub fn del<IntoK: Into<&'a K>, IntoV: Into<Option<&'a V>>>(
    &mut self,
    k: IntoK,
    v: IntoV,
  ) -> Result<bool, <T as LoadPage>::Error> {
    let tx = unsafe { &mut *self.tx };
    let r = btree::del(tx, &mut self.db, k.into(), v.into());
    tx.set_root(self.id, self.db.db);
    r
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

  pub fn db<K: Storable, V: Storable, P: BTreeMutPage<K, V> + BTreePage<K, V>>(
    &self,
    id: usize,
  ) -> Db_<K, V, P> {
    Db_ {
      tx: &self,
      id: id,
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
    Tx {
      env: Env::new(dir.join(filename), init_size, max_tx).unwrap(),
    }
  }
}
