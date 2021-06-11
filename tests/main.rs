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
  /*
  for entry in w.iter(None)? {
    let (k, v) = entry?;
    println!("> {:?} {:?}", k, v)
  }
  */
  w.commit()?;

  let mut w = db.w()?;
  w.put(&5, &13)?;

  println!("# print all key use readonly transaction");
  let r = db.r()?;
  for entry in r.iter(None)? {
    let (k, v) = entry?;
    println!("> {:?} {:?}", k, v)
  }

  /*

  let mut db = btree::create_db::<_, u64, u64>(&mut txn).unwrap();
  txn.set_root(root_db, db.db);
  btree::put(&mut txn, &mut db, &1, &3).unwrap();
  btree::put(&mut txn, &mut db, &3, &9).unwrap();
  btree::put(&mut txn, &mut db, &4, &10).unwrap();
  btree::put(&mut txn, &mut db, &1, &6).unwrap();
  btree::put(&mut txn, &mut db, &1, &4).unwrap();
  btree::put(&mut txn, &mut db, &1, &1).unwrap();
  btree::put(&mut txn, &mut db, &1, &0).unwrap();
  txn.commit().unwrap();
    let mut txn = Env::mut_txn_begin(&sdb.0)?;
    let root_db = 1;
    let mut db = btree::create_db::<_, Hash, u64>(&mut txn).unwrap();
    txn.set_root(root_db, db.db);
    btree::put(&mut txn, &mut db, &Hash::default(), &3).unwrap();
    txn.commit().unwrap();

    let txn = Env::txn_begin(&sdb.0).unwrap();
    let db: btree::Db<Hash, u64> = txn.root_db(root_db).unwrap();
    for entry in btree::iter(&txn, &db, None).unwrap() {
      let (k, v) = entry.unwrap();
      println!("{:?} {:?}", k, v)
    }
  */
  //  assert_eq!(4, 4);

  Ok(())
}
