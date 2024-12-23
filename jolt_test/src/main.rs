use clap::{Parser, Subcommand};
use guest::{build_fib, ScoreInput, ScoreOutput, BATCH_SIZE};
use jolt_core::jolt::vm::rv32i_vm::{RV32IHyraxProof, Serializable};
use log::{debug, error, info, trace};
use postcard;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use serde_json;
use std::{self, process::ExitCode};

#[derive(Serialize, Deserialize, Debug)]
struct DecodedInputs {
    #[serde(with = "BigArray")]
    val: [u32; BATCH_SIZE],
}
#[derive(Serialize, Deserialize, Debug)]
struct DecodedOutputs {
    #[serde(with = "BigArray")]
    out: [u128; BATCH_SIZE],
}

#[derive(Parser, Clone)]
struct Args {
    #[arg(short, long, default_value = "input.json")]
    input: String,
    #[arg(short, long, default_value = "output.json")]
    output: String,
    #[arg(short, long, default_value = "proof.bin")]
    proof: String,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Prove(Args),
    Verify(Args),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

pub fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    let (prove_fib, verify_fib) = build_fib();

    match &cli.command {
        Commands::Prove(args) => match prove_command(args, prove_fib) {
            Ok(_) => {
                info!("Proof generated successfully");
                ExitCode::SUCCESS
            }
            Err(e) => {
                error!("Error in prove command: {}", e);
                ExitCode::FAILURE
            }
        },
        Commands::Verify(args) => match verify_command(args, verify_fib) {
            Ok(is_valid) => {
                if is_valid {
                    info!("Proof is valid");
                    ExitCode::SUCCESS
                } else {
                    error!("Proof is invalid");
                    ExitCode::FAILURE
                }
            }
            Err(e) => {
                error!("Error in verify command: {}", e);
                error!("Proof is invalid");
                ExitCode::FAILURE
            }
        },
    }
}

fn prove_command(
    args: &Args,
    prove_fib: impl Fn(ScoreInput) -> (ScoreOutput, RV32IHyraxProof),
) -> Result<(), Box<dyn std::error::Error>> {
    let input = read_input_from_file(&args.input)?;

    let (output, proof) = prove_fib(ScoreInput {
        val: input.val
    });

    proof.save_to_file(&args.proof)?;

    let output_str = serde_json::to_string(&output)?;
    std::fs::write(&args.output, output_str)?;

    debug!("Proof inputs: {:?}", proof.proof.program_io.inputs);
    debug!("Proof outputs: {:?}", proof.proof.program_io.outputs);
    let (decoded_inputs, decoded_outputs) = decode_proof_io(&proof);
    trace!("val: {:?}", decoded_inputs.val);
    trace!("new_out: {:?}", decoded_outputs.out);

    debug!("Proof generated and saved successfully");
    Ok(())
}

fn verify_command(
    args: &Args,
    verify_fib: impl Fn(RV32IHyraxProof) -> bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let input = read_input_from_file(&args.input)?;
    let output = read_output_from_file(&args.output)?;

    trace!("Output: {:?}", output.0);

    let mut proof = RV32IHyraxProof::from_file(&args.proof)?;

    // Replace proof io with the input and output passed in
    // to verify that these inputs and outputs were the ones used to generate the proof
    let (decoded_inputs, decoded_outputs) = decode_proof_io(&proof);
    proof.proof.program_io.inputs = postcard::to_stdvec(&input).unwrap();
    proof.proof.program_io.outputs = postcard::to_stdvec(&output).unwrap();

    debug!("Decoded proof inputs:");
    trace!("val: {:?}", decoded_inputs.val);
    trace!("Decoded proof outputs: {:?}", decoded_outputs);

    let is_valid = verify_fib(proof);

    debug!("Proof is {}", if is_valid { "valid" } else { "invalid" });
    Ok(is_valid)
}

fn read_input_from_file(path: &str) -> Result<ScoreInput, Box<dyn std::error::Error>> {
    let file_content = std::fs::read_to_string(path)?;
    serde_json::from_str(&file_content).map_err(Into::into)
}

fn read_output_from_file(path: &str) -> Result<ScoreOutput, Box<dyn std::error::Error>> {
    let file_content = std::fs::read_to_string(path)?;
    serde_json::from_str(&file_content).map_err(Into::into)
}

fn decode_proof_io(proof: &RV32IHyraxProof) -> (DecodedInputs, DecodedOutputs) {
    let decoded_inputs: DecodedInputs =
        postcard::from_bytes(&proof.proof.program_io.inputs).expect("Failed to decode inputs");
    let decoded_outputs: DecodedOutputs =
        postcard::from_bytes(&proof.proof.program_io.outputs).expect("Failed to decode outputs");
    (decoded_inputs, decoded_outputs)
}