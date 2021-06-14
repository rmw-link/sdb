mod db;
use anyhow::Result;
use db::TX;
//use db::{DB0, TX};
use sdb::Db;

#[test]
fn main() -> Result<()> {
  let DB0: Db<u64, u64> = TX.db(0);
  let tx = TX.w()?;
  let mut t1 = tx.db(&DB0);

  t1.put(&1, &1)?;
  t1.put(&1, &3)?;
  t1.put(&1, &5)?;

  println!("# print all key");
  for entry in t1.iter(None, None)? {
    let (k, v) = entry?;
    println!("> {:?} {:?}", k, v)
  }

  //tx.commit()?;

  //let t2 = tx.db(&*T2)?;
  /*
  let test = tx.db(&*db::TEST);

  test.put(&mut tx, &1, &1)?;
  */
  /*
  test.put(&mut tx, &1, &2)?;
  tx.put(&mut test.db, &1, &1)?;
  tx.put(&mut test.db, &1, &1)?;
  tx.put(&mut test, &1, &2)?;
  tx.put(&mut test, &2, &0)?;
  tx.put(&mut test, &2, &1)?;
  tx.put(&mut test, &2, &2)?;

  println!("# get one key > {:?}", test.get(&1)?);
  println!("# get one key > {:?}", tx.get(&test, &1)?);
  */

  /*

    println!("# get one key > {:?}", tx.get(&test, &1)?);

    println!("# exist 1,2 > {:?}", tx.exist(&test, &1, &2)?);

    println!("# exist 1,3 > {:?}", tx.exist(&test, &1, &3)?);

    println!("# print all key");
    for entry in tx.iter(&test, None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("# print key greater or equal 2");
    for entry in tx.iter(&test, &2, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("# print key greater or equal 2 and value greater or equal 1");
    for entry in tx.iter(&test, &2, &1)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("# print key less or equal 2 and value less or equal 1");
    for entry in tx.riter(&test, &1, &2)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    tx.commit()?;
  */

  /*
     W!(
       put!(db,1,2)
     )
  */
  /*

  tx.put(db,1,2);

  let mut tx = sdb.w()?;
  w.put(&5, &1)?;
  tx.commit()?;
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
  */
  Ok(())
}
