use anyhow::Result;
use sanakirja::*;
use sdb::Sdb;
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
    use sdb::SdbArgs::*;
    println!("DATABASE DIR {}", dir.display().to_string());
    Sdb::new(&[Dir(&dir)])
  };

  let db = sdb.db::<u64, u64>(0);
  let mut w = db.w()?;
  println!("db.id {}", db.id);
  btree::put(&mut w.tx, &mut w.tree, &5, &11)?;
  btree::put(&mut w.tx, &mut w.tree, &5, &12)?;
  w.commit()?;
  let mut w = db.w()?;
  btree::put(&mut w.tx, &mut w.tree, &5, &13)?;

  let r = db.r()?;
  for entry in btree::iter(&r.tx, r.tree, None)? {
    let (k, v) = entry.unwrap();
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
