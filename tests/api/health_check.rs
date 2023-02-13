use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    //Arange
    let app = spawn_app().await;
    // Bring  in reqwest
    // to perform HTTP request against our application
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/api/health-check", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
