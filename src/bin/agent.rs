use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use anyhow::Context;

use rand::Rng;
use serde_json::{json, Value};

use iamy::{json_to_cosmwasm, AnyInstantiateMsg};
use iamy::{get_context, ask_gpt};
use iamy::{get_classes_config, classify_ado, ProgContext};

/// List of supported ADO operation classes.
const OPERATIONS: [&str; 7] = [
    "nft_marketplace",
    "crowdfund",
    "cw20_exchange",
    "auction_using_cw20_tokens",
    "extended_marketplace",
    "commission_based_sales",
    "vesting_and_staking",
];

/// Prompt template for query‐classification calls to the LLM.
const CLASSIFY_QUERY: &str = "Lets pretend that we have an LLM app that generates Andromeda Protocol app contracts \
 using user promtps in natural language. You will be given a user's promt. Based on the context, classify the query \
 to one of the following classes. Classes: ***OPERATIONS***. User's query: ***QUERY***";

/// Prompt template for schema‑generation calls to the LLM.
const GENERATE_FLEX: &str = "You will be given a description of the modules and the schema of the modules. Based on this context and the \
 user's query, generate the schema that fulfills the users intent. User's query: ***QUERY***";

//---------------------------------------------------------------------
// HELPER UTILITIES
//---------------------------------------------------------------------

/// Extract parameters from a free‑form query.  (Not yet implemented.)
fn parse_query(_query: &str) -> Value {
    Value::Null
}

/// Convert query parameters back to a prompt context string.  (Not yet implemented.)
fn context_from_params(_params: &Value) -> String {
    String::new()
}

/// Persist the raw LLM answer for later inspection
fn validate(class_: &str, answer: &str) -> std::io::Result<()> {
    let mut rng = rand::rng();
    let hash: u128 = rng.random();

    fs::create_dir_all("generated")?;
    let filename = format!("generated/{}_{}", hash, class_);
    let mut file = File::create(filename)?;

    file.write_all(answer.as_bytes())?;
    Ok(())
}

//---------------------------------------------------------------------
// CORE GENERATION LOGIC 
//---------------------------------------------------------------------

fn generate(query: &str, map: &mut Vec<Value>) -> anyhow::Result<()> {
    // 1. Read global context blob from disk.
    let context = fs::read_to_string("static/context.json")?;

    // 2. CLASSIFICATION STAGE ------------------------------------------------
    let messages = get_context(
        &CLASSIFY_QUERY
            .replace("***OPERATIONS***", &serde_json::to_string(&OPERATIONS)?)
            .replace("***QUERY***", query),
        &context,
    );

    let prog_context = get_classes_config(classify_ado(&messages))
        .expect("Failed to get classes config");

    // 3. CONFIGURATION ASSEMBLY ---------------------------------------------
    let gen_messages = get_context(
        &GENERATE_FLEX.replace("***QUERY***", query),
        &prog_context.to_string(),
    );

    // 4. GENERATION STAGE ----------------------------------------------------
    let answer_gen = ask_gpt(&gen_messages);
    let answer_str = answer_gen.unwrap_or_else(|err| format!("Error: {}", err));

    validate(&prog_context.label, &answer_str)?;

    // 5. UPDATE UI MAP -------------------------------------------------------
    let mut rng = rand::rng();
    let hash: u128 = rng.random();
    map.push(json!({
        "name": format!("{}_{}", hash, &prog_context.label),
        "query": query,
        "label": &prog_context.label,
    }));
    fs::write("generated_map.json", serde_json::to_string_pretty(&map)?)?;

    // 6. JSON TO COSMWASM -----------------------------------------------------
    // let args = Args::parse();
    // let json = fs::read_to_string(&args.spec_path).await?;
    // let sender = Addr::unchecked(args.sender.or_else(|| std::env::var("SENDER_ADDR").ok()).expect("sender address required"));

    let msg: AnyInstantiateMsg = serde_json::from_str(&answer_str)
        .context("invalid ADO JSON – cannot deserialize into InstantiateMsg")?;
    let msg = json_to_cosmwasm(&msg, 0, "")?;
    Ok(())
}

//---------------------------------------------------------------------
// ENTRY‑POINT
//---------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    // Load the synthetic query set.
    let queries: Value = serde_json::from_str(&fs::read_to_string("static/queries.json")?)?;

    // Load or initialise the generated‑script index.
    let mut map: Vec<Value> = if Path::new("generated_map.json").exists() {
        serde_json::from_str(&fs::read_to_string("generated_map.json")?)?
    } else {
        Vec::new()
    };

    // Dispatch sample generations (mirrors Python order).
    generate(queries["nft_marketplace"][0].as_str().unwrap(), &mut map)?;
    generate(queries["crowdfund"][0].as_str().unwrap(), &mut map)?;
    generate(queries["cw20_exchange"][0].as_str().unwrap(), &mut map)?;
    generate(queries["auction_using_cw20_tokens"][0].as_str().unwrap(), &mut map)?;
    generate(queries["extended_marketplace"][0].as_str().unwrap(), &mut map)?;
    generate(queries["commission_based_sales"][0].as_str().unwrap(), &mut map)?;
    generate(queries["vesting_and_staking"][0].as_str().unwrap(), &mut map)?;
    generate(queries["extended_marketplace"][1].as_str().unwrap(), &mut map)?;
    generate(queries["cw20_exchange"][1].as_str().unwrap(), &mut map)?;
    generate(queries["vesting_and_staking"][1].as_str().unwrap(), &mut map)?;

    Ok(())
}

//---------------------------------------------------------------------
// DEPENDENCY HINTS (add to Cargo.toml)
//---------------------------------------------------------------------
// [dependencies]
// anyhow = "1"
// rand = "0.8"
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
