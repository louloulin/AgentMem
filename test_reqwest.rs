// 测试 reqwest 是否能正常调用 Backend API

use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试 reqwest 调用 Backend API...");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    let request_body = json!({
        "content": "reqwest test",
        "user_id": "test-user",
        "agent_id": "agent-92070062-78bb-4553-9701-9a7a4a89d87a",
        "memory_type": "Episodic",
        "importance": 0.5
    });

    println!("发送请求到 http://localhost:8080/api/v1/memories");
    println!("请求体: {}", serde_json::to_string_pretty(&request_body)?);

    let response = client
        .post("http://localhost:8080/api/v1/memories")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    println!("响应状态: {}", response.status());
    
    let response_text = response.text().await?;
    println!("响应内容: {}", response_text);

    Ok(())
}

