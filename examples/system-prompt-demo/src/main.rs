//! System Prompt Demo
//!
//! This example demonstrates the system prompt extraction and construction features.
//! It shows how to:
//! 1. Add memories for a user
//! 2. Extract relevant memories for system prompts
//! 3. Construct complete system messages with memory context

use agent_mem_core::client::{AgentMemClient, Messages, MemoryType};
use agent_mem_traits::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== AgentMem System Prompt Demo ===\n");

    // 1. Create client
    let client = AgentMemClient::default();
    println!("✅ Client created\n");

    // 2. Create a test user
    let user = client.create_user("Alice".to_string()).await?;
    println!("✅ User created: {} (ID: {})\n", user.name, user.id);

    // 3. Add some memories for the user
    println!("--- Adding memories ---");

    client
        .add(
            Messages::Single("I love pizza".to_string()),
            Some(user.id.clone()),
            None,
            None,
            None,
            false,
            Some(MemoryType::Semantic),
            None,
        )
        .await?;
    println!("  ✓ Added: I love pizza");

    client
        .add(
            Messages::Single("My favorite color is blue".to_string()),
            Some(user.id.clone()),
            None,
            None,
            None,
            false,
            Some(MemoryType::Semantic),
            None,
        )
        .await?;
    println!("  ✓ Added: My favorite color is blue");

    client
        .add(
            Messages::Single("I work as a software engineer".to_string()),
            Some(user.id.clone()),
            None,
            None,
            None,
            false,
            Some(MemoryType::Semantic),
            None,
        )
        .await?;
    println!("  ✓ Added: I work as a software engineer");

    client
        .add(
            Messages::Single("I enjoy hiking on weekends".to_string()),
            Some(user.id.clone()),
            None,
            None,
            None,
            false,
            Some(MemoryType::Episodic),
            None,
        )
        .await?;
    println!("  ✓ Added: I enjoy hiking on weekends\n");

    // 4. Extract memory for system prompt
    println!("--- Extracting memory for system prompt ---");
    let query = "What do you know about me?";
    println!("Query: {}\n", query);

    let memory_prompt = client
        .extract_memory_for_system_prompt(query.to_string(), Some(user.id.clone()), Some(5))
        .await?;

    println!("Extracted Memory Prompt:");
    println!("{}", memory_prompt);
    println!();

    // 5. Construct complete system message
    println!("--- Constructing system message ---");
    let user_message = "Tell me about my preferences";
    println!("User message: {}\n", user_message);

    let system_message = client
        .construct_system_message(
            user_message.to_string(),
            Some(user.id.clone()),
            None, // Use default system prefix
        )
        .await?;

    println!("Complete System Message:");
    println!("{}", system_message);
    println!();

    // 6. Construct system message with custom prefix
    println!("--- Constructing system message with custom prefix ---");
    let custom_prefix = "You are a personal assistant who knows the user well:";
    let system_message_custom = client
        .construct_system_message(
            user_message.to_string(),
            Some(user.id.clone()),
            Some(custom_prefix.to_string()),
        )
        .await?;

    println!("System Message with Custom Prefix:");
    println!("{}", system_message_custom);
    println!();

    // 7. Test with a query that has no relevant memories
    println!("--- Testing with irrelevant query ---");
    let irrelevant_query = "What is the weather like on Mars?";
    println!("Query: {}\n", irrelevant_query);

    let empty_prompt = client
        .extract_memory_for_system_prompt(
            irrelevant_query.to_string(),
            Some(user.id.clone()),
            Some(5),
        )
        .await?;

    println!("Extracted Memory Prompt:");
    println!("{}", empty_prompt);
    println!();

    println!("=== Demo completed successfully! ===");

    Ok(())
}
