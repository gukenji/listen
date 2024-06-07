use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::execute::Executor;
use crate::http_client::HttpClient;
use crate::util::healthz;
use crate::{
    buyer,
    provider::Provider,
    util::{env, pubkey_to_string, string_to_pubkey},
};
use crate::{constants, jito, seller};
use actix_web::web::{self, Json};
use actix_web::{get, post};
use actix_web::{App, Error, HttpResponse, HttpServer};
use futures_util::StreamExt;
use jito_searcher_client::get_searcher_client;
use log::{info, warn};
use raydium_library::amm;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{EncodableKey, Signer};
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize)]
pub struct SellRequest {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub amm_pool: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub input_mint: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub output_mint: Pubkey,
    pub lamports_spent: u64,
    pub insta: Option<bool>,
}

#[post("/sell")]
async fn handle_sell(sell_request: Json<SellRequest>) -> Result<HttpResponse, Error> {
    info!(
        "handling sell_request {}",
        serde_json::to_string_pretty(&sell_request)?
    );
    actix_rt::spawn(async move {
        let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).expect("read wallet");
        let provider = Provider::new(env("RPC_URL"));
        let token_account = spl_associated_token_account::get_associated_token_address(
            &wallet.pubkey(),
            &sell_request.input_mint,
        );
        let pubsub_client = PubsubClient::new(&env("WS_URL"))
            .await
            .expect("make pubsub client");
        let balance = tokio::select! {
                balance = seller::get_spl_balance_stream(&pubsub_client, &token_account) => balance.expect("balance stream"),
                balance = seller::get_spl_balance(&provider, &token_account) => balance.expect("balance rpc"),
        };
        // TODO generally, those params should be different for pump.fun coins and
        // the standard coins
        // --
        // number one thing now would be to analyze after looking at some charts
        // rn I think the crucial thing is to get rid of the rugs where someone
        // even though all checks pass, some holder dumps $XXK and -99.9%s the token
        if !sell_request.insta.unwrap_or(false) {
            // load amm keys
            let amm_program =
                Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY).expect("amm program");
            let amm_keys = amm::utils::load_amm_keys(
                &provider.rpc_client,
                &amm_program,
                &sell_request.amm_pool,
            )
            .await
            .expect("amm_keys");
            let executor = Executor {
                amm_keys,
                funder: wallet,
                lamports_in: sell_request.lamports_spent,
                token_balance: balance,

                sl_levels: vec![0.7, 0.5],
                sl_amounts: vec![0.5, 0.5].iter().map(|x| *x * balance as f64).collect(),
                sl_reached: vec![false, false],

                tp_levels: vec![1.5, 2.0, 3.0, 5.0, 10.0],
                tp_amounts: vec![0.4, 0.2, 0.2, 0.2, 0.2].iter().map(|x| *x * balance as f64).collect(),
                tp_reached: vec![false, false, false, false, false],
            };
            executor
                .execute(&provider, &pubsub_client, &sell_request.amm_pool)
                .await
                .expect("execute");
        } else {
            info!("balance: {}", balance);
            if balance == 0 {
                warn!("could not fetch balance, exiting");
                return;
            }
            buyer::swap(
                &sell_request.amm_pool,
                &sell_request.input_mint,
                &sell_request.output_mint,
                balance, // sell initial and leave the remainder
                &wallet,
                &provider,
            )
            .await
            .expect("swap");
        }
    });

    Ok(HttpResponse::Ok().json(json!({"status": "OK, triggered sell"})))
}

#[derive(Deserialize, Serialize)]
pub struct SimpleSellRequest {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub amm_pool: Pubkey,
}

#[post("/sell-simple")]
async fn handle_sell_simple(sell_request: Json<SimpleSellRequest>) -> Result<HttpResponse, Error> {
    info!(
        "handling simple_sell_request {}",
        serde_json::to_string_pretty(&sell_request)?
    );
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let amm_program = Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY).unwrap();
    let amm_keys = amm::utils::load_amm_keys(&rpc_client, &amm_program, &sell_request.amm_pool)
        .await
        .expect("amm_keys");

    let (input_mint, output_mint) =
        if amm_keys.amm_pc_mint.to_string() == constants::SOLANA_PROGRAM_ID {
            (amm_keys.amm_coin_mint, amm_keys.amm_pc_mint)
        } else {
            (amm_keys.amm_pc_mint, amm_keys.amm_coin_mint)
        };

    HttpClient::new()
        .sell(&SellRequest {
            amm_pool: sell_request.amm_pool,
            input_mint,
            output_mint,
            lamports_spent: 0u64,
            insta: Some(true),
        })
        .await
        .expect("sell");

    Ok(HttpResponse::Ok().json(json!({"status": "OK"})))
}

#[derive(Debug, Default, Clone)]
pub struct BalanceContext {
    pub lamports: Arc<RwLock<u64>>,
    pub token_balances: Arc<RwLock<HashMap<String, f64>>>,
}

impl BalanceContext {
    pub async fn track_lamports_balance(&self, funder: &Pubkey) {
        let pubsub_client = PubsubClient::new(&env("WS_URL"))
            .await
            .expect("make pubsub client");
        let (mut stream, unsub) = pubsub_client
            .account_subscribe(
                funder,
                Some(RpcAccountInfoConfig {
                    commitment: Some(CommitmentConfig::processed()),
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                }),
            )
            .await
            .expect("account_subscribe");
        while let Some(log) = stream.next().await {
            *self.lamports.write().await = log.value.lamports;
            println!("{:?}", self.lamports.read().await);
        }
        unsub().await;
    }

    pub async fn track_token_balance(&self, mint: &Pubkey, owner: &Pubkey) {}
}

#[get("/balance")]
pub async fn handle_balance(
    balance_ctx: web::Data<Arc<BalanceContext>>,
) -> Result<HttpResponse, Error> {
    info!("handling balance request");
    let balance = *balance_ctx.lamports.read().await;
    Ok(HttpResponse::Ok().json(json!({"balance": balance})))
}

pub async fn run_seller_service() -> std::io::Result<()> {
    info!("Running seller service on 8081");
    let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).expect("read wallet");
    info!(
        "Subscribing to balance updates for {}",
        wallet.pubkey().to_string()
    );
    let balance_lamports = RpcClient::new(env("RPC_URL"))
        .get_balance_with_commitment(&wallet.pubkey(), CommitmentConfig::confirmed())
        .await
        .expect("get balance")
        .value;
    let balance_ctx = Arc::new(BalanceContext {
        lamports: Arc::new(RwLock::new(balance_lamports)),
        token_balances: Arc::new(RwLock::new(HashMap::<String, f64>::new())),
    });

    let auth = Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).expect("read auth keypair");
    let searcher_client = get_searcher_client(&env("BLOCK_ENGINE_URL"), &Arc::new(auth))
        .await
        .expect("makes searcher client");

    let poll = balance_ctx.clone();
    tokio::spawn(async move {
        poll.track_lamports_balance(&wallet.pubkey()).await;
    });
    HttpServer::new(move || {
        App::new()
            .service(handle_sell)
            .service(handle_sell_simple)
            .service(handle_balance)
            .app_data(web::Data::new(balance_ctx.clone()))
            .app_data(web::Data::new(searcher_client.clone()))
            .service(healthz)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
