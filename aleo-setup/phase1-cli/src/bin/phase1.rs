use phase1::{helpers::CurveKind, CurveParameters, Phase1Parameters};
use phase1_cli::{
    combine,
    contribute,
    new_challenge,
    transform_pok_and_correctness,
    transform_ratios,
    Command,
    Phase1Opts,
};
use setup_utils::{beacon_randomness, derive_rng_from_seed, from_slice, CheckForCorrectness, UseCompression};

use zexe_algebra::{Bls12_377, PairingEngine as Engine, BW6_761};

use gumdrop::Options;
use std::{fs::read_to_string, process, time::Instant};
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{time::ChronoUtc, Subscriber},
};

const CHALLENGE_IS_COMPRESSED: UseCompression = UseCompression::No;
const CONTRIBUTION_IS_COMPRESSED: UseCompression = UseCompression::Yes;
const CHECK_CONTRIBUTION_INPUT_FOR_CORRECTNESS: CheckForCorrectness = CheckForCorrectness::No;

fn execute_cmd<E: Engine>(opts: Phase1Opts) {
    let curve = CurveParameters::<E>::new();
    let parameters = Phase1Parameters::<E>::new(
        opts.contribution_mode,
        opts.chunk_index,
        opts.chunk_size,
        curve,
        opts.proving_system,
        opts.power,
        opts.batch_size,
    );

    let command = opts.clone().command.unwrap_or_else(|| {
        eprintln!("No command was provided.");
        eprintln!("{}", Phase1Opts::usage());
        process::exit(2)
    });

    let now = Instant::now();
    match command {
        Command::New(opt) => {
            new_challenge(CHALLENGE_IS_COMPRESSED, &opt.challenge_fname, &parameters);
        }
        Command::Contribute(opt) => {
            // contribute to the randomness
            let seed = hex::decode(&read_to_string(&opts.seed).expect("should have read seed").trim())
                .expect("seed should be a hex string");
            let rng = derive_rng_from_seed(&seed);
            contribute(
                CHALLENGE_IS_COMPRESSED,
                &opt.challenge_fname,
                CONTRIBUTION_IS_COMPRESSED,
                &opt.response_fname,
                CHECK_CONTRIBUTION_INPUT_FOR_CORRECTNESS,
                &parameters,
                rng,
            );
        }
        Command::Beacon(opt) => {
            // use the beacon's randomness
            // Place block hash here (block number #564321)
            let beacon_hash = hex::decode(&opt.beacon_hash).expect("could not hex decode beacon hash");
            let rng = derive_rng_from_seed(&beacon_randomness(from_slice(&beacon_hash)));
            contribute(
                CHALLENGE_IS_COMPRESSED,
                &opt.challenge_fname,
                CONTRIBUTION_IS_COMPRESSED,
                &opt.response_fname,
                CHECK_CONTRIBUTION_INPUT_FOR_CORRECTNESS,
                &parameters,
                rng,
            );
        }
        Command::VerifyAndTransformPokAndCorrectness(opt) => {
            // we receive a previous participation, verify it, and generate a new challenge from it
            transform_pok_and_correctness(
                CHALLENGE_IS_COMPRESSED,
                &opt.challenge_fname,
                CONTRIBUTION_IS_COMPRESSED,
                &opt.response_fname,
                CHALLENGE_IS_COMPRESSED,
                &opt.new_challenge_fname,
                &parameters,
            );
        }
        Command::VerifyAndTransformRatios(opt) => {
            // we receive a previous participation, verify it, and generate a new challenge from it
            transform_ratios(&opt.response_fname, &parameters);
        }
        Command::Combine(opt) => {
            combine(&opt.response_list_fname, &opt.combined_fname, &parameters);
        }
    };

    let new_now = Instant::now();
    println!("Executing {:?} took: {:?}", opts, new_now.duration_since(now));
}

fn main() {
    Subscriber::builder()
        .with_target(false)
        .with_timer(ChronoUtc::rfc3339())
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let opts: Phase1Opts = Phase1Opts::parse_args_default_or_exit();

    match opts.curve_kind {
        CurveKind::Bls12_377 => execute_cmd::<Bls12_377>(opts),
        CurveKind::BW6 => execute_cmd::<BW6_761>(opts),
    };
}
