use crate::{
    environment::{Environment, Parameters, Testing},
    objects::{Participant, Round},
    storage::{Storage, StorageLock},
    Coordinator,
    CoordinatorError,
};

use chrono::{DateTime, TimeZone, Utc};
use once_cell::sync::Lazy;
use serde_diff::{Diff, SerdeDiff};
use serial_test::serial;
use std::{
    path::Path,
    sync::{Arc, RwLock},
};
use tracing::{error, info, warn};

lazy_static! {
    /// Environment for testing purposes only.
    pub static ref TEST_ENVIRONMENT: Environment = Testing::from(Parameters::Test8Chunks).into();

    /// Environment for testing purposes only.
    pub static ref TEST_ENVIRONMENT_3: Environment = Testing::from(Parameters::Test3Chunks).into();

    /// Round start datetime for testing purposes only.
    pub static ref TEST_STARTED_AT: DateTime<Utc> = Utc.ymd(1970, 1, 1).and_hms(0, 1, 1);

    /// Contributor ID for testing purposes only.
    pub static ref TEST_CONTRIBUTOR_ID: Lazy<Participant> = Lazy::new(|| test_coordinator_contributor(&TEST_ENVIRONMENT).unwrap());

    /// Contributor ID 2 for testing purposes only.
    pub static ref TEST_CONTRIBUTOR_ID_2: Lazy<Participant> = Lazy::new(|| Participant::Contributor(format!("testing-coordinator-contributor-2")));

    /// Contributor ID 3 for testing purposes only.
    pub static ref TEST_CONTRIBUTOR_ID_3: Lazy<Participant> = Lazy::new(|| Participant::Contributor(format!("testing-coordinator-contributor-3")));

    /// Verifier ID for testing purposes only.
    pub static ref TEST_VERIFIER_ID: Lazy<Participant> = Lazy::new(|| test_coordinator_verifier(&TEST_ENVIRONMENT).unwrap());

    /// Verifier ID 2 for testing purposes only.
    pub static ref TEST_VERIFIER_ID_2: Lazy<Participant> = Lazy::new(|| Participant::Verifier(format!("testing-coordinator-verifier-2")));

    /// Contributor IDs for testing purposes only.
    pub static ref TEST_CONTRIBUTOR_IDS: Lazy<Vec<Participant>> = Lazy::new(|| vec![Lazy::force(&TEST_CONTRIBUTOR_ID).clone()]);

    /// Verifier IDs for testing purposes only.
    pub static ref TEST_VERIFIER_IDS: Lazy<Vec<Participant>> =  Lazy::new(|| vec![Lazy::force(&TEST_VERIFIER_ID).clone()]);
}

pub fn test_coordinator(environment: &Environment) -> anyhow::Result<Coordinator> {
    info!("Starting coordinator");
    let coordinator = Coordinator::new(environment.clone())?;
    info!("Coordinator is ready");
    Ok(coordinator)
}

pub fn test_coordinator_contributor(environment: &Environment) -> anyhow::Result<Participant> {
    Ok(environment
        .coordinator_contributors()
        .first()
        .ok_or(CoordinatorError::ContributorsMissing)?
        .clone())
}

pub fn test_coordinator_verifier(environment: &Environment) -> anyhow::Result<Participant> {
    Ok(environment
        .coordinator_verifiers()
        .first()
        .ok_or(CoordinatorError::VerifierMissing)?
        .clone())
}

pub fn initialize_test_environment(environment: &Environment) {
    #[cfg(not(feature = "silent"))]
    test_logger();

    clear_test_storage(environment);
}

#[cfg(not(feature = "silent"))]
pub(crate) fn test_logger() {
    use once_cell::sync::OnceCell;
    use tracing::Level;

    static INSTANCE: OnceCell<()> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let subscriber = tracing_subscriber::fmt().with_max_level(Level::TRACE).finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    });
}

/// Clears the transcript directory for testing purposes only.
fn clear_test_storage(environment: &Environment) {
    let path = environment.local_base_directory();
    if Path::new(path).exists() {
        warn!("Coordinator is clearing {:?}", &path);
        match std::fs::remove_dir_all(&path) {
            Ok(_) => (),
            Err(error) => error!(
                "The testing framework tried to clear the test transcript and failed. {}",
                error
            ),
        }
        warn!("Coordinator cleared {:?}", &path);
    }
}

/// Initializes a test storage object.
pub fn test_storage(environment: &Environment) -> Arc<RwLock<Box<dyn Storage>>> {
    Arc::new(RwLock::new(environment.storage().unwrap()))
}

/// Loads the reference JSON object with a serialized round for testing purposes only.
pub fn test_round_0_json() -> anyhow::Result<Round> {
    Ok(serde_json::from_str(include_str!("resources/test_round_0.json"))?)
}

/// Loads the reference JSON object with a serialized round for testing purposes only.
pub fn test_round_1_initial_json() -> anyhow::Result<Round> {
    Ok(serde_json::from_str(include_str!(
        "resources/test_round_1_initial.json"
    ))?)
}

/// Creates the initial round for testing purposes only.
pub fn test_round_0() -> anyhow::Result<Round> {
    // Define test storage.
    let test_storage = test_storage(&TEST_ENVIRONMENT);
    let storage = StorageLock::Write(test_storage.write().unwrap());

    Ok(Round::new(
        &TEST_ENVIRONMENT,
        &storage,
        0, /* height */
        *TEST_STARTED_AT,
        vec![],
        TEST_VERIFIER_IDS.to_vec(),
    )?)
}

/// Prints the difference in JSON objects between `a` and `b`.
pub fn print_diff<S: SerdeDiff>(a: &S, b: &S) {
    println!(
        "\nDifference(s) between left and right values\n-------------------------------------------\n{}\n",
        serde_json::to_string_pretty(&Diff::serializable(a, b)).unwrap()
    );
}

#[test]
#[serial]
fn test_round_0_matches() {
    initialize_test_environment(&TEST_ENVIRONMENT);

    let expected = test_round_0_json().unwrap();
    let candidate = test_round_0().unwrap();

    // Print the differences in JSON if they do not match.
    if candidate != expected {
        print_diff(&expected, &candidate);
    }
    assert_eq!(candidate, expected);
}
