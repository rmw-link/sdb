use sanakirja::btree::{BTreePage, Cursor, Db_};
use sanakirja::{LoadPage, Storable};
use std::marker::PhantomData;

pub fn key_iter<'a, T, K, V, P>(
  txn: &'a T,
  db: &Db_<K, V, P>,
  key: &K,
) -> Result<Box<dyn Iterator<Item = Result<(&'a K, &'a V), T::Error>> + 'a>, T::Error>
where
  T: LoadPage,
  K: 'a + PartialEq + Storable + ?Sized,
  V: 'a + Storable + ?Sized,
  P: 'a + BTreePage<K, V>,
{
  let mut cursor = Cursor::new(txn, db)?;

  match cursor.set(txn, key, None)? {
    Some((key_c, _)) => Ok(Box::new(KeyIter {
      cursor,
      txn,
      key: key_c,
    })),
    None => Ok(Box::new(StopIter::<T, K, V>(PhantomData {}))),
  }
}

pub struct StopIter<'a, T: LoadPage, K: PartialEq + Storable + ?Sized, V: Storable + ?Sized>(
  PhantomData<(&'a T, &'a K, &'a V)>,
);

impl<'a, T: LoadPage, K: PartialEq + Storable + ?Sized + 'a, V: Storable + ?Sized + 'a> Iterator
  for StopIter<'a, T, K, V>
{
  type Item = Result<(&'a K, &'a V), T::Error>;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    None
  }
}

pub struct KeyIter<
  'a,
  T: LoadPage,
  K: PartialEq + Storable + ?Sized,
  V: Storable + ?Sized,
  P: BTreePage<K, V>,
> {
  txn: &'a T,
  cursor: Cursor<K, V, P>,
  key: &'a K,
}

impl<
    'a,
    T: LoadPage,
    K: PartialEq + Storable + ?Sized + 'a,
    V: Storable + ?Sized + 'a,
    P: BTreePage<K, V> + 'a,
  > Iterator for KeyIter<'a, T, K, V, P>
{
  type Item = Result<(&'a K, &'a V), T::Error>;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let entry = self.cursor.next(self.txn).transpose();
    match entry {
      Some(kv) => match kv {
        Ok((k, _)) => {
          if k == self.key {
            Some(kv)
          } else {
            None
          }
        }
        _ => Some(kv),
      },
      _ => entry,
    }
  }
}
