#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct NFTCertificate {
    id: u64,
    owner: String,
    metadata: String,
    created_at: u64,
}

// A trait that must be implemented for a struct that is stored in a stable struct
impl Storable for NFTCertificate {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for NFTCertificate {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, NFTCertificate, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct NFTPayload {
    owner: String,
    metadata: String,
}

#[ic_cdk::query]
fn get_nft(id: u64) -> Result<NFTCertificate, Error> {
    match _get_nft(&id) {
        Some(nft) => Ok(nft),
        None => Err(Error::NotFound {
            msg: format!("An NFT with id={} not found", id),
        }),
    }
}

// Validate NFTPayload before processing
fn validate_payload(payload: &NFTPayload) -> Result<(), Error> {
    if payload.owner.trim().is_empty() {
        return Err(Error::InvalidInput { msg: "Owner is required".to_string() });
    }
    if payload.metadata.trim().is_empty() {
        return Err(Error::InvalidInput { msg: "Metadata is required".to_string() });
    }
    Ok(())
}

#[ic_cdk::update]
fn create_nft(payload: NFTPayload) -> Result<NFTCertificate, Error> {
    // Validate the payload
    validate_payload(&payload)?;

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");
    
    let nft = NFTCertificate {
        id,
        owner: payload.owner,
        metadata: payload.metadata,
        created_at: time(),
    };

    do_insert(&nft);
    Ok(nft)
}

// Helper method to perform insert
fn do_insert(nft: &NFTCertificate) {
    STORAGE.with(|service| service.borrow_mut().insert(nft.id, nft.clone()));
}

#[ic_cdk::update]
fn delete_nft(id: u64) -> Result<NFTCertificate, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(nft) => Ok(nft),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete an NFT with id={}. NFT not found.", id),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String },
}

// Helper method to get an NFT by id, used in get_nft
fn _get_nft(id: &u64) -> Option<NFTCertificate> {
    STORAGE.with(|service| service.borrow().get(id))
}

// Need this to generate candid
ic_cdk::export_candid!();
