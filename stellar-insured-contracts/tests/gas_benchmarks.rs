#![cfg(feature = "e2e-tests")]

use ink_e2e::build_message;
use propchain_contracts::PropertyRegistry;
use propchain_traits::PropertyMetadata;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Benchmark thresholds for gas consumption (regressions)
/// Value represents gas_consumed in Substrate units (simulating CPU/Memory)
const THRESHOLD_REGISTER_PROPERTY: u64 = 1_000_000_000;
const THRESHOLD_TRANSFER_PROPERTY: u64 = 500_000_000;
const THRESHOLD_GET_PORTFOLIO: u64 = 100_000_000;

#[ink_e2e::test]
async fn benchmark_gas_register_property(mut client: ink_e2e::Client<ink_e2e::PolkadotConfig, _>) -> E2EResult<()> {
    // Given
    let constructor = PropertyRegistry::new();
    let contract_id = client
        .instantiate("propchain-contracts", &ink_e2e::alice(), constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;

    let metadata = PropertyMetadata {
        location: "Benchmark St".to_string(),
        size: 1000,
        legal_description: "Benchmark property".to_string(),
        valuation: 100000,
        documents_url: "ipfs://benchmark".to_string(),
    };

    // When
    let register_msg = build_message::<PropertyRegistry>(contract_id.clone())
        .call(|contract| contract.register_property(metadata));
    
    let call_result = client.call(&ink_e2e::alice(), register_msg, 0, None).await.expect("call failed");
    
    // Then
    let gas_consumed = call_result.handle().gas_consumed();
    println!("Gas consumed (Register): {}", gas_consumed);
    
    assert!(
        gas_consumed <= THRESHOLD_REGISTER_PROPERTY,
        "Gas regression detected! Used: {}, Threshold: {}",
        gas_consumed,
        THRESHOLD_REGISTER_PROPERTY
    );

    Ok(())
}

#[ink_e2e::test]
async fn benchmark_gas_transfer_property(mut client: ink_e2e::Client<ink_e2e::PolkadotConfig, _>) -> E2EResult<()> {
    // Given
    let constructor = PropertyRegistry::new();
    let contract_id = client
        .instantiate("propchain-contracts", &ink_e2e::alice(), constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;

    let metadata = PropertyMetadata {
        location: "Transfer St".to_string(),
        size: 500,
        legal_description: "Transfer property".to_string(),
        valuation: 50000,
        documents_url: "ipfs://transfer".to_string(),
    };

    let register_msg = build_message::<PropertyRegistry>(contract_id.clone())
        .call(|contract| contract.register_property(metadata));
    let register_result = client.call(&ink_e2e::alice(), register_msg, 0, None).await.expect("register failed");
    let property_id = register_result.return_value().expect("return value failed");

    // When
    let transfer_msg = build_message::<PropertyRegistry>(contract_id.clone())
        .call(|contract| contract.transfer_property(property_id, ink_e2e::bob().account_id()));
    
    let call_result = client.call(&ink_e2e::alice(), transfer_msg, 0, None).await.expect("call failed");
    
    // Then
    let gas_consumed = call_result.handle().gas_consumed();
    println!("Gas consumed (Transfer): {}", gas_consumed);
    
    assert!(
        gas_consumed <= THRESHOLD_TRANSFER_PROPERTY,
        "Gas regression detected! Used: {}, Threshold: {}",
        gas_consumed,
        THRESHOLD_TRANSFER_PROPERTY
    );

    Ok(())
}

#[ink_e2e::test]
async fn benchmark_gas_portfolio_analytics(mut client: ink_e2e::Client<ink_e2e::PolkadotConfig, _>) -> E2EResult<()> {
    // Given
    let constructor = PropertyRegistry::new();
    let contract_id = client
        .instantiate("propchain-contracts", &ink_e2e::alice(), constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;

    // Register 10 properties to have a portfolio
    for i in 1..=10 {
        let metadata = PropertyMetadata {
            location: format!("Prop {}", i),
            size: 100,
            legal_description: "Part of portfolio".to_string(),
            valuation: 10000,
            documents_url: "ipfs://part".to_string(),
        };
        let register_msg = build_message::<PropertyRegistry>(contract_id.clone())
            .call(|contract| contract.register_property(metadata));
        client.call(&ink_e2e::alice(), register_msg, 0, None).await.expect("register failed");
    }

    // When
    let summary_msg = build_message::<PropertyRegistry>(contract_id.clone())
        .call(|contract| contract.get_portfolio_summary(ink_e2e::alice().account_id()));
    
    let dry_run_result = client.call_dry_run(&ink_e2e::alice(), &summary_msg, 0, None).await.expect("dry run failed");
    
    // Then
    let gas_consumed = dry_run_result.handle().gas_consumed();
    println!("Gas consumed (Portfolio Summary, 10 items): {}", gas_consumed);
    
    // Portfolio summary is optimized with unrolled loops, gas should be stable
    assert!(
        gas_consumed <= THRESHOLD_GET_PORTFOLIO,
        "Gas regression detected! Used: {}, Threshold: {}",
        gas_consumed,
        THRESHOLD_GET_PORTFOLIO
    );

    Ok(())
}

#[ink_e2e::test]
async fn benchmark_gas_mint_property_token(mut client: ink_e2e::Client<ink_e2e::PolkadotConfig, _>) -> E2EResult<()> {
    use property_token::property_token::PropertyToken;
    let constructor = PropertyToken::new();
    let contract_id = client
        .instantiate("property-token", &ink_e2e::alice(), constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;

    // When
    let mint_msg = build_message::<PropertyToken>(contract_id.clone())
        .call(|contract| contract.mint(1, ink_e2e::alice().account_id(), 100)); // Simulating minting 100 shares for property 1
    
    let call_result = client.call(&ink_e2e::alice(), mint_msg, 0, None).await.expect("call failed");
    
    // Then
    let gas_consumed = call_result.handle().gas_consumed();
    println!("Gas consumed (Mint Token): {}", gas_consumed);
    
    assert!(gas_consumed <= 1_500_000_000);

    Ok(())
}

#[ink_e2e::test]
async fn benchmark_gas_batch_transfer_token(mut client: ink_e2e::Client<ink_e2e::PolkadotConfig, _>) -> E2EResult<()> {
    use property_token::property_token::PropertyToken;
    let constructor = PropertyToken::new();
    let contract_id = client
        .instantiate("property-token", &ink_e2e::alice(), constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;

    // Pre-mint some tokens
    for i in 1..=5 {
        let mint_msg = build_message::<PropertyToken>(contract_id.clone())
            .call(|contract| contract.mint(i, ink_e2e::alice().account_id(), 100));
        client.call(&ink_e2e::alice(), mint_msg, 0, None).await.expect("mint failed");
    }

    // When - batch transfer 5 tokens
    let ids = vec![1, 2, 3, 4, 5];
    let amounts = vec![10, 10, 10, 10, 10];
    let transfer_msg = build_message::<PropertyToken>(contract_id.clone())
        .call(|contract| contract.safe_batch_transfer_from(
            ink_e2e::alice().account_id(), 
            ink_e2e::bob().account_id(), 
            ids.clone(), 
            amounts.clone(), 
            vec![]
        ));
    
    let call_result = client.call(&ink_e2e::alice(), transfer_msg, 0, None).await.expect("call failed");
    
    // Then
    let gas_consumed = call_result.handle().gas_consumed();
    println!("Gas consumed (Batch Transfer 5 items): {}", gas_consumed);
    
    assert!(gas_consumed <= 2_500_000_000);

    Ok(())
}
