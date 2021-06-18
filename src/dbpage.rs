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
  > DbPage<'a, K, V, P>
{
  pub fn put<IntoK: Into<&'a K>, IntoV: Into<&'a V>>(
    &self,
    k: IntoK,
    v: IntoV,
  ) -> Result<bool, Error> {
    db_page_w!(self, db, db.put(k.into(), v.into()))
  }

  pub fn rm<IntoK: Into<&'a K>>(&self, k: IntoK) -> Result<usize, Error> {
    db_page_w!(self, db, db.rm(k.into()))
  }
  pub fn one<IntoK: Into<&'a K>>(
    &self,
    k: IntoK,
  ) -> Result<Option<&'a V>, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.one(k.into()))
  }

  pub fn exist<IntoK: Into<&'a K>, IntoV: Into<&'a V>>(
    &self,
    k: IntoK,
    v: IntoV,
  ) -> Result<bool, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.exist(k.into(), v.into()))
  }

  pub fn rm1<IntoK: Into<&'a K>, IntoV: Into<Option<&'a V>>>(
    &self,
    k: IntoK,
    v: IntoV,
  ) -> Result<bool, Error> {
    db_page_w!(self, db, db.rm1(k.into(), v.into()))
  }

  pub fn key_iter<IntoK: Into<&'a K>>(
    &self,
    k: IntoK,
  ) -> Result<KeyIter<TxnEnv, K, V, P>, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.key_iter(k.into()))
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
> {
  pub(crate) tx: &'a Tx,
  pub id: usize,
  pub(crate) _kvp: PhantomData<(&'a K, &'a V, &'a P)>,
}
