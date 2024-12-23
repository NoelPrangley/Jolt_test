#![cfg_attr(feature = "guest", no_std)]
#![no_main]

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub const BATCH_SIZE: usize = 1;

#[derive(Clone, Serialize, Deserialize)]
pub struct ScoreInput {
    #[serde(with = "BigArray")]
    pub val: [u32; BATCH_SIZE],
}

#[derive(Serialize, Deserialize)]
pub struct ScoreOutput(#[serde(with = "BigArray")] pub [u128; BATCH_SIZE]);

#[jolt::provable]
fn fib(input: ScoreInput) -> ScoreOutput {
    let default_input: u128 = 0;
    let mut new_outs = [default_input; BATCH_SIZE];
    new_outs[0] = calculate_single_fib(input.val[0]);

    ScoreOutput(new_outs)
}

fn calculate_single_fib(
    val: u32
) -> u128 {
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    let mut sum: u128;
    for _ in 1..val {
        sum = a + b;
        a = b;
        b = sum;
    }
    let new_out = b;
    return new_out;
}