//! pbc-did-registry/

#[macro_use]
extern crate pbc_contract_codegen;

use pbc_contract_common::address::{Address, AddressType};
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::sorted_vec_map::SortedVecMap;

#[state]
pub struct ContractState {
    owner: Address,
    owner_did: String,
    nonce: SortedVecMap<Address, u128>, // Key: Address, Value: Nonce
    dids: SortedVecMap<String, Address>, // Key: DID, Value: Controller Address
    attributes: SortedVecMap<String, Vec<String>>, // Key: DID, Value: Attributes list
}

#[init]
fn initialize(
    ctx: ContractContext,
) -> ContractState {

    let mut full_identifier: [u8; 21] = [0; 21];
    full_identifier[0] = ctx.sender.address_type as u8;
    full_identifier[1..21].clone_from_slice(&ctx.sender.identifier);
    
    let mut did: String = "did:metablox:0x".to_owned();
    did.push_str(&hex::encode(full_identifier));

    let mut nonce_storage: SortedVecMap<Address, u128> = SortedVecMap::new();
    let mut did_storage: SortedVecMap<String, Address> = SortedVecMap::new();
    let mut attribute_storage: SortedVecMap<String, Vec<String>> = SortedVecMap::new();
    
    nonce_storage.insert(ctx.sender, 0x01);
    did_storage.insert(did.clone(), ctx.sender);
    let mut new_attribute : Vec<String> = Vec::new();
    new_attribute.push("Issuer".to_string());
    attribute_storage.insert(did.clone(), new_attribute);


    let state = ContractState {
        owner: ctx.sender,
        nonce: nonce_storage,
        dids: did_storage,
        owner_did:  did,
        attributes: attribute_storage,
    };

    state
}

/*
#[action(shortname = 0x01)]
pub fn nonce(    
    context: ContractContext,
    state: ContractState,
) -> u128{

    if state.nonce.contains_key(&context.sender) {
        state.nonce.get(&context.sender).copied().unwrap()
    } else {
        0x00 as u128
    }
}
*/

#[action(shortname = 0x02)]
pub fn did_lookup(    
    _context: ContractContext,
    state: ContractState,
    did: String
) -> Address{
    let empty_identifier : [u8; 20] = [0; 20];
    let controller = Address {
        address_type: AddressType::Account,
        identifier : empty_identifier,
    };

    if state.dids.contains_key(&did) {
        state.dids.get(&did).unwrap().clone()
    } else {
        controller
    }
}

#[action(shortname = 0x03)]
pub fn register_did(    
    context: ContractContext,
    mut state: ContractState,
    did: String
) -> ContractState{

    let parts = did.split(":").collect::<Vec<&str>>();
    assert!(parts.len() == 3, "DID Format Incorrect! Part len '{}' while expecting 3", parts.len());
    assert!(parts[0].to_string() == "did".to_string(), "DID Format Incorrect! Scheme needs to be 'did'");
    assert!(parts[1].to_string() == "metablox".to_string(), "DID Format Incorrect! Method needs to be 'metablox'");

    if !state.nonce.contains_key(&context.sender) {
        state.nonce.insert(context.sender, 0x01);
    } else {
        *state.nonce.get_mut(&context.sender).unwrap() += 1;
    }


    if state.dids.contains_key(&did) {
        let controller: Address = state.dids.get(&did).copied().unwrap();
            if controller == context.sender {
                panic!("DID Already registered!")
            } else {
                panic!("DID registered by another Controller!")
            }
    } else {
        state.dids.insert(did, context.sender);

        state
    }
}

#[action(shortname = 0x04)]
pub fn set_attribute(
    context: ContractContext,
    mut state: ContractState,
    new_attributes: Vec<String>,
    did: String,
) -> ContractState{

    if state.dids.contains_key(&did) {
        let controller = state.dids.get(&did).unwrap().clone();

        if controller != context.sender {
            panic!("Not Authorized!")
        }

        if state.attributes.contains_key(&did) {
            state.attributes.remove(&did);
        }
        state.attributes.insert(did, new_attributes);

    } else {
        panic!("DID Not Exist!")
    }

    if !state.nonce.contains_key(&context.sender) {
        state.nonce.insert(context.sender, 0x01);
    } else {
        *state.nonce.get_mut(&context.sender).unwrap() += 1;
    }

    state
}

#[action(shortname = 0x05)]
pub fn get_attribute(
    _context: ContractContext,
    state: ContractState,
    did: String,
) -> Vec<String>{

    let mut attribute_vec : Vec<String> = Vec::new();

    if state.dids.contains_key(&did) {
    
        if state.attributes.contains_key(&did) {
            attribute_vec = state.attributes.get(&did).unwrap().clone();
        } else {
            attribute_vec.push("".to_string());
        }
        
        
    } else {
        panic!("DID Not Exist!")
    }

    attribute_vec
}