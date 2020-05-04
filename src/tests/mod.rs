mod always_success;

use ckb_crypto::secp::Privkey;
use ckb_script::DataLoader;
use ckb_types::{
    bytes::Bytes,
    core::{
        cell::{CellMeta, CellMetaBuilder, ResolvedTransaction},
        BlockExt, Capacity, DepType, EpochExt, HeaderView, ScriptHashType, TransactionBuilder,
        TransactionView,
    },
    packed::{
        self, Byte32, CellDep, CellInput, CellOutput, OutPoint, Script, WitnessArgs,
        WitnessArgsBuilder,
    },
    prelude::*,
    H256,
};
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

pub const MAX_CYCLES: u64 = std::u64::MAX;

lazy_static! {
    // pub static ref ANYONE_CAN_PAY: Bytes =
    //     Bytes::from(&include_bytes!("../../specs/cells/anyone_can_pay")[..]);
    // pub static ref SECP256K1_DATA_BIN: Bytes =
    //     Bytes::from(&include_bytes!("../../specs/cells/secp256k1_data")[..]);
    pub static ref ALWAYS_SUCCESS: Bytes =
        Bytes::from(&include_bytes!("../../specs/cells/always_success")[..]);
}

#[derive(Default)]
pub struct DummyDataLoader {
    pub cells: HashMap<OutPoint, (CellOutput, Bytes)>,
    pub headers: HashMap<Byte32, HeaderView>,
    pub epoches: HashMap<Byte32, EpochExt>,
}

impl DummyDataLoader {
    fn new() -> Self {
        Self::default()
    }
}

impl DataLoader for DummyDataLoader {
    // load Cell Data
    fn load_cell_data(&self, cell: &CellMeta) -> Option<(Bytes, Byte32)> {
        cell.mem_cell_data.clone().or_else(|| {
            self.cells
                .get(&cell.out_point)
                .map(|(_, data)| (data.clone(), CellOutput::calc_data_hash(&data)))
        })
    }
    // load BlockExt
    fn get_block_ext(&self, _hash: &Byte32) -> Option<BlockExt> {
        unreachable!()
    }

    // load header
    fn get_header(&self, block_hash: &Byte32) -> Option<HeaderView> {
        self.headers.get(block_hash).cloned()
    }

    // load EpochExt
    fn get_block_epoch(&self, block_hash: &Byte32) -> Option<EpochExt> {
        self.epoches.get(block_hash).cloned()
    }
}

pub fn blake160(message: &[u8]) -> Bytes {
    Bytes::from(&ckb_hash::blake2b_256(message)[..20])
}


pub fn gen_tx(dummy: &mut DummyDataLoader, lock_args: Bytes) -> TransactionView {
    let mut rng = thread_rng();
    gen_tx_with_grouped_args(dummy, vec![(lock_args, 1)], &mut rng)
}

pub fn gen_tx_with_grouped_args<R: Rng>(
    dummy: &mut DummyDataLoader,
    grouped_args: Vec<(Bytes, usize)>,
    rng: &mut R,
) -> TransactionView {
    // setup sighash_all dep
    let sighash_all_out_point = {
        let contract_tx_hash = {
            let mut buf = [0u8; 32];
            rng.fill(&mut buf);
            buf.pack()
        };
        OutPoint::new(contract_tx_hash.clone(), 0)
    };
    // dep contract code
    // let sighash_all_cell = CellOutput::new_builder()
    //     .capacity(
    //         Capacity::bytes(ANYONE_CAN_PAY.len())
    //             .expect("script capacity")
    //             .pack(),
    //     )
    //     .build();
    // let sighash_all_cell_data_hash = CellOutput::calc_data_hash(&ANYONE_CAN_PAY);
    // dummy.cells.insert(
    //     sighash_all_out_point.clone(),
    //     (sighash_all_cell, ANYONE_CAN_PAY.clone()),
    // );
    // always success
    let always_success_out_point = {
        let contract_tx_hash = {
            let mut buf = [0u8; 32];
            rng.fill(&mut buf);
            buf.pack()
        };
        OutPoint::new(contract_tx_hash.clone(), 0)
    };
    let always_success_cell = CellOutput::new_builder()
        .capacity(
            Capacity::bytes(ALWAYS_SUCCESS.len())
                .expect("script capacity")
                .pack(),
        )
        .build();
    let always_success_cell_data_hash = CellOutput::calc_data_hash(&ALWAYS_SUCCESS);

    println!("code_hash {}", always_success_cell_data_hash);

    dummy.cells.insert(
        always_success_out_point.clone(),
        (always_success_cell, ALWAYS_SUCCESS.clone()),
    );
    // setup secp256k1_data dep
    // let secp256k1_data_out_point = {
    //     let tx_hash = {
    //         let mut buf = [0u8; 32];
    //         rng.fill(&mut buf);
    //         buf.pack()
    //     };
    //     OutPoint::new(tx_hash, 0)
    // };
    // let secp256k1_data_cell = CellOutput::new_builder()
    //     .capacity(
    //         Capacity::bytes(SECP256K1_DATA_BIN.len())
    //             .expect("data capacity")
    //             .pack(),
    //     )
    //     .build();
    // dummy.cells.insert(
    //     secp256k1_data_out_point.clone(),
    //     (secp256k1_data_cell, SECP256K1_DATA_BIN.clone()),
    // );
    // setup default tx builder
    let dummy_capacity = Capacity::shannons(42);
    let mut tx_builder = TransactionBuilder::default()
        // .cell_dep(
        //     CellDep::new_builder()
        //         .out_point(sighash_all_out_point)
        //         .dep_type(DepType::Code.into())
        //         .build(),
        // )
        .cell_dep(
            CellDep::new_builder()
                .out_point(always_success_out_point)
                .dep_type(DepType::Code.into())
                .build(),
        )
        // .cell_dep(
        //     CellDep::new_builder()
        //         .out_point(secp256k1_data_out_point)
        //         .dep_type(DepType::Code.into())
        //         .build(),
        // )
        .output(
            CellOutput::new_builder()
                .capacity(dummy_capacity.pack())
                .build(),
        )
        .output_data(Bytes::new().pack());

    for (args, inputs_size) in grouped_args {
        // setup dummy input unlock script
        for _ in 0..inputs_size {
            let previous_tx_hash = {
                let mut buf = [0u8; 32];
                rng.fill(&mut buf);
                buf.pack()
            };
            let previous_out_point = OutPoint::new(previous_tx_hash, 0);
            let script = Script::new_builder()
                .args(args.pack())
                // .code_hash(sighash_all_cell_data_hash.clone())
                .code_hash(always_success_cell_data_hash.clone())
                .hash_type(ScriptHashType::Data.into())
                .build();
            let previous_output_cell = CellOutput::new_builder()
                .capacity(dummy_capacity.pack())
                .lock(script)
                .build();
            dummy.cells.insert(
                previous_out_point.clone(),
                (previous_output_cell.clone(), Bytes::new()),
            );
            let mut random_extra_witness = [0u8; 32];
            rng.fill(&mut random_extra_witness);
            let witness_args = WitnessArgsBuilder::default()
                .extra(Bytes::from(random_extra_witness.to_vec()).pack())
                .build();
            tx_builder = tx_builder
                .input(CellInput::new(previous_out_point, 0))
                .witness(witness_args.as_bytes().pack());
        }
    }

    tx_builder.build()
}


pub fn build_resolved_tx(
    data_loader: &DummyDataLoader,
    tx: &TransactionView,
) -> ResolvedTransaction {
    let resolved_cell_deps = tx
        .cell_deps()
        .into_iter()
        .map(|dep| {
            let deps_out_point = dep.clone();
            let (dep_output, dep_data) =
                data_loader.cells.get(&deps_out_point.out_point()).unwrap();
            CellMetaBuilder::from_cell_output(dep_output.to_owned(), dep_data.to_owned())
                .out_point(deps_out_point.out_point().clone())
                .build()
        })
        .collect();

    let mut resolved_inputs = Vec::new();
    for i in 0..tx.inputs().len() {
        let previous_out_point = tx.inputs().get(i).unwrap().previous_output();
        let (input_output, input_data) = data_loader.cells.get(&previous_out_point).unwrap();
        resolved_inputs.push(
            CellMetaBuilder::from_cell_output(input_output.to_owned(), input_data.to_owned())
                .out_point(previous_out_point)
                .build(),
        );
    }

    ResolvedTransaction {
        transaction: tx.clone(),
        resolved_cell_deps,
        resolved_inputs,
        resolved_dep_groups: vec![],
    }
}
