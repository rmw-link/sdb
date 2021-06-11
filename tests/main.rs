use anyhow::Result;
use sanakirja::*;
use sdb::{Sdb, W};
use std::env;

#[derive(Default, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
struct Hash([u8; 32]);
direct_repr!(Hash);

#[test]
fn main() -> Result<()> {
  let dir = env::current_exe()?
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join("db");

  let sdb = {
    use sdb::SdbArgs::Dir;
    println!("DATABASE DIR {}", dir.display().to_string());
    Sdb::new(&[Dir(&dir)])
  };

  let db = sdb.db::<u64, u64>(0);

  println!("db.id {}", db.id);

  W!(db, db.put(&1, &0)?);
  W!(db, {
    db.put(&1, &1)?;
    db.put(&2, &1)?;
    db.put(&2, &2)?;
    db.put(&2, &3)?;

    println!("# print all key");
    for entry in db.iter(None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("# print key greater or equal 2");
    for entry in db.iter(Some((&2, None)))? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("# print key greater or equal 2 and value greater or equal 2");
    for entry in db.iter(Some((&2, Some(&2))))? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }
  });

  let mut w = db.w()?;
  w.put(&5, &1)?;
  println!("# print greatest key");
  for entry in w.riter(None)? {
    let (k, v) = entry?;
    println!("> {:?} {:?}", k, v);
    break;
  }
  w.commit()?;

  let mut w = db.w()?;
  w.put(&5, &13)?;

  println!("# print all key use readonly transaction");
  let r = db.r()?;
  for entry in r.iter(None)? {
    let (k, v) = entry?;
    println!("> {:?} {:?}", k, v)
  }

  Ok(())
}
