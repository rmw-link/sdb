use anyhow::Result;
use sanakirja::btree::page::Page;
use sanakirja::{btree, Commit, Env, MutTxn, RootDb, Storable, Txn};
use std::fs::create_dir_all;
use std::path::PathBuf;

pub type MutTx<'a> = MutTxn<&'a Env, ()>;

pub struct W<'a, K: Storable, V: Storable> {
  pub tree: btree::Db<K, V>,
  pub tx: MutTx<'a>,
}

impl<'a, K: Storable, V: Storable> W<'a, K, V> {
  pub fn put(&mut self, k: &K, v: &V) -> Result<bool> {
    Ok(btree::put(&mut self.tx, &mut self.tree, k, v)?)
  }

  pub fn commit(self) -> Result<()> {
    Ok(self.tx.commit()?)
  }

  #[inline]
  pub fn tree_ref(&self) -> &btree::Db<K, V> {
    &self.tree
  }
}

pub type Tx<'a> = Txn<&'a Env>;

pub struct R<'a, K: Storable, V: Storable> {
  pub tree: &'a btree::Db<K, V>,
  pub tx: Tx<'a>,
}

impl<'a, K: Storable, V: Storable> R<'a, K, V> {
  #[inline]
  pub fn tree_ref(&self) -> &btree::Db<K, V> {
    self.tree
  }
}

#[macro_export]
macro_rules! W {
  ($db:ident, $fn:expr) => {
    $db.w(|$db| {
      $fn;
      Ok(())
    })?
  };
}

macro_rules! impl_R {
  ($cls:ident, $txn:ident) => {
    impl<'a, K: Storable, V: Storable> $cls<'a, K, V> {
      pub fn iter(
        &'a self,
        start: Option<(&K, Option<&V>)>,
      ) -> Result<btree::Iter<'a, $txn, K, V, Page<K, V>>> {
        Ok(btree::iter(&self.tx, self.tree_ref(), start)?)
      }
    }
  };
}

impl_R!(R, Tx);
impl_R!(W, MutTx);

pub enum SdbArgs<'a> {
  Dir(&'a PathBuf),
  Filename(&'a str),
  InitSize(u64),
  MaxTx(usize),
}

pub struct Sdb(pub(crate) Env);

impl Sdb {
  pub fn db<K: Storable, V: Storable>(&self, id: usize) -> Db<K, V> {
    let tx = Env::txn_begin(&self.0).unwrap();
    let tree = match tx.root_db(id) {
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
      tree: tree,
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

pub struct Db<'a, K: Storable, V: Storable> {
  pub sdb: &'a Sdb,
  pub id: usize,
  pub tree: btree::Db<K, V>,
}

impl<'a, K: Storable, V: Storable> Db<'a, K, V> {
  pub fn writer(&self) -> Result<W<K, V>> {
    let tx = Env::mut_txn_begin(&self.sdb.0)?;
    let tree: btree::Db<K, V> = tx.root_db(self.id).unwrap();
    Ok(W { tree, tx })
  }
  pub fn reader(&self) -> Result<R<K, V>> {
    let tx = Env::txn_begin(&self.sdb.0)?;
    Ok(R {
      tree: &self.tree,
      tx: tx,
    })
  }
  pub fn w<F: FnOnce(&mut W<K, V>) -> Result<()>>(&self, f: F) -> Result<()> {
    let mut w = self.writer()?;
    f(&mut w)?;
    w.commit()?;
    Ok(())
  }
}
