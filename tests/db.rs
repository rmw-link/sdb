use sdb::{direct_repr, Db, DbU, Storable, Tx, UnsizedStorable};
use static_init::dynamic;
use std::env;
use std::path::Path;

#[dynamic]
pub static DIR: String = env::current_exe()
  .unwrap()
  .parent()
  .unwrap()
  .parent()
  .unwrap()
  .display()
  .to_string();

#[dynamic]
pub static TX: Tx = {
  let dir = Path::new(&*DIR).join("db");

  println!("DATABASE DIR {}", dir.display().to_string());

  //use sdb::SdbArgs::{InitSize, MaxTx, Filename};

  Tx::new(
    &dir,
    &[
      //MaxTx(3),
      //Filename("sdb"),
      //InitSize(1<<21),
    ],
  )
};

#[dynamic]
pub static DB0: Db<'static, u64, u64> = TX.db(0);

#[derive(Default, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Hash([u8; 32]);

direct_repr!(Hash);

#[dynamic]
pub static DB1: Db<'static, u64, Hash> = TX.db(1);
