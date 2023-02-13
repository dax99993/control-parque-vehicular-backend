use crate::helpers::spawn_app;

#[tokio::test]
async fn sucess_response_when_user_logout() {
    // Arrange
    let app = spawn_app().await;
    
    // Act - Part 1 - Login
    let response = app.test_user.login(&app).await;

    #[derive(serde::Deserialize)]
    struct JsonResponse {
        pub status: String,
        pub token: String,
    }

    let json_response = response.json::<JsonResponse>().await.unwrap();
    let token = json_response.token;
    // Act - Part 2 - Logout
    app.api_client
        .get(format!("{}/api/auth/logout", &app.address))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to execute request");

    // Asert
    assert_eq!(json_response.status, "sucess".to_string());
}

#[tokio::test]
async fn error_response_when_anonymous_user_without_header_tries_to_logout() {
    // Arrange
    let app = spawn_app().await;
    
    // Act - Fake login token
    let token = "".to_string();
    // Act - Part 2 - Try to logout
    let response = app.api_client
        .get(format!("{}/api/auth/logout", &app.address))
        //.bearer_auth(token)
        .send()
        .await
        .expect("Failed to execute request");

    #[derive(serde::Deserialize)]
    struct JsonResponse {
        pub status: String,
        pub message: String,
    }

    let json_response = response.json::<JsonResponse>().await.unwrap();

    // Asert
    assert_eq!(json_response.status, "failed".to_string());
    assert_eq!(json_response.message, "You are not logged in, please provide token".to_string());
}

#[tokio::test]
async fn error_response_when_anonymous_user_with_invalid_token_tries_to_logout() {
    // Arrange
    let app = spawn_app().await;
    
    // Act - Fake login token
    let token = uuid::Uuid::new_v4().to_string();
    // Act - Part 2 - Try to logout
    let response = app.api_client
        .get(format!("{}/api/auth/logout", &app.address))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to execute request");

    #[derive(serde::Deserialize)]
    struct JsonResponse {
        pub status: String,
        pub message: String,
    }

    let json_response = response.json::<JsonResponse>().await.unwrap();

    // Asert
    assert_eq!(json_response.status, "failed".to_string());
    assert_eq!(json_response.message, "Invalid token".to_string());
}
