mod programs;
use crate::programs::wba_prereq::{CompleteArgs, WbaPrereqProgram};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::read_keypair_file, signer::Signer, system_program};
const RPC_URL: &str = "https://api.devnet.solana.com";
fn main() {
    let rpc_client = RpcClient::new(RPC_URL);
    let signer = read_keypair_file("mainnet-wallet.json").expect("Error reading keypair");
    let prereq = WbaPrereqProgram::derive_program_address(&[b"prereq", signer.pubkey().to_bytes().as_ref()]);
    let args = CompleteArgs {
        github: b"ved08".to_vec(),
    };
    let blockhash = rpc_client.get_latest_blockhash().expect("Error getting blockhash");
    let transaction = WbaPrereqProgram::complete(
        &[&signer.pubkey(), &prereq, &system_program::id()], 
        &args,
        Some(&signer.pubkey()),
        &[&signer], 
        blockhash
    );
    let signature = rpc_client.send_and_confirm_transaction(&transaction)
        .expect("Error sending transaction");
    println!("Successfully Registered: {}", signature);
}

#[cfg(test)]
mod tests {
    use solana_sdk::{
        message::Message, native_token::LAMPORTS_PER_SOL, signature::{read_keypair_file, Keypair, Signer}, transaction::Transaction
    };
    use solana_client::rpc_client::RpcClient;
    use solana_program::{
        pubkey::Pubkey,
        system_instruction::transfer
    };
    use bs58;
    use std::{io::{self, BufRead}, str::FromStr};

    use crate::RPC_URL;
    #[test]
    fn base58_to_wallet() {
        println!("Input private key: ");
        let stdin = io::stdin();
        let base58 = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap();
        println!("Your wallet file is: ");
        let wallet = bs58::decode(base58)
        .into_vec()
        .unwrap();
        println!("{:?}", wallet)
    }
    #[test]
    fn wallet_to_base58() {
        println!("Enter wallet file: ");
        let stdin = io::stdin();
        let wallet = stdin
        .lock()
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .trim_start_matches("[")
        .trim_end_matches("]")
        .split(",")
        .map(|s| s.trim().parse::<u8>().unwrap())
        .collect::<Vec<u8>>();
        let private_key = bs58::encode(wallet).into_string();
        println!("Pkey: {}", private_key);
    }
    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!("New keypair generated: {}", kp.pubkey().to_string());
        println!("{:?}",kp.to_bytes())
    }
    #[test]
    fn airdrop() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Cannot read file");
        let client = RpcClient::new(RPC_URL);
        match client.request_airdrop(&keypair.pubkey(), LAMPORTS_PER_SOL*2) {
            Ok(s) => println!("Signature: {}", s),
            Err(s) => println!("Error: {}", s.to_string())
        };
    }
    #[test]
    fn transfer_sol() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Cannot read file");
        let to_pubkey = Pubkey::from_str("91Q1XdVxobuAjX8vcKj4PruC7KZsaEL3cU5cH61WyDmw").unwrap();
        let rpc_client = RpcClient::new(RPC_URL);
        let recent_blockhash = rpc_client.get_latest_blockhash().expect("Error fetching blockhash");
        // let transaction = Transaction::new_signed_with_payer(
        //     &[transfer(
        //         &keypair.pubkey(), 
        //         &to_pubkey, 
        //         LAMPORTS_PER_SOL
        //     )], Some(&keypair.pubkey()), 
        //     &vec![&keypair], 
        //     recent_blockhash
        // );
        // let signature = rpc_client.send_and_confirm_transaction(&transaction)
        //     .expect("Failed to transfer");
        // println!("Sucess: {}", signature);

        let balance = rpc_client.get_balance(&keypair.pubkey())
            .expect("Failed to get balance");
        let message = Message::new_with_blockhash(
            &[transfer(
                &keypair.pubkey(), 
                &to_pubkey, 
                balance
            )], 
            Some(&keypair.pubkey()), 
            &recent_blockhash
        );
        let fee = rpc_client.get_fee_for_message(&message).expect("Unable to calculate fee");
        let send_amount = balance - fee;
        let transaction = Transaction::new_signed_with_payer(
            &[
                transfer(
                    &keypair.pubkey(), 
                    &to_pubkey, 
                    send_amount
                )
            ], 
            Some(&keypair.pubkey()), 
            &vec![keypair], 
            recent_blockhash
        );
        let signature = rpc_client.send_and_confirm_transaction(&transaction).expect("Error");
        println!("Transferred sol: {}", signature);
    }
}