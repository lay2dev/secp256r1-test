use super::{
  blake160, build_resolved_tx, gen_tx, gen_tx_with_grouped_args, DummyDataLoader, ALWAYS_SUCCESS,
   MAX_CYCLES,
};
use ckb_crypto::secp::Generator;
use ckb_error::assert_error_eq;
use ckb_script::{ScriptError, TransactionScriptsVerifier};
use ckb_types::{
  bytes::Bytes,
  core::ScriptHashType,
  packed::{CellOutput, Script},
  prelude::*,
};
use rand::thread_rng;

fn build_anyone_can_pay_script(args: Bytes) -> Script {
  let sighash_all_cell_data_hash = CellOutput::calc_data_hash(&ALWAYS_SUCCESS);
  Script::new_builder()
      .args(args.pack())
      .code_hash(sighash_all_cell_data_hash.clone())
      .hash_type(ScriptHashType::Data.into())
      .build()
}


#[test]
fn test_unlock_by_anyone() {
  let mut data_loader = DummyDataLoader::new();
  let privkey = Generator::random_privkey();
  let pubkey = privkey.pubkey().expect("pubkey");
  let pubkey_hash = blake160(&pubkey.serialize());

  let script = build_anyone_can_pay_script(pubkey_hash.to_owned());
  let tx = gen_tx(&mut data_loader, pubkey_hash);
  let output = tx.outputs().get(0).unwrap();
  let tx = tx
      .as_advanced_builder()
      .set_witnesses(Vec::new())
      .set_outputs(vec![output
          .as_builder()
          .lock(script)
          .capacity(44u64.pack())
          .build()])
      .build();

  let resolved_tx = build_resolved_tx(&data_loader, &tx);
  let verifier = TransactionScriptsVerifier::new(&resolved_tx, &data_loader);
  let verify_result = verifier.verify(MAX_CYCLES);
  verify_result.expect("pass");
}
