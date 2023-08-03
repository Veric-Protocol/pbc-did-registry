//! pbc-did-registry/

#[macro_use]
extern crate pbc_contract_codegen;

use pbc_contract_common::address::Address;
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::sorted_vec_map::SortedVecMap;

#[state]
pub struct ContractState {
    owner: Address,
    owner_did: String,
    nonce: SortedVecMap<Address, u128>, // Key: Address, Value: Nonce
    dids: SortedVecMap<String, Address>, // Key: DID, Value: Controller Address
    attributes: SortedVecMap<String, Vec<String>>, // Key: DID, Value: Attributes list
    delegates: SortedVecMap<String, SortedVecMap<Address, i64>>, // Key: DID, Value: Delegates list <Key: Delegate Address, Value: Expire At>
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
    let delegate_storage: SortedVecMap<String, SortedVecMap<Address, i64>> = SortedVecMap::new();

    
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
        delegates: delegate_storage,
    };

    state
}

#[action(shortname = 0x01)]
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

#[action(shortname = 0x02)]
pub fn set_attribute(
    context: ContractContext,
    mut state: ContractState,
    new_attributes: Vec<String>,
    did: String,
) -> ContractState{

    if state.dids.contains_key(&did) {

        let controller = state.dids.get(&did).unwrap().clone();
        // Do nothing if Sender is the Controller
        if controller == context.sender {
            // Empty block
        // Sender not the Controller, check if the DID has Delegates
        } else if state.delegates.contains_key(&did) {
            let delegates_map = state.delegates.get(&did).unwrap();
            // Check if the Sender is one of the Delegates
            if delegates_map.contains_key(&context.sender) {
                // Check if the Delelgate has expired
                if delegates_map.get(&context.sender).unwrap().clone() < context.block_time {
                    panic!("Delegate Expired!")
                }
            } else {
                panic!("Not Authorized!");
            }
        // DID has no Delegates
        } else {
            panic!("Not Authorized!");
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

#[action(shortname = 0x03)]
pub fn change_owner(
    context: ContractContext,
    mut state: ContractState,
    new_owner: Address,
    did: String,
) -> ContractState{
    if state.dids.contains_key(&did) {
        let controller = state.dids.get(&did).unwrap().clone();
        // Should we allow Delegate to change the DID owner?
        if controller != context.sender {
            panic!("Not Authorized!")
        }

        *state.dids.get_mut(&did).unwrap() = new_owner;

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

#[action(shortname = 0x04)]
pub fn add_delegate(
    context: ContractContext,
    mut state: ContractState,
    delegate_address: Address,
    did: String,
    expire_in: i64,
) -> ContractState{
    if state.dids.contains_key(&did) {
        let controller = state.dids.get(&did).unwrap().clone();

        if controller != context.sender {
            panic!("Not Authorized!")
        }

        if state.delegates.contains_key(&did) {
            let delegates_map = state.delegates.get_mut(&did).unwrap();
            if delegates_map.contains_key(&delegate_address) {
                *delegates_map.get_mut(&delegate_address).unwrap()=context.block_time + expire_in;
            } else {
                delegates_map.insert(delegate_address, context.block_time + expire_in);
            }

        } else {
            let mut new_delegates_map : SortedVecMap<Address, i64> = SortedVecMap::new();
            new_delegates_map.insert(delegate_address, context.block_time + expire_in);
            state.delegates.insert(did, new_delegates_map);
        }

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