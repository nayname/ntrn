use anyhow::{Context, Result};
use cosmwasm_std::{to_binary, Addr};
use cosmwasm_std::Binary;
// Pull in InstantiateMsg definitions for every ADO you support.
use andromeda_non_fungible_tokens::{
    auction::InstantiateMsg       as AuctionMsg,
    crowdfund::InstantiateMsg     as CrowdfundMsg,
    // marketplace::InstantiateMsg as MarketplaceMsg, // add more as needed
};

use andromeda_fungible_tokens::cw20_exchange::InstantiateMsg as ExchangeMsg;

/// Convenience enum so one `serde_json::from_str` handles *all* ADO kinds.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(tag = "ado_type", rename_all = "snake_case")]
pub enum AnyInstantiateMsg {
    Auction(AuctionMsg),
    Crowdfund(CrowdfundMsg),
    Cw20Exchange(ExchangeMsg),
    // … extend with other ADOs
}

/// Convert an ADO JSON spec into a `MsgInstantiateContract` ready for broadcast.
///
/// * `json` - raw JSON string emitted by your generator.
/// * `code_id` – Code‑ID for the ADO on the target chain (query ADODB or cache).
/// * `sender` – address that will pay the instantiate fees.
#[allow(clippy::too_many_arguments)]
pub fn json_to_cosmwasm(
    msg: &AnyInstantiateMsg,
    code_id: u64,
    // sender: &Addr,
    label: &str,
) -> Result<cosmos_sdk_proto::cosmwasm::wasm::v1::MsgInstantiateContract> {

    Ok(cosmos_sdk_proto::cosmwasm::wasm::v1::MsgInstantiateContract {
        sender: "".into(),//sender.to_string(),
        admin: "".into(),
        code_id,
        label: label.into(),
        msg: to_binary(&msg)?.into(),
        funds: vec![],
    })
}