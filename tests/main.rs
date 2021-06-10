use anyhow::Result;
use std::env;

#[test]
fn main() -> Result<()> {
  let dir = env::current_exe()?
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .display()
    .to_string();

  println!("{}", dir);

  assert_eq!(4, 4);

  Ok(())
}
