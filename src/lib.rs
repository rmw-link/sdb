use anyhow::Result;
use sanakirja::{btree, Commit, Env, MutTxn, RootDb, Storable, Txn};
use std::fs::create_dir_all;
use std::path::PathBuf;

pub enum SdbArgs<'a> {
  Dir(&'a PathBuf),
  Filename(&'a str),
  InitSize(u64),
  MaxTx(usize),
}

pub struct Sdb(pub(crate) Env);

pub struct Db<'a, K: Storable, V: Storable> {
  pub sdb: &'a Sdb,
  pub id: usize,
  pub tree: btree::Db<K, V>,
}

pub struct W<'a, K: Storable, V: Storable> {
  pub tree: btree::Db<K, V>,
  pub tx: MutTxn<&'a Env, ()>,
}

impl<'a, K: Storable, V: Storable> W<'a, K, V> {
  pub fn put(&mut self, k: &K, v: &V) -> Result<bool> {
    Ok(btree::put(&mut self.tx, &mut self.tree, k, v)?)
  }
}

pub struct R<'a, K: Storable, V: Storable> {
  pub tree: &'a btree::Db<K, V>,
  pub tx: Txn<&'a Env>,
}

impl<'a, K: Storable, V: Storable> W<'a, K, V> {
  pub fn commit(self) -> Result<()> {
    Ok(self.tx.commit()?)
  }
}

impl<'a, K: Storable, V: Storable> Db<'a, K, V> {
  pub fn w(&self) -> Result<W<K, V>> {
    let tx = Env::mut_txn_begin(&self.sdb.0)?;
    let tree: btree::Db<K, V> = tx.root_db(self.id).unwrap();
    Ok(W { tree, tx })
  }
  pub fn r(&self) -> Result<R<K, V>> {
    let tx = Env::txn_begin(&self.sdb.0)?;
    Ok(R {
      tree: &self.tree,
      tx: tx,
    })
  }
}

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
