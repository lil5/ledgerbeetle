#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    use dotenvy::dotenv;
    use tokio::time::sleep;

    use crate::{models, responses, router};
    use axum::http::StatusCode;
    use models::TB_MAX_BATCH_SIZE;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_e2e_accountnames() {
        dotenv().ok();
        let server = TestServer::new(router().await).unwrap();
        let response = server.get("/accountnames").await;
        let json = response.json::<responses::ResponseAccountNames>();
        assert!(json.iter().any(|v| v.starts_with("assets:")));
    }

    #[tokio::test]
    async fn test_e2e_commodities() {
        dotenv().ok();
        let server = TestServer::new(router().await).unwrap();
        let response = server.get("/commodities").await;
        let json = response.json::<responses::ResponseCommodities>();
        assert!(json.iter().any(|v| v == "TEST"));
    }

    #[tokio::test]
    async fn test_e2e_one_transaction() {
        dotenv().ok();
        let server = TestServer::new(router().await).unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64;

        let transactions = vec![responses::AddTransaction {
            commodity_unit: String::from("TEST"),
            code: 9999,
            related_id: format!("{}f", 1),
            debit_account: format!("liabilities:test:{now}:debit"),
            credit_account: format!("liabilities:test:{now}:credit"),
            amount: 1,
        }];

        println!("send add transaction");
        let response = server
            .put("/add")
            .json(&responses::AddTransactions {
                full_date2: now,
                transactions,
            })
            .await;
        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "unable to add transaction",
        );

        {
            let response = server
                .get(format!("/accounttransactions/liabilities:test:{}:debit", now).as_str())
                .await;
            assert_eq!(response.status_code(), StatusCode::OK);
            let json = response.json::<responses::ResponseTransactions>();
            assert_eq!(json.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_e2e_huge_batch_transactions() {
        dotenv().ok();
        let server = TestServer::new(router().await).unwrap();
        let amount = TB_MAX_BATCH_SIZE;
        let iterations = 2;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64;

        for iteration in 0..iterations {
            let transactions = (1..amount)
                .map(|i| responses::AddTransaction {
                    commodity_unit: String::from("TEST"),
                    code: 9999,
                    related_id: format!("{}f{}", i, iteration),
                    debit_account: format!("liabilities:test:{now}:debit"),
                    credit_account: format!("liabilities:test:{now}:credit"),
                    amount: 1,
                })
                .collect();

            println!("send add transaction");
            let response = server
                .put("/add")
                .json(&responses::AddTransactions {
                    full_date2: now,
                    transactions,
                })
                .await;
            assert_eq!(
                response.status_code(),
                StatusCode::OK,
                "unable to add transaction at iteration: {}",
                iteration
            );
            sleep(Duration::from_millis(20)).await;
        }

        {
            let response = server
                .get(format!("/accounttransactions/liabilities:test:{}:debit", now).as_str())
                .await;
            assert_eq!(response.status_code(), StatusCode::OK);
            let json = response.json::<responses::ResponseTransactions>();
            assert!(json.len() > TB_MAX_BATCH_SIZE as usize);
        }
    }
}
