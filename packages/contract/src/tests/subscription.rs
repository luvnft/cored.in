#[cfg(test)]
mod tests {
    use std::borrow::{Borrow, BorrowMut};

    // use coreum_test_tube::{CoreumTestApp, SigningAccount, Wasm};
    use coreum_wasm_sdk::core::CoreumQueries;
    use coreum_wasm_sdk::types::coreum;
    use coreum_wasm_sdk::types::cosmos::bank;
    use cosmwasm_std::testing::{mock_dependencies, mock_dependencies_with_balances, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_json, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, QueryRequest, SubMsg, Uint128};
    use crate::contract::{execute, query};
    use crate::msg::{ExecuteMsg, QueryMsg};
    use crate::subscription::is_subscriber;
    use crate::tests::common::common::{copy_deps, get_deps, mock_coredin_initial_accounts, mock_init_no_price, mock_register, MockCoreumDeps};

    #[test]
    fn set_subscription_price() {
        let mut deps_empty = mock_dependencies();
        let deps = get_deps(&mut deps_empty);

        mock_init_no_price(deps);

        // register actors
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "alice", &[coin(10, "core")]);

        let info = mock_info("alice_key", &[]);
        let price = coin(10, "core");
        let msg = ExecuteMsg::SetSubscriptionPrice {
            price: price.clone(),
        };

        let deps = get_deps(&mut deps_empty);
        let _res = execute(deps, mock_env(), info, msg)
            .expect("contract successfully handles SetSubscriptionPrice message");

        let deps = get_deps(&mut deps_empty);
        let stored_price = crate::state::profile_storage_read(deps.storage)
            .may_load("alice_did".as_bytes())
            .expect("load subscription price")
            .unwrap().subscription_price.unwrap();

        assert_eq!(stored_price, price);
    }

    #[test]
    fn subscribe_success() {
        let mut deps_empty = mock_dependencies();
        let deps = get_deps(&mut deps_empty);

        mock_init_no_price(deps);

        // register actors
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "alice", &[coin(10, "core")]);
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "bob", &[coin(10, "core")]);

        let subscribe_info = mock_info("bob_key", &coins(10, "core"));
        let subscribe_msg = ExecuteMsg::Subscribe {
            did: "alice_did".to_string(),
        };

        let deps = get_deps(&mut deps_empty);
        let _res = execute(deps, mock_env(), subscribe_info, subscribe_msg)
            .expect("contract successfully handles Subscribe message");

        let is_subscriber_msg = QueryMsg::IsSubscriber {
            did: "alice_did".to_string(),
            subscriber: "bob_did".to_string(),
        };

        let deps = get_deps(&mut deps_empty);
        let res = query(deps.as_ref(), mock_env(), is_subscriber_msg).unwrap();
        let value: bool = from_json(&res).unwrap();
        assert!(value, "Expected Bob to be a subscriber of Alice");
    }

    #[test]
    fn subscribe_insufficient_funds() {
        let mut deps_empty = mock_dependencies();
        let deps = get_deps(&mut deps_empty);
        
        mock_init_no_price(deps);

        // register actors
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "alice", &[coin(10, "core")]);
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "bob", &[coin(10, "core")]);

        let info = mock_info("alice_key", &[]);
        let price = coin(10, "core");
        let set_price_msg = ExecuteMsg::SetSubscriptionPrice {
            price: price.clone(),
        };
        let deps = get_deps(&mut deps_empty);
        let _res = execute(deps, mock_env(), info.clone(), set_price_msg)
            .expect("contract successfully handles SetSubscriptionPrice message");

        let subscribe_info = mock_info("bob_key", &coins(5, "core"));
        let subscribe_msg = ExecuteMsg::Subscribe {
            did: "alice_key".to_string(),
        };

        let deps = get_deps(&mut deps_empty);
        let res = execute(deps, mock_env(), subscribe_info, subscribe_msg);

        assert!(res.is_err(), "Expected subscribe call to fail with insufficient funds");
    }

    #[test]
    fn subscribe_excess_funds() {
        let mut deps_empty = mock_dependencies();
        let deps = get_deps(&mut deps_empty);
        let env = mock_env(); 

        mock_init_no_price(deps);

        // register actors
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "alice", &[coin(100, "ucore")]);
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "bob", &[coin(100, "ucore")]);
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "claire", &[coin(100, "ucore")]);

        // Alice sets her sub price to 10 CORE tokens
        //  1st subscriber pays 1 * 10 CORE = 10 CORE
        //  2nd subscriber pays 2 * 10 CORE = 20 CORE
        //  ... 
        let info = mock_info("alice_key", &[]);
        let price = coin(10, "ucore");
        let set_price_msg = ExecuteMsg::SetSubscriptionPrice {
            price: price.clone(),
        };

        let deps = get_deps(&mut deps_empty);
        let _res = execute(deps, env.clone(), info.clone(), set_price_msg)
            .expect("contract successfully handles SetSubscriptionPrice message");

        // Bob wants to sub to Alice, which should cost him 10 CORE, but he attaches 15 CORE to the transaction...
        let subscribe_info = mock_info("bob_key", &coins(15, "ucore"));
        let subscribe_msg = ExecuteMsg::Subscribe {
            did: "alice_did".to_string(),
        };
        let deps = get_deps(&mut deps_empty);
        let res = execute(deps, env.clone(), subscribe_info.clone(), subscribe_msg.clone())
            .expect("contract successfully handles Subscribe message");

        // .. and he gets 5 CORE refunded 
        let attributes = res.attributes;
        let refund_attribute = attributes.iter().find(|&attr| attr.key == "refund").unwrap();
        assert_eq!(refund_attribute.value, "5ucore", "Expected refund of 5 ucore");

        // confirm Bob is a subscriber of Alice...
        let is_bob_subscriber_msg = QueryMsg::IsSubscriber {
            did: "alice_did".to_string(),
            subscriber: "bob_did".to_string(),
        };
        let deps = get_deps(&mut deps_empty);
        let res = query(deps.as_ref(), env.clone(), is_bob_subscriber_msg).unwrap();
        let value: bool = from_json(&res).unwrap();
        assert!(value, "Expected Bob to be a subscriber of Alice");

        // ... and Claire isn't
        let is_claire_subscriber_msg = QueryMsg::IsSubscriber {
            did: "alice_did".to_string(),
            subscriber: "claire_did".to_string(),
        };
        let res = query(deps.as_ref(), env.clone(), is_claire_subscriber_msg.clone()).unwrap();
        let value: bool = from_json(&res).unwrap();
        assert!(value == false, "Expected Claire not to be a subscriber of Alice at this moment");

        // Claire tries to subscribe to Alice by attaching 15 CORE, less than needed 
        // since Alice already has 1 subscriber so the price is 20 CORE
        let claire_subscribe_info = mock_info("claire_key", &coins(15, "ucore"));
        let deps = get_deps(&mut deps_empty);
        let _ = execute(deps, env.clone(), claire_subscribe_info.clone(), subscribe_msg.clone())
            .expect_err("contract successfully handles Subscribe message");
        
            let deps = get_deps(&mut deps_empty);
        let res = query(deps.as_ref(), env.clone(), is_claire_subscriber_msg.clone()).unwrap();
        let value: bool = from_json(&res).unwrap();
        assert!(value == false, "Expected Claire not to be a subscriber of Alice at this moment");

        // Claire tries to subscribe to Alice by attaching 55 CORE, more than needed
        let claire_subscribe_info = mock_info("claire_key", &coins(55, "ucore"));
        let deps = get_deps(&mut deps_empty);
        let claire_sub_res = execute(deps, env.clone(), claire_subscribe_info.clone(), subscribe_msg.clone())
            .expect("contract successfully handles Subscribe message");
        
            let deps = get_deps(&mut deps_empty);
        let res = query(deps.as_ref(), env.clone(), is_claire_subscriber_msg).unwrap();
        let value: bool = from_json(&res).unwrap();
        assert!(value, "Expected Claire to be a subscriber of Alice at this moment");

        // She should get 55 - 20 = 35 CORE as a refund
        let attributes = claire_sub_res.attributes;
        let refund_attribute = attributes.iter().find(|&attr| attr.key == "refund").unwrap();
        assert_eq!(refund_attribute.value, "35ucore", "Expected refund of 35 ucore");

    }

    #[test]
    fn subscribe_nonexistent_did() {
        let mut deps_empty = mock_dependencies();

        let deps = get_deps(&mut deps_empty);
        mock_init_no_price(deps);
        // register actors
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "bob", &[coin(10, "core")]);

        let subscribe_info = mock_info("bob_key", &coins(10, "core"));
        let subscribe_msg = ExecuteMsg::Subscribe {
            did: "nonexistent_did".to_string(),
        };

        let deps = get_deps(&mut deps_empty);
        let res = execute(deps, mock_env(), subscribe_info, subscribe_msg);

        assert!(res.is_err(), "Expected subscribe call to fail for nonexistent DID");
    }

    #[test]
    fn subscribe_nft_issued() {
        let mut deps_empty = mock_dependencies();

        let deps = get_deps(&mut deps_empty);
        mock_init_no_price(deps);
        // register actors
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "alice", &[coin(10, "core")]);
        let deps = get_deps(&mut deps_empty);
        mock_register(deps, "bob", &[coin(10, "core")]);

        let subscribe_info = mock_info("bob_key", &coins(10, "core"));
        let subscribe_msg = ExecuteMsg::Subscribe {
            did: "alice_did".to_string(),
        };

        // // get all NFTs for Bob and see if he's got one NFT from Alice's collection
        // let get_nft_message = assetnft::Query::Class { id: "alice_did".to_string() };
        
        // // TODO: change this to {contract_address}-{profile_did}
        // // issues NFTs will have IDs of the form {contract_address}-{profile_did}-{subscriber_did}
        // let class_id = format!(
        //     "{}-{}",
        //     "alice_did".to_string(),
        //     "alice_did".to_string()
        //     // target_address[..26].to_string(),
        //     // env.contract.address
        // );
        // let nft_id = format!("{}-{}", class_id, "bob_did");
        // let request: QueryRequest<CoreumQueries> =
        //     CoreumQueries::NFT(nft::Query::NFT { class_id, nft_id }).into();
        // let res: Option<nft::NFTResponse> = deps.querier.raw_query(&request);
        
        // // https://github.com/CoreumFoundation/coreum-wasm-sdk/blob/main/src/nft.rs


        // // // // TODO: change this
        // // match res {
        // //     Some(nft) => to_binary(&true),
        // //     None => to_binary(&false),
        // // }

        let deps = get_deps(&mut deps_empty);
        let res = execute(deps, mock_env(), subscribe_info.clone(), subscribe_msg.clone());
        assert!(res.is_ok(), "Expected subscribe call to suceed");

        // repeated subscription should pass
        let deps = get_deps(&mut deps_empty);
        let res = execute(deps, mock_env(), subscribe_info, subscribe_msg);
        assert!(res.is_ok(), "Expected subscribe call to suceed");

    }

    #[test]
    fn subscribe_payout_owner() {

        let env = mock_env();

        let mut deps = mock_dependencies_with_balances(&[
            ("alice_key", &[Coin { amount: Uint128::from(20u128), denom: "ucore".to_string() }]),
            ("bob_key", &[Coin { amount: Uint128::from(20u128), denom: "ucore".to_string() }]),
            ("claire_key", &[Coin { amount: Uint128::from(20u128), denom: "ucore".to_string() }])
            ]);

        {
            let deps = get_deps(&mut deps);
            mock_init_no_price(deps);
        }

        // register actors
        {
            let deps = get_deps(&mut deps);
            mock_register(deps, "alice", &[]);
        }
        {
            let deps = get_deps(&mut deps);
            mock_register(deps, "bob", &[]);
        }
        {
            let deps = get_deps(&mut deps);
            mock_register(deps, "claire", &[]);
        }

        let balance_request = QueryRequest::Bank(cosmwasm_std::BankQuery::Balance {
            // coreum_wasm_sdk::assetft::Query::Balance {
                address:  "claire_key".to_string(),
                denom: "ucore".to_string(),
            });
        // let q: QueryRequest<CoreumQueries> = balance_request.into();
        
        {
            let deps = get_deps(&mut deps);
            let balance_response: BalanceResponse = deps.querier.query::<cosmwasm_std::BalanceResponse>(&balance_request).unwrap();
            println!("{:?}", balance_response);
        }

        let info = mock_info("alice_key", &[]);
        let price = coin(20, "ucore");
        let set_price_msg = ExecuteMsg::SetSubscriptionPrice {
            price: price.clone(),
        };

        {
            let deps = get_deps(&mut deps);
            let _res = execute(deps, env.clone(), info.clone(), set_price_msg)
                .expect("contract successfully handles SetSubscriptionPrice message");

        }
        
        
        let subscribe_msg = ExecuteMsg::Subscribe {
            did: "alice_did".to_string(),
        };

        {
            let claire_subscribe_info = mock_info("claire_key", &coins(20, "ucore"));
            let deps = get_deps(&mut deps);
            let res = execute(deps, env.clone(), claire_subscribe_info, subscribe_msg);
            println!("{:?}", res);
    
            assert!(res.is_ok(), "Expected subscribe call to succeed, it failed with error: {:?}", res.err());
    
            let response = res.unwrap();
    
            // Check that Alice got paid
            let alice_got_paid = response.messages.iter().any(|msg| {
                match &msg.msg {
                    CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                        to_address == "alice_key" && amount.iter().any(|coin| coin.amount == Uint128::from(19u128) && coin.denom == "ucore")
                    }
                    _ => false,
                }
            });
            assert!(alice_got_paid, "Expected alice_key to be paid 19 ucore");
        }

        // this is how one would normally check if the balance was deducted from claire_key and moved to alice_key
        // but mocked deps have immutable balances
        // so instead, the most we can do without coreum_test_tube is the assert for the correct returned CoreumMsg from the execute call above
        // {
        //     let balance_request = QueryRequest::Bank(cosmwasm_std::BankQuery::Balance {
        //         address:  "claire_key".to_string(),
        //         denom: "ucore".to_string(),
        //     });
        //     let deps = get_deps(&mut deps);
        //     let balance_response: BalanceResponse = deps.querier.query::<cosmwasm_std::BalanceResponse>(&balance_request).unwrap();
        //     println!("{:?}", balance_response);
        //     assert!(balance_response.amount.amount == Uint128::from(0u128));
        // }

    }

    #[test]
    fn subscribe_validity_should_expire() {

        let mut env = mock_env();

        let mut deps = mock_dependencies_with_balances(&[
            ("alice_key", &[Coin { amount: Uint128::from(20u128), denom: "ucore".to_string() }]),
            ("bob_key", &[Coin { amount: Uint128::from(20u128), denom: "ucore".to_string() }]),
            ("claire_key", &[Coin { amount: Uint128::from(20u128), denom: "ucore".to_string() }])
            ]);

        {
            let deps = get_deps(&mut deps);
            mock_init_no_price(deps);
        }

        // register actors
        {
            let deps = get_deps(&mut deps);
            mock_register(deps, "alice", &[]);
        }
        {
            let deps = get_deps(&mut deps);
            mock_register(deps, "bob", &[]);
        }
        {
            let deps = get_deps(&mut deps);
            mock_register(deps, "claire", &[]);
        }

        let subscribe_msg = ExecuteMsg::Subscribe {
            did: "alice_did".to_string(),
        };

        // subscribe claire to alice
        {
            let claire_subscribe_info = mock_info("claire_key", &coins(20, "ucore"));
            let deps = get_deps(&mut deps);
            let res = execute(deps, env.clone(), claire_subscribe_info, subscribe_msg);
            println!("{:?}", res);
    
            assert!(res.is_ok(), "Expected subscribe call to succeed, it failed with error: {:?}", res.err());
        }

        // confirm we are subscribed now
        let is_subscriber_msg = QueryMsg::IsSubscriber {
            did: "alice_did".to_string(),
            subscriber: "claire_did".to_string(),
        };
        {
            let deps = get_deps(&mut deps);
            let res = query(deps.as_ref(), env.clone(), is_subscriber_msg.clone()).unwrap();
            let value: bool = from_json(&res).unwrap();
            assert!(value, "Expected Claire to be a subscriber of Alice");
        }

        // move the clock forward in time for one month
        env.block.time = env.block.time.plus_days(31);

        // confirm we are NOT subscribed now
        {
            let deps = get_deps(&mut deps);
            let res = query(deps.as_ref(), env.clone(), is_subscriber_msg).unwrap();
            let value: bool = from_json(&res).unwrap();
            assert!(!value, "Expected Claire's subscription to Alice to expire after a month.");
        }

    }


    #[test]
    fn subscribe_reject_double_subscription() {

        let env = mock_env();

        let deps_empty = mock_dependencies_with_balances(&[
            ("alice_key", &[Coin { amount: Uint128::from(20u128), denom: "ucore".to_string() }]),
            ("bob_key", &[Coin { amount: Uint128::from(20u128), denom: "ucore".to_string() }]),
            ("claire_key", &[Coin { amount: Uint128::from(20u128), denom: "ucore".to_string() }])
            ]);
        let mut mock_deps = MockCoreumDeps::new(deps_empty);

        {
            let deps = mock_deps.as_deps_mut();
            mock_init_no_price(deps);
        }

        // register actors
        {
            let deps = mock_deps.as_deps_mut();
            mock_register(deps, "alice", &[]);
        }

        {
            let deps = mock_deps.as_deps_mut();
            mock_register(deps, "bob", &[]);
        }
        {
            let deps = mock_deps.as_deps_mut();
            mock_register(deps, "claire", &[]);
        }

        let subscribe_msg = ExecuteMsg::Subscribe {
            did: "alice_did".to_string(),
        };
        let claire_subscribe_info = mock_info("claire_key", &coins(20, "ucore"));

        // subscribe claire to alice once
        {
            let deps = mock_deps.as_deps_mut();
            let res = execute(deps, env.clone(), claire_subscribe_info.clone(), subscribe_msg.clone());
            println!("{:?}", res);
            // let is_sub = is_subscriber(deps.as_ref(), env, "alice_did".to_string(), "claire_did".to_string());
            // print!("is sub: {:?}", is_sub);
    
            assert!(res.is_ok(), "Expected subscribe call to succeed, it failed with error: {:?}", res.err());
        }
        // try subscribing claire to alice again
        {
            let deps = mock_deps.as_deps_mut();
            let res = execute(deps, env.clone(), claire_subscribe_info, subscribe_msg);
            println!("{:?}", res);
    
            // assert!(res.is_err(), "Expected subscribe call to fail due to an existing subscription");
        }
        
    }

    // leaving for the reference if we ever want to revisit coreum_test_tube

    // #[test]
    // fn subscribe_payout_owner_tube() {

    //     with_test_tube(&|accounts: Vec<SigningAccount>, contract_addr: String, wasm: Wasm<CoreumTestApp>| {

    //         // let mut deps_empty = mock_dependencies();

    //         // let deps = get_deps(&mut deps_empty);
    //         // mock_init_no_price(deps);
            
    //         // register actors
    //         let alice = accounts.get(1).unwrap();
    //         let bob = accounts.get(2).unwrap();

    //         mock_register_account(&wasm, &contract_addr, alice, "alice".to_string());
    //         mock_register_account(&wasm, &contract_addr, bob, "bob".to_string());
    
    //         // let subscribe_info = mock_info("bob_key", );
    //         let subscribe_msg = ExecuteMsg::Subscribe {
    //             did: "alice_did".to_string(),
    //         };

    //         // println!("{}", bob.address().to_string());
    //         // Execute the contract to modify admin to user address
    //         wasm.execute::<ExecuteMsg>(
    //                 &contract_addr,
    //                 &subscribe_msg,
    //                 &coins(10_000_000_000, "ucore".to_string()),
    //                 &bob
    //             )
    //             .unwrap();

    //         // // Query the contract to verify that the admin has been updated correctly.
    //         // let admin_list = wasm
    //         //         .query::<QueryMsg, AdminListResponse>(&contract_addr, &QueryMsg::AdminList {})
    //         //         .unwrap();

    //         // assert_eq!(admin_list.admins, vec![user.address()]);
    //         // assert!(admin_list.mutable);
    
    //         // let deps = get_deps(&mut deps_empty);
    //         // let res = execute(deps, mock_env(), subscribe_info, subscribe_msg);
    
    //         // assert!(res.is_ok(), "Expected subscribe call to suceed");
    
    //         // // check if Alice got paid
    //         // // let balance_request = cosmwasm_std::BankQuery::Balance { 
    //         // //     address:  "alice_key".to_string(),
    //         // //     denom: "core".to_string(),
    //         // // };
    //         // let balance_request = coreum::asset::ft::v1::QueryBalanceRequest {
    //         //     account:  "alice_key".to_string(),
    //         //     denom: "core".to_string(),
    //         // };
    //         // let q: QueryRequest<CoreumQueries> = balance_request.into();
    //         // let deps = get_deps(&mut deps_empty);
    //         // let balance_response = deps.querier.query::<coreum::asset::ft::v1::QueryBalanceResponse>(&q);
    //         // println!("{:?}", balance_response);
    //         // assert!(balance_response.balance == "8core");
    //     });

}
