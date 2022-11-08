use crate::{environment::Environment, objects::Round, CoordinatorError, CoordinatorState};
use phase1::helpers::CurveKind;

use memmap::MmapMut;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};
use zexe_algebra::{Bls12_377, BW6_761};

/// A data structure representing all possible types of keys in storage.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Locator {
    CoordinatorState,
    RoundHeight,
    RoundState(u64),
    RoundFile(u64),
    ContributionFile(u64, u64, u64, bool),
}

/// A data structure representing all possible types of values in storage.
#[derive(Debug, Clone)]
pub enum Object {
    CoordinatorState(CoordinatorState),
    RoundHeight(u64),
    RoundState(Round),
    RoundFile(Vec<u8>),
    ContributionFile(Vec<u8>),
}

impl Object {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Object::CoordinatorState(state) => {
                serde_json::to_vec_pretty(state).expect("coordinator state to bytes failed")
            }
            Object::RoundHeight(height) => serde_json::to_vec(height).expect("round height to bytes failed"),
            Object::RoundState(round) => serde_json::to_vec_pretty(round).expect("round state to bytes failed"),
            Object::RoundFile(round) => round.to_vec(),
            Object::ContributionFile(contribution) => contribution.to_vec(),
        }
    }

    /// Returns the size in bytes of the object.
    pub fn size(&self) -> u64 {
        match self {
            Object::CoordinatorState(_) => self.to_bytes().len() as u64,
            Object::RoundHeight(_) => self.to_bytes().len() as u64,
            Object::RoundState(_) => self.to_bytes().len() as u64,
            Object::RoundFile(round) => round.len() as u64,
            Object::ContributionFile(contribution) => contribution.len() as u64,
        }
    }

    /// Returns the expected file size of an aggregated round.
    pub fn round_file_size(environment: &Environment) -> u64 {
        let compressed = environment.compressed_inputs();
        let settings = environment.parameters();
        let (_, _, curve, _, _, _) = settings;
        match curve {
            CurveKind::Bls12_377 => round_filesize!(Bls12_377, settings, compressed),
            CurveKind::BW6 => round_filesize!(BW6_761, settings, compressed),
        }
    }

    /// Returns the expected file size of a chunked contribution.
    pub fn contribution_file_size(environment: &Environment, chunk_id: u64, verified: bool) -> u64 {
        let settings = environment.parameters();
        let (_, _, curve, _, _, _) = settings;
        let compressed = match verified {
            // The verified contribution file is used as *input* in the next computation.
            true => environment.compressed_inputs(),
            // The unverified contribution file the *output* of the current computation.
            false => environment.compressed_outputs(),
        };
        match (curve, verified) {
            (CurveKind::Bls12_377, true) => verified_contribution_size!(Bls12_377, settings, chunk_id, compressed),
            (CurveKind::Bls12_377, false) => unverified_contribution_size!(Bls12_377, settings, chunk_id, compressed),
            (CurveKind::BW6, true) => verified_contribution_size!(BW6_761, settings, chunk_id, compressed),
            (CurveKind::BW6, false) => unverified_contribution_size!(BW6_761, settings, chunk_id, compressed),
        }
    }
}

pub(crate) enum Lock<'a, T> {
    Read(RwLockReadGuard<'a, T>),
    Write(RwLockWriteGuard<'a, T>),
}

impl<'a, T> Deref for Lock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Lock::Read(read_guard) => read_guard.deref(),
            Lock::Write(write_guard) => write_guard.deref(),
        }
    }
}

impl<'a, T> DerefMut for Lock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Lock::Read(_) => panic!("Cannot mutably dereference a read-only lock"),
            Lock::Write(write_guard) => write_guard,
        }
    }
}

pub(crate) type StorageLock<'a> = Lock<'a, Box<dyn Storage>>;

// pub type StorageWrite<'a> = RwLockWriteGuard<'a, Box<dyn Storage>>;

// TODO (howardwu): Genericize this if necessary for remote objects.
//  Alternatively, usage of temporary memory-backed local files can also work.
pub type ObjectReader<'a> = RwLockReadGuard<'a, MmapMut>;
pub type ObjectWriter<'a> = RwLockWriteGuard<'a, MmapMut>;

/// A standard model for storage.
pub trait Storage: Send + Sync + StorageLocator + StorageObject {
    /// Loads a new instance of `Storage`.
    fn load(environment: &Environment) -> Result<Self, CoordinatorError>
    where
        Self: Sized;

    /// Initializes the location corresponding to the given locator with the given size.
    fn initialize(&mut self, locator: Locator, size: u64) -> Result<(), CoordinatorError>;

    /// Returns `true` if a given locator exists in storage. Otherwise, returns `false`.
    fn exists(&self, locator: &Locator) -> bool;

    /// Returns `true` if a given locator is opened in storage. Otherwise, returns `false`.
    fn is_open(&self, locator: &Locator) -> bool;

    /// Returns a copy of an object at the given locator in storage, if it exists.
    fn get(&self, locator: &Locator) -> Result<Object, CoordinatorError>;

    /// Inserts a new object at the given locator into storage, if it does not exist.
    fn insert(&mut self, locator: Locator, object: Object) -> Result<(), CoordinatorError>;

    /// Updates an existing object for the given locator in storage, if it exists.
    fn update(&mut self, locator: &Locator, object: Object) -> Result<(), CoordinatorError>;

    /// Copies the object in the given source locator to the given destination locator.
    fn copy(&mut self, source_locator: &Locator, destination_locator: &Locator) -> Result<(), CoordinatorError>;

    /// Removes a object from storage for a given locator.
    fn remove(&mut self, locator: &Locator) -> Result<(), CoordinatorError>;

    /// Returns the size of the object stored at the given locator.
    fn size(&self, locator: &Locator) -> Result<u64, CoordinatorError>;
}

pub trait StorageLocator {
    /// Returns a locator path string corresponding to the given locator.
    fn to_path(&self, locator: &Locator) -> Result<String, CoordinatorError>;

    /// Returns a locator corresponding to the given locator path string.
    fn to_locator(&self, path: &str) -> Result<Locator, CoordinatorError>;
}

pub trait StorageObject {
    /// Returns an object reader for the given locator.
    fn reader(&self, locator: &Locator) -> Result<ObjectReader, CoordinatorError>;

    /// Returns an object writer for the given locator.
    fn writer(&self, locator: &Locator) -> Result<ObjectWriter, CoordinatorError>;
}
