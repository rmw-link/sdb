use sanakirja::Env;
use std::fs::create_dir_all;
use std::path::PathBuf;

pub struct Sdb {
  pub env: Env,
}

impl Sdb {
  pub fn new(dir: &PathBuf, filename: &str) -> Self {
    create_dir_all(&dir).unwrap();
    let env = Env::new(
      dir.join(filename),
      1 << 21, // 初始化的文件大小
      2,       // 可以同时存活的最大版本数，之前mut_txn_begin必须等待老读者停止
    )
    .unwrap();
    Sdb { env }
  }
}
