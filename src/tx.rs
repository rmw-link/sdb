use sanakirja::{Env, MutTxn, Txn};

pub type MutTxnEnv<'a> = MutTxn<&'a Env, ()>;
pub type TxnEnv<'a> = Txn<&'a Env>;

pub struct Tx {
  pub(crate) env: Env,
}
