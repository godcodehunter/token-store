use clap::{crate_description, value_t, crate_name, crate_version, App, AppSettings, Arg, SubCommand};
use solana_client::rpc_client::RpcClient;
use solana_clap_utils::{
    fee_payer::fee_payer_arg,
    input_parsers::{pubkey_of, value_of},
    input_validators::{
        is_url_or_moniker,
        is_valid_signer,
        is_pubkey
    }
};
use token_market::instruction;
use solana_sdk::pubkey::Pubkey;

struct Config {
    rpc_client: RpcClient
}

// fn create_market() {
//     println!("Creating market...");

//     let mut ts = Transaction::new_with_payer(
//         &[
//             system_instruction::create_account(
//                 &config.fee_payer.pubkey(),
//                 &bank.pubkey(),
//                 bank_balance,
//                 token::state::Account::LEN as u64,
//                 &spl_token::id(),
//             ),
//             instruction::initialize(
//                 program_id, 
//                 market_account, 
//                 rent
//             ),
//         ],
//         Some(&config.fee_payer.pubkey())
//     );
// }

fn buy_tokens(market: Pubkey, recipient: Pubkey, amount: u64) {
    
    instruction::buy_tokens(
        todo!(),
        &market,
        todo!(),
        &recipient,
        amount
    );
}

fn main() {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(&config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("json_rpc_url")
                .short("u")
                .long("url")
                .value_name("URL_OR_MONIKER")
                .takes_value(true)
                .global(true)
                .validator(is_url_or_moniker)
                .help(
                    "URL for Solana's JSON RPC or moniker (or their first letter): \
                       [mainnet-beta, testnet, devnet, localhost] \
                    Default from the configuration file.",
                ),
        )
        .arg(
            Arg::with_name("owner")
            .long("owner")
            .value_name("KEYPAIR")
            .validator(is_valid_signer)
            .takes_value(true)
            .global(true)
            .help(
                "Specify the token owner account. \
                 This may be a keypair file, the ASK keyword. \
                 Defaults to the client keypair.",
            ),
        )
        .arg(fee_payer_arg().global(true))
        .subcommand(
            SubCommand::with_name("create-market")
            .about("Create a new token market")
        )
        .subcommand(
            SubCommand::with_name("buy-tokens")
            .args(&[
                Arg::with_name("market")
                    .validator(is_pubkey)
                    .value_name("ADDRESS")
                    .takes_value(true)
                    .required(true)
                    .help("Market account pubkey"),
                Arg::with_name("recipient")
                    .validator(is_pubkey)
                    .value_name("ADDRESS")
                    .takes_value(true)
                    .required(true)
                    .help("Associated token account to which tokens are received"),
                Arg::with_name("amount")
                    .value_name("NUMBER")
                    .takes_value(true)
                    .required(true)
                    .help("Number of exchanged tokens")
            ])
        )
        .get_matches();
    
    

    // Config {
    //     rpc_client: RpcClient::new_with_commitment(json_rpc_url, CommitmentConfig::confirmed())
    // };
    
    solana_logger::setup_with_default("solana=info");

    match matches.subcommand() {
        ("create-market", Some(args)) => {
            // create_market();
        },
        ("buy-tokens", Some(args)) => {
            let market = pubkey_of(args, "market").unwrap();
            let recipient = pubkey_of(args, "recipient").unwrap();
            let amount = value_t!(matches.value_of("amount"), u64).expect("Can't parse amount, it is must present like integer");
            buy_tokens(market, recipient, amount)
        },
        _ => unreachable!(),
    }
}
