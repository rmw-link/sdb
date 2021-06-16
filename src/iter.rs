use sanakirja::btree::{BTreePage, Cursor, Db_};
use sanakirja::{LoadPage, Storable};

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

pub fn key_iter<'a, T, K, V, P>(
  txn: &'a T,
  db: &Db_<K, V, P>,
  key: &'a K,
) -> Result<KeyIter<'a, T, K, V, P>, T::Error>
where
  T: LoadPage,
  K: PartialEq + Storable + ?Sized,
  V: Storable + ?Sized,
  P: BTreePage<K, V>,
{
  let mut cursor = Cursor::new(txn, db)?;

  cursor.set(txn, key, None)?;
  Ok(KeyIter { cursor, txn, key })
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
