use sanakirja::Env;
use std::fs::create_dir_all;
use std::path::PathBuf;

pub enum SdbArgs<'a> {
  Dir(&'a PathBuf),
  Filename(&'a str),
  InitDbSize(u64),
  MaxTx(usize),
}

use SdbArgs::*;

pub struct Sdb {
  pub env: Env,
}

impl Sdb {
  pub fn new(args: &[SdbArgs]) -> Self {
    let mut dir = None;
    let mut filename = None;
    let mut init_db_size = None;
    let mut max_tx = None;

    for arg in args {
      match arg {
        Dir(i) => dir = (*i).into(),
        Filename(i) => filename = i.to_string().into(),
        InitDbSize(i) => init_db_size = (*i).into(),
        MaxTx(i) => max_tx = (*i).into(),
      }
    }

    let dir = dir.unwrap();
    let filename = filename.unwrap_or_else(|| "sdb".into());
    let init_db_size = init_db_size.unwrap_or(1 << 21);
    let max_tx = max_tx.unwrap_or(3);

    create_dir_all(&dir).unwrap();
    let env = Env::new(dir.join(filename), init_db_size, max_tx).unwrap();
    Sdb { env }
  }
}
