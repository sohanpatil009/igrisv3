# 🔑 Test Your Gemini API Key

## Quick Test with Curl

Try this command in your terminal to test if your API key works:

```bash
curl -H 'Content-Type: application/json' \
     -d '{"contents":[{"parts":[{"text":"Hello, what is 2+2?"}]}]}' \
     -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=AIzaSyAb8RN6LMMBsptZQiH_9ns_H6ns2oSOyKIFs20xPtM6Yg7rqdIg"
```

## Expected Results:

### ✅ **If API Key Works:**
You should see a JSON response like:
```json
{
  "candidates": [
    {
      "content": {
        "parts": [
          {
            "text": "Hello! 2 + 2 = 4."
          }
        ]
      }
    }
  ]
}
```

### ❌ **If API Key Doesn't Work:**
You'll see an error like:
```json
{
  "error": {
    "code": 400,
    "message": "API key not valid. Please pass a valid API key.",
    "status": "INVALID_ARGUMENT"
  }
}
```

## 🔧 **If API Key Fails:**

1. **Get New API Key**: Go to [Google AI Studio](https://makersuite.google.com/app/apikey)
2. **Create New Key**: Click "Create API Key"
3. **Copy Key**: Make sure to copy the full key
4. **Test Again**: Use the curl command above with your new key

## 📝 **Alternative: Use Free Tier**

If you don't have a valid API key, we can:
1. **Disable Gemini**: Keep IGRIS working offline-only
2. **Use Different API**: Try a different AI service
3. **Get Valid Key**: Help you set up a proper Gemini API key

Let me know what the curl command returns!