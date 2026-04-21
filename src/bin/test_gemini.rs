// Test Gemini integration
use igrisv3::core::gemini::{enhanced_web_search, enhance_voice_command, GeminiClient};
use serde_json;

#[tokio::main]
async fn main() {
    println!("🧪 Testing Gemini Integration\n");
    
    // Test 1: Connectivity
    println!("1. Testing connectivity...");
    let is_online = GeminiClient::is_online().await;
    println!("   Result: {}\n", if is_online { "✅ Online" } else { "❌ Offline" });
    
    if !is_online {
        println!("❌ Cannot test Gemini features - no internet connection");
        return;
    }
    
    // Test 1.5: List available models
    println!("1.5. Listing available models...");
    let client = GeminiClient::new();
    match client.list_models().await {
        Ok(models) => {
            println!("   ✅ Available models:");
            // Parse and show model names
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&models) {
                if let Some(models_array) = json.get("models").and_then(|m| m.as_array()) {
                    for model in models_array.iter().take(5) { // Show first 5 models
                        if let Some(name) = model.get("name").and_then(|n| n.as_str()) {
                            println!("      - {}", name);
                        }
                    }
                }
            }
        }
        Err(e) => println!("   ❌ Failed to list models: {}", e),
    }
    println!();
    
    // Test 2: Web Search
    println!("2. Testing web search...");
    match enhanced_web_search("What is 2+2?").await {
        Some(response) => println!("   ✅ Web search works: {}", response),
        None => println!("   ❌ Web search failed"),
    }
    println!();
    
    // Test 3: Voice Command Enhancement
    println!("3. Testing voice command enhancement...");
    match enhance_voice_command("open chrome browser").await {
        Some(response) => println!("   ✅ Voice enhancement works: {}", response),
        None => println!("   ❌ Voice enhancement failed"),
    }
    println!();
    
    println!("🎉 Gemini integration test complete!");
}