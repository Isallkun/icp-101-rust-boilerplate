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

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for NFTCertificate {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
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
            msg: format!("an NFT with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn create_nft(payload: NFTPayload) -> Option<NFTCertificate> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let nft = NFTCertificate {
        id,
        owner: payload.owner,
        metadata: payload.metadata,
        created_at: time(),
    };
    do_insert(&nft);
    Some(nft)
}

// helper method to perform insert.
fn do_insert(nft: &NFTCertificate) {
    STORAGE.with(|service| service.borrow_mut().insert(nft.id, nft.clone()));
}

#[ic_cdk::update]
fn delete_nft(id: u64) -> Result<NFTCertificate, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(nft) => Ok(nft),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete an NFT with id={}. NFT not found.",
                id
            ),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// a helper method to get an NFT by id. used in get_nft
fn _get_nft(id: &u64) -> Option<NFTCertificate> {
    STORAGE.with(|service| service.borrow().get(id))
}

// need this to generate candid
ic_cdk::export_candid!();