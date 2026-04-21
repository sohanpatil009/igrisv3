// Test Gemini integration
use igrisv3::core::gemini::{enhanced_web_search, enhance_voice_command, GeminiClient};

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