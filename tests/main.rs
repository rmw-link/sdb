use anyhow::Result;
use sanakirja::*;
use sdb::Sdb;
use std::env;

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

  let mut txn = Env::mut_txn_begin(sdb.env)?;
  let root_db = 0;
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

  /*
    let mut txn = Env::mut_txn_begin(sdb.env)?;
    let root_db = 1;
    let mut db = btree::create_db::<_, [u8; 4], u64>(&mut txn).unwrap();
    txn.set_root(root_db, db.db);
    btree::put(&mut txn, &mut db, &"1234", &0).unwrap();
    txn.commit().unwrap();
  */
  //  assert_eq!(4, 4);

  Ok(())
}
