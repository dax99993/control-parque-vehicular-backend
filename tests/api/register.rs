use crate::helpers::spawn_app;

#[tokio::test]
async fn register_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (serde_json::json!({"first_name": "Ursula", "last_name": "Le guin", "email": "ursula_le_guin@gmail.com"}), "missing the password"),
        (serde_json::json!({"first_name": "Ursula", "last_name": "Le guin", "password": "password1234"}), "missing the email"),
        (serde_json::json!({"first_name": "Ursula", "email": "ursula_le_guin@gmail.com", "password": "password1234"}), "missing the last name"),
        (serde_json::json!({"last_name": "Le guin", "email": "ursula_le_guin@gmail.com", "password": "password1234"}), "missing the first name"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_register(&invalid_body).await;

        // Assert
        assert_eq!(400,
                   response.status().as_u16(),
                   "the api did not fail with 400 bad request when the payload was {}",
                   error_message
                   );
    }
}

#[tokio::test]
async fn suscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (serde_json::json!({"first_name": "Ursula", "last_name": "Le guin", "email": "ursula_le_guin@gmail.com", "password": ""}), "empty the password"),
        (serde_json::json!({"first_name": "Ursula", "last_name": "Le guin", "email": "", "password": "password1234"}), "empty the email"),
        (serde_json::json!({"first_name": "Ursula", "last_name": "Le guin", "email": "invalid_password@email", "password": "password1234"}), "invalid the email"),
        (serde_json::json!({"first_name": "Ursula", "last_name": "", "email": "ursula_le_guin@gmail.com", "password": "password1234"}), "empty the last name"),
        (serde_json::json!({"first_name": "", "last_name": "Le guin", "email": "ursula_le_guin@gmail.com", "password": "password1234"}), "empty the first name"),
    ];

    for (body, description) in test_cases {
        // Act
        let response = app.post_register(&body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}",
            description
        );
    }
}

#[tokio::test]
async fn register_persists_the_new_user() {
    // Arrange
    let app = spawn_app().await;
    let body = serde_json::json!({
        "first_name": "Ursula",
        "last_name": "Le guin",
        "email": "ursula_le_guin@gmail.com",
        "password": "password1234",
        "re_password": "password1234",
    });

    /*
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    */

    // Act
    let _response = app.post_register(&body).await;

    // Assert
    // We need the OFFSET since we are registering test_user when calling spawn_app
    let saved = sqlx::query!("SELECT first_name, last_name, email FROM users OFFSET 1",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved user.");

    assert_eq!(saved.first_name, "Ursula");
    assert_eq!(saved.last_name, "Le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn register_returns_a_201_for_valid_json_data() {
    // Arrange
    let app = spawn_app().await;
    let body = serde_json::json!({
        "first_name": "Ursula",
        "last_name": "Le guin",
        "email": "ursula_le_guin@gmail.com",
        "password": "password1234",
        "re_password": "password1234",
    });

    /*
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    */

    // Act
    let response = app.post_register(&body).await;

    // Assert
    assert_eq!(201, response.status().as_u16());
}
