use primitives::transaction::{SignedTransaction, Transaction};
use serde_json::json;

pub async fn send_tx_to_rpc(signed: SignedTransaction, url: &str) -> anyhow::Result<String> {
    // Test code! Initial transactions
    // From: pint, To: apple, Fee: 10, Value: 1000, Nonce: 0

    let encoded = hex::encode(signed.encode());

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "local_transaction",
        "params": [encoded],
        "id": 0
    });

    let _res = reqwest::Client::new().post(url).json(&payload).send().await;

    Ok(encoded)
}

pub async fn get_tx_from_rpc(tx_hash: String, url: &str) -> anyhow::Result<Transaction> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "transaction",
        "params": [tx_hash],
        "id": 0
    });

    let res = reqwest::Client::new()
        .post(url)
        .json(&payload)
        .send()
        .await?;

    let status = res.status();
    let ct = res.headers().get(reqwest::header::CONTENT_TYPE).cloned();
    let body = res.text().await?; // 먼저 문자열로 읽기
    dbg!(status, ct, &body);

    let resp: serde_json::Value = serde_json::from_str(&body)?;

    println!("{:?}", &resp);

    let tx_hex = resp["result"]["tx"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing result.tx"))?;
    let bytes = hex::decode(tx_hex)?;
    let (signed, _size) = SignedTransaction::decode(&bytes).unwrap();
    Ok(signed.tx)
}
