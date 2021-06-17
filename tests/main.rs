mod db;
use anyhow::Result;
use db::{Data, Hash, DB0, DB1, DB2, DB3, DB4, TX};

#[test]
fn main() -> Result<()> {
  println!("direct put");
  DB0.put(&0, &0)?;
  DB0.put(&0, &1)?;
  DB0.put(&6, &1)?;
  DB0.put(&6, &2)?;
  DB0.put(&6, &3)?;
  println!("- print val where key = 6");
  for entry in DB0.key_iter(&6)? {
    let (k, v) = entry?;
    println!("> {:?} {:?}", k, v)
  }
  DB0.rm1(&6, None)?;

  {
    println!("# write transaction");
    let tx = TX.w()?;
    let mut db0 = tx.db(&DB0);

    db0.put(&1, &5)?;
    db0.put(&1, &3)?;
    db0.put(&2, &2)?;
    db0.put(&2, &1)?;
    db0.put(&3, &7)?;
    db0.put(&3, &9)?;

    println!("- print all key db0");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- print db1 where key is 2");
    for entry in db0.key_iter(&2)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    println!("- delete key 2 the first value : {}", db0.rm1(&2, None)?);
    println!("- delete key=2 and value=5 : {}", db0.rm1(&2, &5)?);
    println!("- delete key 3 all value : delete number = {}", db0.rm(&3)?);
    println!("- delete key 5 the first value : {}", db0.rm1(&5, None)?);

    println!("- print all key");
    for entry in db0.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    let mut db1 = tx.db(&DB1);
    db1.put(&1, &Hash([1, 2]))?;
    println!("- print all key db1");
    for entry in db1.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    let mut db2 = tx.db(&DB2);
    db2.put(&1, &[1, 2, 3][..])?;
    db2.put(&2, &[4, 6][..])?;

    println!("- print all key db2");
    for entry in db2.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    let mut db3 = tx.db(&DB3);
    db3.put(&[1, 2][..], &[1, 2, 3][..])?;

    println!("- print all key db3");
    for entry in db3.iter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v)
    }

    let mut db4 = tx.db(&DB4);
    let data = Data {
      id: 1234,
      hash: [3, 4],
    };
    //let serialized = data.serialize();
    //println!("- serialized {:?} len {}", serialized, serialized.len());
    db4.put(&1, &data)?;
    println!("- print all key db4");
    for entry in db4.iter(None, None)? {
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

    println!("- print key in revese order");
    for entry in db0.riter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v);
    }

    let db4 = tx.db(&DB4);
    println!("- db4 : print key in revese order");
    for entry in db4.riter(None, None)? {
      let (k, v) = entry?;
      println!("> {:?} {:?}", k, v);
    }
  }

  Ok(())
}
