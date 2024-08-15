use std::str::FromStr;

use crate::coin_helpers::{assert_sent_sufficient_coin, generate_nft_class_id, generate_nft_id};
use crate::contract::FEE_DENOM;
use crate::error::ContractError;
use crate::msg::GetSubscriptionInfoResponse;
use crate::state::{
    config_storage_read, did_storage_read, profile_storage, profile_storage_read, single_subscription_storage, single_subscription_storage_read, wallet_storage_read, SubscriptionInfo
};
use coreum_wasm_sdk::assetnft;
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};
use cosmwasm_std::{
    coin, coins, from_json, to_json_binary, BankMsg, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, Uint64
};


// hash function used to map an arbitrary length string (i.e. DID) into a base64 string of length n
// needed for using DID as part of the NFT id
pub fn hash_did(s: &str, n: usize) -> String {
    let mut ret = base64::encode(s);

    // match the string to this regex before returning it: ^[a-zA-Z][a-zA-Z0-9/:._-]{2,100}$
    // replacing all characters which don't match with an empty string
    // and keep the length of the string to n
    
    ret = ret.chars().filter(|c| c.is_alphanumeric() || *c == '/' || *c == ':' || *c == '.' || *c == '_' || *c == '-').collect();

    // return the first n characters
    ret.chars().take(n).collect()
}

pub fn subscribe(
    deps: DepsMut<CoreumQueries>,
    env: Env,
    info: MessageInfo,
    subscribe_to_did: String,
) -> Result<Response<CoreumMsg>, ContractError> {
    // check if request has sufficient amount of coins attached
    let price: Coin = from_json(get_subscription_price(
        deps.as_ref(),
        subscribe_to_did.clone(),
    )?)?;
    deps.api
        .debug(format!("sub price: {}", price.amount).as_str());
    assert_sent_sufficient_coin(&info.funds, Some(price.clone()))?;

    let mut response = Response::default();

    // compute the refund
    if price.amount != Uint128::zero() {
        // returns error if 0 or 2+ coins are sent
        let mut coins_sent: Option<&Coin> = None;
        if !info.funds.is_empty() && info.funds.len() < 2 {
            coins_sent = Some(info.funds.get(0).unwrap())
        };

        if coins_sent.is_none() {
            return Err(ContractError::InsufficientFundsSend {
                sent: Some(coin(0u128, price.denom.clone())),
                expected: Some(price.clone()),
            });
        } else {
            // if yes, refund the surplus
            let refund_amount = coins_sent
                .unwrap()
                .amount
                .checked_sub(price.amount)
                .map_err(|_| StdError::generic_err("Overflow error"))?;
            let mut refund = Coin {
                denom: price.denom.clone(),
                amount: refund_amount,
            };

            if refund.amount.is_zero() {
                refund = Coin {
                    denom: FEE_DENOM.to_owned(),
                    amount: 0u128.into(),
                };
            }

            response = response.add_attribute("refund", refund.to_string());
        }
    }

    // convert the sender wallet to DID
    let subscriber_did = wallet_storage_read(deps.storage)
        .may_load(info.sender.as_bytes())?
        .ok_or(StdError::generic_err(format!("Couldn't find the subscriber's DID for wallet {} in the contract registry", info.sender)))?;

    // convert the subscribed_to DID into a wallet
    let subscribed_to_wallet = did_storage_read(deps.storage)
        .may_load(subscribe_to_did.clone().as_bytes())?
        .ok_or(StdError::generic_err(format!("Couldn't fetch the wallet info from the registry for the DID subscribing to: {}", subscribe_to_did)))?;

    // make sure there's no existing valid subscription
    let subscription_key = subscriber_did.did.clone() + subscribe_to_did.as_str();
    let existing_sub =
        single_subscription_storage_read(deps.storage).load(subscription_key.as_bytes());
    if existing_sub.is_ok() {
        let existing_subscription = existing_sub.unwrap();
        if existing_subscription.valid_until.seconds() > env.block.time.seconds() {
            return Err(ContractError::ExistingSubscriptionFound {
                subscription_info: existing_subscription,
            });
        }
    }

    // get the profile info for the subscribed_to DID
    let mut profile_info = profile_storage_read(deps.storage)
        .may_load(subscribe_to_did.as_bytes())?
        .unwrap();

    // store a single subscription info
    let subscription = SubscriptionInfo {
        subscriber: subscriber_did.did.clone(),
        subscribed_to: subscribe_to_did.clone(),
        valid_until: env
            .block
            .time
            .plus_days(profile_info.subscription_duration_days.unwrap().u64()),
        cost: price.clone(),
    };

    single_subscription_storage(deps.storage).save(subscription_key.as_ref(), &subscription)?;

    // increment the subscription counter for the profile
    profile_info.subscriber_count += Uint64::from(1u64);
    profile_storage(deps.storage)
        .save(subscribe_to_did.as_bytes(), &profile_info)
        .expect("Error incrementing subscriber count");

    // mint the subscription NFT
    let nft_class_id = generate_nft_class_id(env.clone(), subscribed_to_wallet.wallet.to_string().clone());
    let nft_id = generate_nft_id(env.clone(), subscriber_did.clone().did, subscribed_to_wallet.wallet.to_string());
    let valid_until = env.block.time.plus_days(profile_info.subscription_duration_days.unwrap().u64());
    response = response.add_attribute("nft_id", nft_id.to_string());
    let mint_res = mint_nft(&deps, info.clone(), nft_class_id, nft_id, Some(to_json_binary(&valid_until).unwrap()));
    if mint_res.is_err() {
        return Err(ContractError::SubscriptionNFTMintingError {});
    } else {
        let sub_msg = mint_res.unwrap();
        deps.api.debug(format!("MINT NFT messages: {:?}", sub_msg).as_str());
        response = response.add_submessages(sub_msg.messages).add_events(sub_msg.events).add_attributes(sub_msg.attributes);
    }
    
    response = response
        .add_attribute("action", "subscribe")
        .add_attribute("subscribed_to_did", subscribed_to_wallet.did.clone())
        .add_attribute("subscribed_to_wallet", subscribed_to_wallet.wallet.clone())
        .add_attribute("valid_until", valid_until.to_string())
        .add_attribute("subscriber", info.sender);

    // payout
    // deps.api.debug("Trying to pay the subscriber...");
    let contact_config = config_storage_read(deps.storage).load().unwrap();
    let sub_fee_percentage = contact_config.subscription_fee;

    let cored_in_commission = 
        sub_fee_percentage.checked_mul(Decimal::from_str(price.amount.to_string().as_str())?).unwrap().floor().to_uint_floor();
    let owner_payout = price.amount.checked_sub(cored_in_commission).unwrap();
    // deps.api.debug(format!("Subscriber payment {}, cored.in commission {}", owner_payout, cored_in_commission).as_str());

    if owner_payout != Uint128::zero() {
        // deps.api.debug(format!("Trying to pay {}...", subscribed_to_wallet.wallet.to_string()).as_str());
        let pay_owner_msg = BankMsg::Send {
            to_address: subscribed_to_wallet.wallet.to_string(),
            amount: coins(owner_payout.u128(), FEE_DENOM),
        };

        response = response.add_message(pay_owner_msg)
    }


    Ok(response)
}

fn mint_nft(
    deps: &DepsMut<CoreumQueries>,
    info: MessageInfo,
    class_id: String,
    nft_id: String,
    // recipient_wallet: String, // if we need to mint into someone else's wallet in the future...
    data: Option<Binary>,
) -> Result<Response<CoreumMsg>, ContractError> {
    let msg = CoreumMsg::AssetNFT(assetnft::Msg::Mint {
        class_id: class_id.clone(),
        id: nft_id.clone(),
        uri: None,
        uri_hash: None,
        data: data.clone(),
        recipient: Some(info.sender.to_string()),
    });

    deps.api.debug(nft_id.as_str());
    println!("{}", nft_id);

    Ok(Response::new()
        .add_attribute("method", "mint_nft")
        .add_attribute("class_id", class_id)
        .add_attribute("nft_id", nft_id)
        .add_attribute("nft_data", data.unwrap_or_default().to_string())
        .add_message(msg))
}

pub fn is_subscriber(deps: Deps<CoreumQueries>, env: Env, target_did: String, subscriber_wallet: String) -> StdResult<Binary> {

    // convert the sender wallet to DID
    let subscriber_profile = wallet_storage_read(deps.storage)
        .may_load(subscriber_wallet.as_bytes())?
        .ok_or(StdError::generic_err(format!("Couldn't find the subscriber's DID for wallet {} in the contract registry", subscriber_wallet)))?;

    // old way, using internal contract storage
    let key = format!("{}{}", subscriber_profile.did, target_did); // Concatenate strings and store in a variable
    let subscriber_info =
        single_subscription_storage_read(deps.storage).may_load(key.as_bytes())?;

    // rely on chain internal state instead of the NFT data until minting is fixed
    let response = match subscriber_info.clone() {
        None => false,
        Some(sub_info) => sub_info.valid_until.seconds() >= env.block.time.seconds(),
    };
    return to_json_binary(&response);

    // the new way - check NFT ownership
    // doc ref: https://github.com/CoreumFoundation/coreum-wasm-sdk/blob/main/src/nft.rs
    // issued NFTs will have IDs of the form {contract_address}-{profile_did}-{subscriber_did}
    // let class_id = generate_nft_class_id(env.clone(), target_did.clone());
    // let nft_id = generate_nft_id(env, subscriber_profile.did, target_did);
    // let request: QueryRequest<CoreumQueries> = CoreumQueries::NFT(nft::Query::Owner {
    //     class_id,
    //     id: nft_id,
    // }).into();
    
    // let res = deps.querier.query::<OwnerResponse>(&request);
    // ^------- this fails with: 
    // thread 'tests::subscription::tests::subscribe_mints_nft' panicked at src/tests/subscription.rs:664:84:
    // called `Result::unwrap()` on an `Err` value: 
    //      QueryError { msg: "Error parsing into type coreum_wasm_sdk::nft::OwnerResponse: missing field `owner`: query wasm contract failed" }

    // in the intent to find out how does the response look like before deserializing it, I tried this:
    // let request_json = match to_json_binary(&request) {
    //     Ok(json) => json,
    //     Err(err) => return Err(err.into()),
    // };
    // let res = deps.querier.raw_query(&request_json);
    // Ok(to_json_binary(&res)?)

    // match res {
    //     Err(e) => return Err(e),
    //     Ok(_nft) => {
    //         // consider the subscriber a subscriber if they own the NFT
    //         to_json_binary(&true)
    //         // TODO: NFT found, verify expiration date
    //     }
    // }
}

pub fn get_subscription_info(
    deps: Deps<CoreumQueries>,
    _env: Env,
    did: String,
    subscriber: String,
) -> StdResult<Binary> {
    let key = format!("{}{}", subscriber, did); // Concatenate strings and store in a variable
    let subscriber_info =
        match single_subscription_storage_read(deps.storage).may_load(key.as_bytes()) {
            Ok(info) => info,
            Err(_) => None,
        };

    let info_response: GetSubscriptionInfoResponse = GetSubscriptionInfoResponse {
        info: subscriber_info,
    };

    return to_json_binary(&info_response);
}

// gets current subscription cost
pub fn get_subscription_price(deps: Deps<CoreumQueries>, did: String) -> StdResult<Binary> {
    let profile_info = profile_storage_read(deps.storage).load(did.as_bytes());

    match profile_info {
        Err(e) => return Err(e),
        Ok(profile) => {
            let mut subscriber_price: Option<Coin> = profile.subscription_price;
            if subscriber_price.is_some() {
                let mut scaled_price = subscriber_price.unwrap();
                scaled_price.amount *= Uint128::from(profile.subscriber_count + Uint64::one());
                subscriber_price = Some(scaled_price);
            }

            return to_json_binary(&subscriber_price);
        }
    }
}

pub fn get_subscription_duration(deps: Deps<CoreumQueries>, did: String) -> StdResult<Binary> {
    let profile_info = profile_storage_read(deps.storage).load(did.as_bytes());

    match profile_info {
        Err(e) => return Err(e),
        Ok(profile) => {
            let subscription_duration_days = profile.subscription_duration_days;

            return to_json_binary(&subscription_duration_days);
        }
    }
}

pub fn set_subscription(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    price: Coin,
    duration: Uint64,
) -> Result<Response<CoreumMsg>, ContractError> {
    let did_info = wallet_storage_read(deps.storage)
        .may_load(info.sender.as_bytes())
        .expect("No DID registerred with the current wallet");
    println!("Set sub price: {:?}", did_info);
    let did_info_unwrapped = did_info.unwrap();
    println!("Set sub price UNWRAPPED: {:?}", did_info_unwrapped);

    let mut profile_info = profile_storage_read(deps.storage)
        .may_load(did_info_unwrapped.did.as_bytes())
        .expect("Profile info for DID not found")
        .expect("Stored profile info is empty");
    profile_info.subscription_price = Some(price.clone());
    profile_info.subscription_duration_days = Some(duration.clone());
    profile_storage(deps.storage)
        .save(did_info_unwrapped.did.as_bytes(), &profile_info)
        .expect("Error storing the new subscription price");

    Ok(Response::<CoreumMsg>::default())
}
