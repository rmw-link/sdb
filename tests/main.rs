mod db;
use anyhow::Result;
use db::{DB0, TX};

#[test]
fn main() -> Result<()> {
  {
    println!("# write transaction");
    let tx = TX.w()?;
    let mut db0 = tx.db(&DB0);

    db0.put(&1, &5)?;
    db0.put(&1, &3)?;

    db0.put(&2, &2)?;
    db0.put(&2, &1)?;
    db0.put(&3, &9)?;

    println!("- print all key");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- delete key 2 the first : {}", db0.del(&2, None)?);
    println!("- delete 2,5 : {}", db0.del(&2, &5)?);
    println!("- delete key 5 the first : {}", db0.del(&5, None)?);

    println!("- print all key");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }
    //write tx will auto commit when drop
  }
  {
    println!("# read transaction");

    let tx = TX.r()?; //
    let db0 = tx.db(&DB0);

    println!("- exist key 2 value 1 > {:?}", db0.exist(&2, &1)?);
    for i in [1, 2, 5] {
      println!("- get key {} > {:?}", i, db0.one(&i)?);
    }

    println!("- print all key");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }
    println!("- print key greater or equal 2");
    for entry in db0.iter(&2, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- print key greater or equal 2 and value greater or equal 1");
    for entry in db0.iter(&2, &1)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- print key greater or equal 2 and value greater or equal 1");
    for entry in db0.iter(&2, &1)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- print greatest key");
    for entry in db0.riter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v);
      break;
    }
  }

  Ok(())
}
