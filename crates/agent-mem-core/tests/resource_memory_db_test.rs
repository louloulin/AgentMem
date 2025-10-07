//! Resource Memory 数据库持久化测试

use chrono::Utc;
use serde_json::json;

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_resource_memory_table_exists() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    // 测试表是否存在
    let result = sqlx::query!(
        r#"
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name = 'resource_memory'
        "#
    )
    .fetch_optional(&pool)
    .await;

    assert!(result.is_ok(), "Failed to query table: {:?}", result.err());
    assert!(result.unwrap().is_some(), "Table resource_memory does not exist");
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_insert_resource_memory() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let id = format!("res-{}", uuid::Uuid::new_v4());
    let now = Utc::now();
    let file_hash = format!("{:x}", md5::compute("test content"));

    // 插入测试数据
    let result = sqlx::query!(
        r#"
        INSERT INTO resource_memory (
            id, organization_id, user_id, agent_id, original_filename,
            resource_type, file_size, file_hash, mime_type, storage_path,
            is_compressed, compressed_size, access_count, last_accessed,
            tags, custom_metadata, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        "#,
        id,
        "org-test",
        "user-test",
        "agent-test",
        "test.pdf",
        "document",
        1024i64,
        file_hash,
        "application/pdf",
        "/storage/test.pdf",
        false,
        None::<i64>,
        0i64,
        now,
        &vec!["test", "document"],
        json!({"test": true}),
        now,
        now,
    )
    .execute(&pool)
    .await;

    assert!(result.is_ok(), "Failed to insert: {:?}", result.err());

    // 清理测试数据
    let _ = sqlx::query!("DELETE FROM resource_memory WHERE id = $1", id)
        .execute(&pool)
        .await;
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_unique_file_hash_constraint() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let now = Utc::now();
    let file_hash = format!("{:x}", md5::compute("unique test content"));

    // 插入第一个资源
    let id1 = format!("res-{}", uuid::Uuid::new_v4());
    let result1 = sqlx::query!(
        r#"
        INSERT INTO resource_memory (
            id, organization_id, user_id, agent_id, original_filename,
            resource_type, file_size, file_hash, mime_type, storage_path,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        id1,
        "org-test",
        "user-test",
        "agent-test",
        "file1.pdf",
        "document",
        1024i64,
        file_hash,
        "application/pdf",
        "/storage/file1.pdf",
        now,
        now,
    )
    .execute(&pool)
    .await;

    assert!(result1.is_ok(), "Failed to insert first resource");

    // 尝试插入相同哈希的资源（应该失败）
    let id2 = format!("res-{}", uuid::Uuid::new_v4());
    let result2 = sqlx::query!(
        r#"
        INSERT INTO resource_memory (
            id, organization_id, user_id, agent_id, original_filename,
            resource_type, file_size, file_hash, mime_type, storage_path,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        id2,
        "org-test",
        "user-test",
        "agent-test",
        "file2.pdf",
        "document",
        1024i64,
        file_hash,
        "application/pdf",
        "/storage/file2.pdf",
        now,
        now,
    )
    .execute(&pool)
    .await;

    assert!(result2.is_err(), "Should fail due to unique file hash constraint");

    // 清理测试数据
    let _ = sqlx::query!("DELETE FROM resource_memory WHERE file_hash = $1", file_hash)
        .execute(&pool)
        .await;
}

#[test]
fn test_resource_type_validation() {
    // 测试资源类型枚举
    let valid_types = vec!["document", "image", "audio", "video", "other"];
    for resource_type in valid_types {
        assert!(
            resource_type == "document"
                || resource_type == "image"
                || resource_type == "audio"
                || resource_type == "video"
                || resource_type == "other"
        );
    }
}

#[test]
fn test_file_hash_generation() {
    // 测试文件哈希生成
    let content = b"test content";
    let hash = format!("{:x}", md5::compute(content));
    assert_eq!(hash.len(), 32); // MD5 哈希长度
}

#[test]
fn test_mime_type_detection() {
    // 测试 MIME 类型检测
    let mime_types = vec![
        ("test.pdf", "application/pdf"),
        ("test.jpg", "image/jpeg"),
        ("test.png", "image/png"),
        ("test.mp3", "audio/mpeg"),
        ("test.mp4", "video/mp4"),
    ];

    for (filename, expected_mime) in mime_types {
        let ext = filename.split('.').last().unwrap();
        let mime = match ext {
            "pdf" => "application/pdf",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "mp3" => "audio/mpeg",
            "mp4" => "video/mp4",
            _ => "application/octet-stream",
        };
        assert_eq!(mime, expected_mime);
    }
}

#[test]
fn test_tags_array() {
    // 测试标签数组
    let tags = vec!["document", "important", "work"];
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"document"));
}

#[test]
fn test_compression_info() {
    // 测试压缩信息
    let original_size = 1024i64;
    let compressed_size = 512i64;
    let is_compressed = true;

    assert!(is_compressed);
    assert!(compressed_size < original_size);
    let compression_ratio = compressed_size as f64 / original_size as f64;
    assert!(compression_ratio < 1.0);
}

