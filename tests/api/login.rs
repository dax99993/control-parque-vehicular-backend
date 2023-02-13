use crate::helpers::spawn_app;


#[tokio::test]
async fn sucess_response_on_valid_credentials() {
    // Arrange
    let app = spawn_app().await;
    // Act - Try to login
    let login_body = serde_json::json!({
        "email": app.test_user.email,
        "password": app.test_user.password,
    });
    let response = app.post_login(&login_body).await;

    let text_response = response.text().await.unwrap();
    //dbg!("{:?}", &text_response);

    assert!(text_response.contains("token"));
}



#[tokio::test]
async fn error_response_on_invalid_credentials() {
    // Arrange
    let app = spawn_app().await;
    
    // Act - Try to login
    let login_body = serde_json::json!({
        "email": "random-email",
        "password": "random-password",
    });
    let response = app.post_login(&login_body).await;

    #[derive(serde::Deserialize)]
    struct JsonResponse {
        pub status: String,
        pub message: String,
    }

    let json_response = response.json::<JsonResponse>().await.unwrap();

    // Asert
    assert_eq!(json_response.status, "failed".to_string());
}
