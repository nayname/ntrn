use std::fmt;

use std::fs::{self, File};
use rig::providers::openai;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use rig::providers::openai::client::Client;

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
enum NTRNs {
    NeutronInterchainQueries,
    NeutronInterchainTransactions,
    NeutronIBCTransfer,
    NeutronDexModuleGrpc,
    Reflect
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ProgContext {
    pub components: Vec<String>,
    pub descr: String,
    pub label: String,
}

impl ProgContext {
    pub fn new(class_def: &Value) -> anyhow::Result<Self> {
        let mut context = ProgContext {
            components: Vec::new(),
            descr: String::new(),
            label: String::new(),
        };

        // Process component names from class definition
        if let Some(component_names) = class_def["classes"].as_array() {
            for name_val in component_names {
                if let Some(name) = name_val.as_str() {
                    let path = format!("static/config/objects/{}.json", name);
                    let comp_schema = fs::read_to_string(path)?;
                    context.components.push(comp_schema);
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
        let components = self.components.join(", ");

        write!(f, "Components: [{}]\nDescription: {}", components, self.descr)
    }
}


#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ContractType {
    cs: NTRNs,
}

#[tokio::main]
pub async fn classify_contract(messages: &Value) -> ContractType {
    // Create OpenAI client
    let openai_client = openai::Client::from_env();

    // Create extractor
    let data_extractor = openai_client
        .extractor::<ContractType>("gpt-4o")
        .build();

    let c_type = data_extractor
        .extract(messages.to_string())
        .await
        .expect("Failed to extract sentiment");

    println!("GPT-4o: {:?}", c_type);

    return c_type
}

pub fn get_classes_config (c_type: ContractType) -> anyhow::Result<ProgContext>{
    let classes_config: Value = serde_json::from_str(&fs::read_to_string("static/config/config_all.json")?)?;

    Ok(match c_type.cs {
        NTRNs::NeutronDexModuleGrpc => {
            ProgContext::new(classes_config.get("neutron_dex_module_grpc").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        NTRNs::NeutronInterchainQueries => {
            ProgContext::new(classes_config.get("neutron_interchain_queries").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        NTRNs::NeutronIBCTransfer => {
            ProgContext::new(classes_config.get("neutron_ibc_transfer").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        NTRNs::NeutronInterchainTransactions => {
            ProgContext::new(classes_config.get("neutron_interchain_transactions").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        },
        NTRNs::Reflect => {
            ProgContext::new(classes_config.get("reflect").unwrap_or_else(|| panic!(
                "Class definition not found"
            )))?
        }
    })
}
