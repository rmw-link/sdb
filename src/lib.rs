use anyhow::Result;
use sanakirja::btree::{BTreePage, Db_, Iter, RevIter};
use sanakirja::{btree, AllocPage, Env, LoadPage, MutTxn, RootDb, Txn};
pub use sanakirja::{direct_repr, Commit, Storable, UnsizedStorable};
use std::convert::Into;
use std::fs::create_dir_all;
use std::marker::PhantomData;
use std::path::PathBuf;

pub enum SdbArgs<'a> {
  Dir(&'a PathBuf),
  Filename(&'a str),
  InitSize(u64),
  MaxTx(usize),
}

pub struct Sdb(pub(crate) Env);

type MutTx<'a> = MutTxn<&'a Env, ()>;

type Tx<'a> = Txn<&'a Env>;

pub struct Db<'a, K: Storable, V: Storable> {
  pub sdb: &'a Sdb,
  pub id: usize,
  _kv: PhantomData<(K, V)>,
}

macro_rules! iter {
  ($fn:ident, $real:ident, $cls:ident) => {
    fn $fn<
      'a,
      K: 'a + Storable,
      V: 'a + Storable,
      P: BTreePage<K, V>,
      OptionK: Into<Option<&'a K>>,
      OptionV: Into<Option<&'a V>>,
    >(
      &self,
      tree: &Db_<K, V, P>,
      key: OptionK,
      value: OptionV,
    ) -> Result<$cls<Self, K, V, P>, <Self as LoadPage>::Error> {
      btree::$real(
        self,
        tree,
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

pub trait R: Sized + LoadPage + RootDb {
  iter!(iter, iter, Iter);
  iter!(riter, rev_iter, RevIter);

  fn tree<K: Storable, V: Storable>(&mut self, db: &Db<K, V>) -> btree::Db<K, V> {
    self.root_db(db.id).unwrap()
  }

  fn exist<'a, K: 'a + PartialEq + Storable, V: 'a + PartialEq + Storable, P: BTreePage<K, V>>(
    &self,
    tree: &Db_<K, V, P>,
    k: &K,
    v: &V,
  ) -> Result<bool, <Self as LoadPage>::Error> {
    match btree::get(self, tree, k, v.into())? {
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

  fn get<'a, K: 'a + PartialEq + Storable, V: Storable, P: BTreePage<K, V>>(
    &'a self,
    tree: &Db_<K, V, P>,
    k: &K,
  ) -> Result<Option<&V>, <Self as LoadPage>::Error> {
    match btree::get(self, tree, k, None)? {
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

pub trait W: AllocPage + Sized + RootDb + Commit {
  //fn put<K: Storable, V: Storable>(&mut self, db: &mut Db<K, V>, k: &K, v: &V) -> Result<bool> {

  fn put<K: Storable, V: Storable>(
    &mut self,
    tree: &mut btree::Db<K, V>,
    k: &K,
    v: &V,
  ) -> std::result::Result<bool, <Self as LoadPage>::Error> {
    btree::put(self, tree, k, v)
  }
}

impl<'a> W for MutTx<'a> {}
impl<'a> R for MutTx<'a> {}
impl<'a> R for Tx<'a> {}

impl Sdb {
  pub fn w(&self) -> Result<MutTx> {
    Ok(Env::mut_txn_begin(&self.0)?)
  }

  pub fn r(&self) -> Result<Tx> {
    Ok(Env::txn_begin(&self.0)?)
  }

  pub fn db<K: Storable, V: Storable>(&self, id: usize) -> Db<K, V> {
    let tx = self.r().unwrap();
    let _ = match tx.root_db(id) {
      Some(tree) => tree,
      None => {
        let mut w = Env::mut_txn_begin(&self.0).unwrap();
        let tree = btree::create_db::<_, K, V>(&mut w).unwrap();
        w.set_root(id, tree.db);
        w.commit().unwrap();
        tree
      }
    };
    Db {
      sdb: &self,
      id: id,
      _kv: PhantomData,
    }
  }

  pub fn new(args: &[SdbArgs]) -> Sdb {
    let mut dir = None;
    let mut filename = None;
    let mut init_size = None;
    let mut max_tx = None;
    use SdbArgs::*;

    for arg in args {
      match arg {
        Dir(i) => dir = (*i).into(),
        Filename(i) => filename = i.to_string().into(),
        InitSize(i) => init_size = (*i).into(),
        MaxTx(i) => max_tx = (*i).into(),
      }
    }

    let dir = dir.unwrap();
    let filename = filename.unwrap_or_else(|| "sdb".into());
    let init_size = init_size.unwrap_or(1 << 21);
    let max_tx = max_tx.unwrap_or(3);

    create_dir_all(&dir).unwrap();
    Sdb(Env::new(dir.join(filename), init_size, max_tx).unwrap())
  }
}
/*

pub struct W<'a, K: Storable, V: Storable> {
  pub tree: btree::Db<K, V>,
  pub tx: Rc<RefCell<MutTx<'a>>>,
}

impl<'a, K: Storable, V: Storable> W<'a, K, V> {
  pub fn put(&mut self, k: &K, v: &V) -> Result<bool> {
    Ok(btree::put(
      &mut *self.tx.borrow_mut(),
      &mut self.tree,
      k,
      v,
    )?)
  }

  #[inline]
  pub fn tree_ref(&self) -> &btree::Db<K, V> {
    &self.tree
  }

  #[inline]
  pub fn tx_ref(&self) -> &MutTx<'a> {
    &self.tx.borrow()
  }
}


pub struct R<'a, K: Storable, V: Storable> {
  pub tree: &'a btree::Db<K, V>,
  pub tx: &'a Tx<'a>,
}

impl<'a, K: Storable, V: Storable> R<'a, K, V> {
  #[inline]
  pub fn tree_ref(&self) -> &btree::Db<K, V> {
    self.tree
  }

  #[inline]
  pub fn tx_ref(&self) -> &Tx<'a> {
    self.tx
  }
}
/*
#[macro_export]
macro_rules! W {
  ($db:ident, $fn:expr) => {
    $db.write(|$db| {
      $fn;
      Ok(())
    })?
  };
}
*/

macro_rules! impl_R {
  ($cls:ident, $txn:ident) => {
    impl<'a, K: Storable, V: Storable> $cls<'a, K, V> {
      pub fn riter(
        &'a self,
        start: Option<(&K, Option<&V>)>,
      ) -> Result<btree::RevIter<'a, $txn, K, V, Page<K, V>>> {
        Ok(btree::rev_iter(self.tx_ref(), self.tree_ref(), start)?)
      }
      pub fn iter(
        &'a self,
        start: Option<(&K, Option<&V>)>,
      ) -> Result<btree::Iter<'a, $txn, K, V, Page<K, V>>> {
        Ok(btree::iter(self.tx_ref(), self.tree_ref(), start)?)
      }
    }
  };
}

impl_R!(R, Tx);
impl_R!(W, MutTx);



impl<'a, K: Storable, V: Storable> Db<'a, K, V> {
  pub fn w(self, tx: Rc<RefCell<MutTx<'a>>>) -> W<K, V> {
    let tree: btree::Db<K, V> = tx.borrow().root_db(self.id).unwrap();
    W { tree, tx }
  }
  pub fn r(&self, tx: &'a Tx<'a>) -> R<K, V> {
    R {
      tree: &self.tree,
      tx: tx,
    }
  }
  /*
  pub fn write<F: FnOnce(&mut W<K, V>) -> Result<()>>(&self, f: F) -> Result<()> {
    let mut w = self.w()?;
    f(&mut w)?;
    w.commit()?;
    Ok(())
  }
  */
}
*/
