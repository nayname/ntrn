use std::fmt;

use std::fs::{self, File};
use rig::providers::openai;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use rig::providers::openai::client::Client;

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
/// An enum representing the sentiment of a document
enum ADOs {
    NftMarketplace,
    Crowdfund,
    Cw20Exchange,
    AuctionUsingCw20Tokens,
    ExtendedMarketplace,
    CommissionBasedSales,
    VestingAndStaking,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
/// A record containing extracted sentiment
pub struct ProgContext {
    pub ados_components: Vec<String>,
    pub descr: String,
    pub label: String,
}

impl ProgContext {
    pub fn new(class_def: &Value) -> anyhow::Result<Self> {
        let mut context = ProgContext {
            ados_components: Vec::new(),
            descr: String::new(),
            label: String::new(),
        };

        // Process component names from class definition
        if let Some(component_names) = class_def["classes"].as_array() {
            for name_val in component_names {
                if let Some(name) = name_val.as_str() {
                    let path = format!("static/config/objects/{}.json", name);
                    let comp_schema = fs::read_to_string(path)?;
                    context.ados_components.push(comp_schema);
                }
            }
        }

        // Set description if available
        if let Some(descr) = class_def["descr"].as_str() {
            context.descr = descr.to_string();
        }

        if let Some(label) = class_def["label"].as_str() {
            context.label = label.to_string();
        }

        Ok(context)
    }
}

impl fmt::Display for ProgContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Format components as a comma-separated list
        let components = self.ados_components.join(", ");

        write!(f, "Components: [{}]\nDescription: {}", components, self.descr)
    }
}


#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct AdoType {
    /// The sentiment of the document
    ado: ADOs,
}

#[tokio::main]
pub async fn classify_ado(messages: &Value) -> AdoType {
    // Create OpenAI client
    let openai_client = openai::Client::from_env();

    // Create extractor
    let data_extractor = openai_client
        .extractor::<AdoType>("gpt-4o")
        .build();

    let ado_type = data_extractor
        .extract(messages.to_string())
        .await
        .expect("Failed to extract sentiment");

    println!("GPT-4o: {:?}", ado_type);

    return ado_type
}

pub fn get_classes_config (ado_type: AdoType) -> anyhow::Result<ProgContext>{
    let classes_config: Value = serde_json::from_str(&fs::read_to_string("static/config/config_all.json")?)?;

    Ok(match ado_type.ado {
        ADOs::NftMarketplace => {
            ProgContext::new(classes_config.get("nft_marketplace").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        ADOs::Crowdfund => {
            ProgContext::new(classes_config.get("crowdfund").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        ADOs::Cw20Exchange => {
            ProgContext::new(classes_config.get("cw20_exchange").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        ADOs::AuctionUsingCw20Tokens => {
            ProgContext::new(classes_config.get("auction_using_cw20_tokens").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        ADOs::ExtendedMarketplace => {
            ProgContext::new(classes_config.get("extended_marketplace").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        ADOs::CommissionBasedSales => {
            ProgContext::new(classes_config.get("commission_based_sales").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        ADOs::VestingAndStaking => {
            ProgContext::new(classes_config.get("vesting_and_staking").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        }
    })
}