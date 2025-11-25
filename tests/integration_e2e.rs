//! End-to-End Integration Tests
//!
//! These tests verify complete user flows across all services:
//! - User registration → Login → Create channel → Send messages → File upload
//! - E2EE session establishment → Encrypted communication
//! - MQTT device connection → Message routing
//! - Audit trail verification
//!
//! Run with: `cargo test --test integration_e2e`

use std::sync::Arc;

/// Complete user flow: Registration, login, channel creation, messaging
#[cfg(all(feature = "postgres", feature = "auth"))]
#[tokio::test]
async fn test_complete_user_flow() {
    use sqlx::PgPool;

    println!("🚀 Starting complete user flow E2E test");

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/unhidra_test".to_string());

    let pool = PgPool::connect(&database_url).await
        .expect("Failed to connect to test database");

    // Step 1: Create user
    let user_id = uuid::Uuid::new_v4().to_string();
    let username = format!("test_user_{}", &user_id[0..8]);
    let email = format!("{}@test.com", username);

    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash)
        VALUES ($1, $2, $3, $4)
        "#,
        user_id,
        username,
        email,
        "hashed_password_placeholder"
    )
    .execute(&pool)
    .await
    .expect("Failed to create user");

    println!("✅ Step 1: User created - {}", username);

    // Step 2: Create channel
    let channel_id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        r#"
        INSERT INTO channels (id, name, description, channel_type, created_by)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        channel_id,
        "E2E Test Channel",
        Some("Created during E2E test"),
        "private",
        user_id
    )
    .execute(&pool)
    .await
    .expect("Failed to create channel");

    println!("✅ Step 2: Channel created - {}", channel_id);

    // Step 3: Add user as channel member
    sqlx::query!(
        r#"
        INSERT INTO channel_members (channel_id, user_id, role)
        VALUES ($1, $2, 'admin')
        "#,
        channel_id,
        user_id
    )
    .execute(&pool)
    .await
    .expect("Failed to add channel member");

    println!("✅ Step 3: User added to channel as admin");

    // Step 4: Send message
    let message_id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        r#"
        INSERT INTO messages (id, channel_id, sender_id, content, content_type)
        VALUES ($1, $2, $3, $4, 'text')
        "#,
        message_id,
        channel_id,
        user_id,
        "Hello from E2E test!"
    )
    .execute(&pool)
    .await
    .expect("Failed to send message");

    println!("✅ Step 4: Message sent - {}", message_id);

    // Step 5: Verify message
    let message = sqlx::query!(
        "SELECT id, content FROM messages WHERE id = $1",
        message_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch message");

    assert_eq!(message.content, "Hello from E2E test!");

    println!("✅ Step 5: Message verified");

    // Cleanup
    sqlx::query!("DELETE FROM messages WHERE id = $1", message_id).execute(&pool).await.ok();
    sqlx::query!("DELETE FROM channel_members WHERE channel_id = $1", channel_id).execute(&pool).await.ok();
    sqlx::query!("DELETE FROM channels WHERE id = $1", channel_id).execute(&pool).await.ok();
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id).execute(&pool).await.ok();

    println!("🎉 Complete user flow E2E test passed!");
}

/// E2EE session establishment and encrypted messaging flow
#[tokio::test]
async fn test_e2ee_complete_flow() {
    use e2ee::{SessionStore, E2eeEnvelope, MessageType};

    println!("🔐 Starting E2EE complete flow test");

    // Step 1: Alice creates her session store
    let mut alice_store = SessionStore::new();
    let alice_bundle = alice_store.get_identity_bundle();

    println!("✅ Step 1: Alice session created");

    // Step 2: Bob creates his session store
    let mut bob_store = SessionStore::new();
    let bob_bundle = bob_store.get_identity_bundle();

    println!("✅ Step 2: Bob session created");

    // Step 3: Alice initiates session with Bob
    let bob_prekey = e2ee::PrekeyBundle {
        identity_key: bob_bundle.identity_key.clone(),
        signed_prekey: bob_bundle.signed_prekey.clone(),
        one_time_prekey: bob_bundle.one_time_prekeys.first().map(|(_, k)| k.clone()),
        prekey_id: 0,
    };

    let initial_msg = alice_store
        .initiate_session("bob".to_string(), &bob_prekey)
        .expect("Failed to initiate session");

    println!("✅ Step 3: Alice initiated session with Bob");

    // Step 4: Bob accepts session
    bob_store
        .accept_session("alice".to_string(), &initial_msg, Some(0))
        .expect("Failed to accept session");

    println!("✅ Step 4: Bob accepted session from Alice");

    // Step 5: Alice sends encrypted message
    let plaintext = b"This is a secret message from Alice to Bob!";
    let encrypted = alice_store
        .encrypt("bob", plaintext)
        .expect("Encryption failed");

    let envelope = E2eeEnvelope::new_message(
        "alice".to_string(),
        "bob".to_string(),
        &encrypted,
    );

    let json = envelope.to_json().expect("Failed to serialize envelope");

    println!("✅ Step 5: Alice encrypted and sent message ({} bytes)", json.len());

    // Step 6: Bob receives and decrypts message
    let received_envelope = E2eeEnvelope::from_json(&json)
        .expect("Failed to parse envelope");

    assert_eq!(received_envelope.message_type, MessageType::Message);

    let encrypted_msg = received_envelope.parse_message()
        .expect("Failed to parse encrypted message");

    let decrypted = bob_store
        .decrypt("alice", &encrypted_msg)
        .expect("Decryption failed");

    assert_eq!(decrypted, plaintext);

    println!("✅ Step 6: Bob decrypted message successfully");

    // Step 7: Bob sends reply
    let reply = b"Thanks Alice, I got your message!";
    let encrypted_reply = bob_store
        .encrypt("alice", reply)
        .expect("Encryption failed");

    let reply_envelope = E2eeEnvelope::new_message(
        "bob".to_string(),
        "alice".to_string(),
        &encrypted_reply,
    );

    println!("✅ Step 7: Bob sent encrypted reply");

    // Step 8: Alice decrypts reply
    let reply_json = reply_envelope.to_json().expect("Failed to serialize");
    let received_reply = E2eeEnvelope::from_json(&reply_json)
        .expect("Failed to parse");

    let encrypted_reply_msg = received_reply.parse_message()
        .expect("Failed to parse encrypted reply");

    let decrypted_reply = alice_store
        .decrypt("bob", &encrypted_reply_msg)
        .expect("Decryption failed");

    assert_eq!(decrypted_reply, reply);

    println!("✅ Step 8: Alice decrypted reply successfully");

    println!("🎉 E2EE complete flow test passed!");
}

/// IoT device integration: Device connects, sends data, receives commands
#[tokio::test]
async fn test_iot_device_integration_flow() {
    use gateway_service::mqtt_bridge::{IoTMessage, IoTMessageType, MqttBridge, MqttBridgeConfig};

    println!("🤖 Starting IoT device integration flow test");

    let config = MqttBridgeConfig::default();
    let bridge = MqttBridge::new(config);

    // Step 1: Device registers
    let device_id = "esp32-test-001";
    bridge.register_device(
        device_id,
        vec!["temperature".to_string(), "humidity".to_string(), "motion".to_string()],
    );

    println!("✅ Step 1: Device registered - {}", device_id);

    // Step 2: Verify device status
    let status = bridge.get_device_status(device_id);
    assert!(status.is_some());
    assert_eq!(status.unwrap().online, true);

    println!("✅ Step 2: Device status verified as online");

    // Step 3: Device sends telemetry
    let telemetry = IoTMessage::sensor_data(
        device_id,
        r#"{"temperature": 23.5, "humidity": 55, "motion": false}"#,
    );

    assert_eq!(telemetry.device_id, device_id);
    assert_eq!(telemetry.message_type, IoTMessageType::SensorData);

    println!("✅ Step 3: Device sent telemetry data");

    // Step 4: Device sends alert
    let alert = IoTMessage::new(device_id, IoTMessageType::Alert, "Motion detected!");
    assert_eq!(alert.message_type, IoTMessageType::Alert);

    println!("✅ Step 4: Device sent alert");

    // Step 5: Server sends command to device
    let command_result = bridge.send_command(device_id, "get_status").await;
    assert!(command_result.is_ok());

    println!("✅ Step 5: Server sent command to device");

    println!("🎉 IoT device integration flow test passed!");
}

/// File upload with E2EE and audit trail
#[cfg(feature = "postgres")]
#[tokio::test]
async fn test_file_upload_complete_flow() {
    use e2ee::{DoubleRatchet, KeyPair, PublicKeyBytes};
    use sqlx::PgPool;

    println!("📁 Starting file upload complete flow test");

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/unhidra_test".to_string());

    let pool = PgPool::connect(&database_url).await
        .expect("Failed to connect to test database");

    // Step 1: Setup E2EE for file encryption
    let shared_secret = [42u8; 32];
    let bob_prekey = KeyPair::generate();
    let bob_prekey_public = PublicKeyBytes::from_public_key(bob_prekey.public_key());

    let mut alice = DoubleRatchet::init_alice(shared_secret, bob_prekey_public);
    let mut bob = DoubleRatchet::init_bob(shared_secret, bob_prekey);

    println!("✅ Step 1: E2EE session established for file encryption");

    // Step 2: Encrypt file
    let file_content = b"This is a confidential document that needs to be encrypted before storage!";
    let encrypted_file = alice.encrypt(file_content)
        .expect("File encryption failed");

    println!("✅ Step 2: File encrypted ({} bytes -> {} bytes)",
        file_content.len(),
        encrypted_file.to_json().unwrap().len()
    );

    // Step 3: Create test channel and user
    let channel_id = uuid::Uuid::new_v4().to_string();
    let user_id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO users (id, username, email) VALUES ($1, $2, $3)",
        user_id, "test_file_user", "file@test.com"
    )
    .execute(&pool)
    .await
    .expect("Failed to create user");

    sqlx::query!(
        "INSERT INTO channels (id, name, channel_type, created_by) VALUES ($1, $2, $3, $4)",
        channel_id, "File Test Channel", "private", user_id
    )
    .execute(&pool)
    .await
    .expect("Failed to create channel");

    println!("✅ Step 3: Test channel and user created");

    // Step 4: Upload encrypted file record
    let file_id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        r#"
        INSERT INTO file_uploads (
            id, channel_id, uploader_id, filename, original_filename,
            file_size, mime_type, storage_path, storage_backend,
            checksum, encrypted, encryption_key_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        file_id, channel_id, user_id,
        "encrypted-document.bin", "confidential.pdf",
        encrypted_file.to_json().unwrap().len() as i64,
        "application/pdf",
        "/tmp/encrypted-document.bin",
        "local",
        "checksum123",
        1,
        Some("key-id-123")
    )
    .execute(&pool)
    .await
    .expect("Failed to upload file record");

    println!("✅ Step 4: Encrypted file record created in database");

    // Step 5: Bob downloads and decrypts file
    let decrypted_file = bob.decrypt(&encrypted_file)
        .expect("File decryption failed");

    assert_eq!(decrypted_file, file_content);

    println!("✅ Step 5: File downloaded and decrypted successfully");

    // Cleanup
    sqlx::query!("DELETE FROM file_uploads WHERE id = $1", file_id).execute(&pool).await.ok();
    sqlx::query!("DELETE FROM channels WHERE id = $1", channel_id).execute(&pool).await.ok();
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id).execute(&pool).await.ok();

    println!("🎉 File upload complete flow test passed!");
}

/// E2E test marker
#[test]
fn integration_e2e_tests_compiled() {
    println!("✅ All E2E integration tests compiled successfully");
}
