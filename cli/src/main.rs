use anyhow::Result;
use borsh::de::BorshDeserialize;
use clap::{
    crate_description, crate_name, crate_version, value_t, App, AppSettings, Arg, SubCommand,
};
use solana_clap_utils::{
    fee_payer::fee_payer_arg,
    input_parsers::pubkey_of,
    input_validators::{is_pubkey, is_url_or_moniker, is_valid_signer},
    keypair::signer_from_path,
};
use solana_client::{rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    message::Message,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_token::state::{Account, Mint};
use std::str::FromStr;
use token_market::{instruction, state::TokenMarket};

struct Config {
    owner: Box<dyn Signer>,
    fee_payer: Box<dyn Signer>,
    rpc_client: RpcClient,
}

fn create_market(config: &Config, mint_acceptable: Pubkey) -> Result<()> {
    println!("Creating market...");

    let market = Keypair::new();
    let bank = Keypair::new();
    let emitter = Keypair::new();

    let instructions = &[
        create_account(
            &config.fee_payer.pubkey(),
            &market.pubkey(),
            Rent::default().minimum_balance(TokenMarket::LEN),
            TokenMarket::LEN as u64,
            &token_market::id(),
        ),
        create_account(
            &config.fee_payer.pubkey(),
            &bank.pubkey(),
            Rent::default().minimum_balance(Account::LEN),
            Account::LEN as u64,
            &token_market::id(),
        ),
        create_account(
            &config.fee_payer.pubkey(),
            &emitter.pubkey(),
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            &token_market::id(),
        ),
        instruction::initialize(
            &token_market::id(),
            &config.owner.pubkey(),
            &market.pubkey(),
            &bank.pubkey(),
            &emitter.pubkey(),
            &emitter.pubkey(),
            &mint_acceptable,
            &spl_token::id(),
        )?,
    ];

    let mut ts = Transaction::new_with_payer(instructions, Some(&config.fee_payer.pubkey()));
    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    let signers = vec![
        config.fee_payer.as_ref(), 
        config.owner.as_ref(), 
        &market as &dyn Signer,
    ];
    ts.sign(&signers, recent_blockhash);
    config
        .rpc_client
        .send_and_confirm_transaction_with_spinner(&ts)?;

    println!(
        "Market created: market {}, accepted tokens: {}, tradable tokens: {}, bank: {}",
        market.pubkey(),
        mint_acceptable,
        emitter.pubkey(),
        bank.pubkey()
    );
    Ok(())
}

fn buy_tokens(config: &Config, market: Pubkey, recipient: Pubkey, amount: u64) -> Result<()> {
    println!("Buying tokens...");

    let mut instructions = vec![];

    let market_data = config.rpc_client.get_account_data(&market)?;
    let token_market = TokenMarket::try_from_slice(market_data.as_slice())?;

    // Finding a suitable account for placement of purchased tokens.
    // If suitable account is not found - create it.
    let recipient_acc = spl_associated_token_account::get_associated_token_address(
        &recipient,
        &token_market.emitter_mint,
    );

    let write_off_account = spl_associated_token_account::get_associated_token_address(
        &config.owner.pubkey(),
        &token_market.mint_of_acceptable,
    );

    instructions.extend_from_slice(&[
        spl_token::instruction::approve(
            &token_market::id(),
            &recipient,
            &token_market::id(),
            &config.owner.pubkey(),
            &[],
            amount,
        )?,
        instruction::buy_tokens(
            &token_market::id(),
            &market,
            &token_market.bank,
            &recipient_acc,
            &write_off_account,
            &token_market::id(),
            amount,
        )?,
    ]);

    let message = Message::new(instructions.as_slice(), Some(&config.fee_payer.pubkey()));
    let transaction = Transaction::new_unsigned(message);
    config
        .rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)?;

    println!(
        "Purchased {} tokens. Recipient user {}. Target ATA {}",
        amount, recipient, recipient_acc
    );
    Ok(())
}

fn main() -> Result<()> {
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
                .args(&[Arg::with_name("acceptable")
                    .value_name("ADDRESS")
                    .takes_value(true)
                    .validator(is_pubkey)
                    .required(true)
                    .help("TODO")])
                .about("Create a new token market"),
        )
        .subcommand(
            SubCommand::with_name("buy-tokens").args(&[
                Arg::with_name("market")
                    .validator(is_pubkey)
                    .value_name("MARKET_ADDRESS")
                    .takes_value(true)
                    .required(true)
                    .help("Market account pubkey"),
                Arg::with_name("recipient")
                    .validator(is_pubkey)
                    .value_name("ACCOUNT_ADDRESS")
                    .takes_value(true)
                    .required(true)
                    .help("User which tokens are received"),
                Arg::with_name("amount")
                    .value_name("NUMBER")
                    .takes_value(true)
                    .required(true)
                    .help("Number of exchanged tokens"),
            ]),
        )
        .get_matches();

    let mut wallet_manager = None;

    let cli_config = if let Some(config_file) = matches.value_of("config_file") {
        solana_cli_config::Config::load(config_file)?
    } else {
        println!("Config file not provided and default config unexist. Create config");
        solana_cli_config::Config::default()
    };
    let json_rpc_url = value_t!(matches, "json_rpc_url", String)
        .unwrap_or_else(|_| cli_config.json_rpc_url.clone());
    let owner = signer_from_path(
        &matches,
        matches
            .value_of("owner")
            .unwrap_or(&cli_config.keypair_path),
        "owner",
        &mut wallet_manager,
    )
    .unwrap(); //TODO
    let fee_payer = signer_from_path(
        &matches,
        matches
            .value_of("fee_payer")
            .unwrap_or(&cli_config.keypair_path),
        "fee_payer",
        &mut wallet_manager,
    )
    .unwrap(); //TODO

    let config = &Config {
        owner: owner,
        fee_payer: fee_payer,
        rpc_client: RpcClient::new_with_commitment(json_rpc_url, CommitmentConfig::confirmed()),
    };

    solana_logger::setup_with_default("solana=info");

    match matches.subcommand() {
        ("create-market", Some(args)) => {
            let acceptable = pubkey_of(args, "acceptable").unwrap();

            create_market(config, acceptable)
        }
        ("buy-tokens", Some(args)) => {
            let market = pubkey_of(args, "market").unwrap();
            let recipient = pubkey_of(args, "recipient").unwrap();
            let mv = matches.value_of("amount");
            dbg!(market, recipient, mv);
            let amount = value_t!(matches.value_of("amount"), u64)
                .expect("Can't parse amount, it is must present like integer");

            buy_tokens(config, market, recipient, amount)
        }
        _ => unreachable!(),
    }
}
