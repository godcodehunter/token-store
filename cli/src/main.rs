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
use solana_client::{rpc_client::RpcClient};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use token_market::{
    instruction::{
        transaction_initialize,
        transaction_buy_tokens,
    }, 
    state::TokenMarket,
};

struct Config {
    owner: Box<dyn Signer>,
    fee_payer: Box<dyn Signer>,
    rpc_client: RpcClient,
}

fn create_market(config: &Config, mint_of_acceptable: Pubkey) -> Result<()> {
    println!("Creating market...");

    let market = Keypair::new();
    let bank = Keypair::new();
    let emitter = Keypair::new();
    let market_authority = Pubkey::create_program_address(
        &[b"tmarket"], 
        &token_market::id(),
    ).unwrap();

    let ts = transaction_initialize(
        config.rpc_client.get_recent_blockhash()?.0,
        config.fee_payer,
        config.owner,
        &market,
        &market_authority,
        &bank,
        &emitter,
        &mint_of_acceptable,
    );    
    config
        .rpc_client
        .send_and_confirm_transaction_with_spinner(&ts)?;

    println!(
        "Market created: market {}, accepted tokens: {}, tradable tokens: {}, bank: {}",
        market.pubkey(),
        mint_of_acceptable,
        emitter.pubkey(),
        bank.pubkey()
    );
    Ok(())
}

fn buy_tokens(config: &Config, market: Pubkey, recipient: Pubkey, amount: u64) -> Result<()> {
    println!("Buying tokens...");

    let market_data = config.rpc_client.get_account_data(&market)?;
    let token_market = TokenMarket::try_from_slice(market_data.as_slice())?;
    let market_authority = Pubkey::create_program_address(
        &[&market.to_bytes()[..32]], 
        &token_market::id(),
    ).unwrap();

    // Finding a suitable account for placement of purchased tokens.
    // If suitable account is not found - create it.
    let recipient_account = spl_associated_token_account::get_associated_token_address(
        &recipient,
        &token_market.emitter_mint,
    );

    let write_off_account = spl_associated_token_account::get_associated_token_address(
        &config.fee_payer.pubkey(),
        &token_market.mint_of_acceptable,
    );

    let ts = transaction_buy_tokens(
        config.rpc_client.get_recent_blockhash()?.0,
        config.fee_payer,
        config.fee_payer,
        market,
        market_authority,
        token_market.emitter_mint,
        token_market.bank,
        recipient_account,
        write_off_account,
        amount,
    );
    config
        .rpc_client
        .send_and_confirm_transaction_with_spinner(&ts)?;

    println!(
        "Purchased {} tokens. Recipient user {}. Target ATA {}",
        amount, recipient, recipient_account
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
            let mv = args.value_of("amount");
            let amount = value_t!(args.value_of("amount"), u64)
                .expect("Can't parse amount, it is must present like integer");

            buy_tokens(config, market, recipient, amount)
        }
        _ => unreachable!(),
    }
}
