use desse::{Desse, DesseSized};
use sdb::{encode_decode, sdb, Db, DbEv, DbU, Encode, Storable, Tx, UnsizedStorable};
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
pub struct Hash(pub [u8; 2]);

sdb!(Hash);

#[dynamic]
pub static DB1: Db<'static, u64, Hash> = TX.db(1);

#[dynamic] // DbU = Db & btree::page_unsized::Page
pub static DB2: DbU<'static, u64, [u8]> = TX.db(2);

#[dynamic]
pub static DB3: DbU<'static, [u8], [u8]> = TX.db(3);

#[derive(Default, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Data {
  pub hash: [u8; 3],
  pub id: u64,
}

sdb!(Data);

#[dynamic]
pub static DB4: Db<'static, u64, Data> = TX.db(4);

#[derive(DesseSized, Desse)]
pub struct Data2 {
  pub hash: [u8; 3],
  pub id: u64,
}

#[derive(Default, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug, DesseSized, Desse)]
pub struct Data2Desse([u8; Data2::SIZE]);

use sdb::direct_repr;
direct_repr!(Data2Desse);

#[dynamic]
pub static DB5: DbEv<'static, u64, Data2Desse, Data2> = TX.db(5);

impl Encode<Data2Desse> for Data2 {
  #[inline]
  fn encode<R: Sized>(&self, next: &mut dyn FnMut(&Data2Desse) -> R) -> R {
    next(&Data2Desse(self.serialize()))
  }
  /*
  #[inline]
  fn decode(val: &Data2Desse) -> &Data2 {
    &Data2::deserialize_from(&val.0)
  }
  */
}
