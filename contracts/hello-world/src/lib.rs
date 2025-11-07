#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address,String, Symbol, symbol_short};

// Structure to store remittance transaction details
#[contracttype]
#[derive(Clone)]
pub struct Remittance {
    pub sender: Address,
    pub recipient: Address,
    pub amount: i128,
    pub currency: Symbol,
    pub timestamp: u64,
    pub status: Symbol, // "PENDING", "COMPLETED", "FAILED"
    pub tx_id: u64,
}

// Symbol for tracking total transaction count
const TX_COUNT: Symbol = symbol_short!("TX_COUNT");

// Enum for mapping transaction ID to remittance details
#[contracttype]
pub enum RemittanceBook {
    Transaction(u64),
}

#[contract]
pub struct RemittanceContract;

#[contractimpl]
impl RemittanceContract {
    
    // Function to initiate a cross-border remittance transaction
    pub fn send_remittance(
        env: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        currency: Symbol,
    ) -> u64 {
        // Verify sender authorization
        sender.require_auth();
        
        // Validate amount
        if amount <= 0 {
            log!(&env, "Amount must be greater than zero");
            panic!("Invalid amount");
        }
        
        // Get and increment transaction counter
        let mut tx_count: u64 = env.storage().instance().get(&TX_COUNT).unwrap_or(0);
        tx_count += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create remittance record
        let remittance = Remittance {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount,
            currency: currency.clone(),
            timestamp,
            status: symbol_short!("PENDING"),
            tx_id: tx_count,
        };
        
        // Store remittance transaction
        env.storage()
            .instance()
            .set(&RemittanceBook::Transaction(tx_count), &remittance);
        
        // Update transaction count
        env.storage().instance().set(&TX_COUNT, &tx_count);
        
        // Extend storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Remittance created with TX-ID: {}", tx_count);
        
        tx_count
    }
    
    // Function to complete a remittance transaction (called by payment processor/oracle)
    pub fn complete_remittance(env: Env, tx_id: u64, processor: Address) {
        // Verify processor authorization
        processor.require_auth();
        
        // Retrieve transaction
        let mut remittance = Self::get_remittance(env.clone(), tx_id);
        
        // Verify transaction exists and is pending
        if remittance.tx_id == 0 {
            log!(&env, "Transaction not found");
            panic!("Transaction not found");
        }
        
        if remittance.status != symbol_short!("PENDING") {
            log!(&env, "Transaction already processed");
            panic!("Transaction already processed");
        }
        
        // Update status to completed
        remittance.status = symbol_short!("COMPLETE");
        
        // Store updated remittance
        env.storage()
            .instance()
            .set(&RemittanceBook::Transaction(tx_id), &remittance);
        
        // Extend storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Remittance TX-ID: {} completed", tx_id);
    }
    
    // Function to retrieve remittance details by transaction ID
    pub fn get_remittance(env: Env, tx_id: u64) -> Remittance {
        let key = RemittanceBook::Transaction(tx_id);
        
        env.storage().instance().get(&key).unwrap_or(Remittance {
            sender: Address::from_string(&String::from_slice(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            recipient: Address::from_string(&String::from_slice(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            amount: 0,
            currency: symbol_short!("NOTFOUND"),
            timestamp: 0,
            status: symbol_short!("NOTFOUND"),
            tx_id: 0,
        })
    }
    
    // Function to get total number of remittance transactions
    pub fn get_total_transactions(env: Env) -> u64 {
        env.storage().instance().get(&TX_COUNT).unwrap_or(0)
    }
}