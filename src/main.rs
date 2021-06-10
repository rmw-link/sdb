fn main() {
  let env = {
    let p = Path::new(&*DIR).join("kv");
    create_dir_all(&p).unwrap();
    let p = p.join("db");
    Env::new(
      p,
      1 << 21, // 文件大小
      2,       // 可以同时存活的最大版本数，之前mut_txn_begin必须等待老读者停止
    )
    .unwrap()
  };
}
