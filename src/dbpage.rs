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
  pub fn put(&self, k: &RK, v: &RV) -> Result<bool, Error> {
    db_page_w!(self, db, db.put(k, v))
  }

  pub fn rm(&self, k: &RK) -> Result<usize, Error> {
    db_page_w!(self, db, db.rm(k))
  }

  pub fn one(&self, k: &'a RK) -> Result<Option<&'a V>, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.one(k))
  }

  pub fn exist(&self, k: &RK, v: &RV) -> Result<bool, <TxnEnv as LoadPage>::Error> {
    db_page_r!(self, db, db.exist(k, v))
  }

  pub fn rm1<IntoV: Into<Option<&'a RV>>>(&self, k: &RK, v: IntoV) -> Result<bool, Error> {
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
  fn encode<R: Sized>(&self, next: &mut dyn FnMut(&T) -> R) -> R;
  // fn decode(val: &T) -> &Self;
}

#[macro_export]
macro_rules! encode_decode {
  ($cls:ty, $t:ty) => {
    impl EncodeDecode<$t> for $cls {
      #[inline]
      fn encode<R: Sized>(&self, next: &mut dyn FnMut(&$t) -> R) -> R {
        next(self)
      }
      /*
          #[inline]
          fn decode(val: &$t) -> &$cls {
            val
          }
      */
    }
  };
  ($cls:ty) => {
    encode_decode!($cls, $cls);
  };
}

macro_rules! encode_decode_li {
  ( $( $x:ty ),* ) => {
    $(encode_decode!($x);)*
  };
}

encode_decode_li!(
  [u8],
  bool,
  i8,
  u8,
  i16,
  u16,
  i32,
  u32,
  i64,
  u64,
  i128,
  u128,
  isize,
  usize,
  f32,
  f64
);
