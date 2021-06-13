use lazy_static::lazy_static;
use sdb::{Db, Tx};
use std::env;
use std::path::Path;

lazy_static! {

pub static ref DIR: String = env::current_exe()
  .unwrap()
  .parent()
  .unwrap()
  .parent()
  .unwrap()
  .display()
  .to_string();

pub static ref TX: Tx = {
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

pub static ref DB0: Db<'static, u64, u64> = TX.db(0);

//pub static ref DB1: Db<'static, u64, u64> = TX.db(1);

}
