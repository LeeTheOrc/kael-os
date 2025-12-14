# Kael-OS Brainstorm Cloud Functions

## Setup

1. **Install dependencies:**

   ```bash
   cd cloud-functions
   npm install
   ```

2. **Configure Gemini API key:**

   ```bash
   firebase functions:config:set gemini.key="YOUR_GEMINI_API_KEY"
   ```

3. **Deploy functions:**
   ```bash
   npm run deploy
   ```

## Functions

### `dailyBrainstorm` (Scheduled)

- **Trigger:** Runs daily at 2 AM
- **Purpose:** Generates fresh ideas across 4 categories (features, UI, optimization, integration)
- **Model:** `gemini-1.5-flash` (cheapest, fastest)
- **Storage:** Firestore `brainstorm_cache` collection
- **Cleanup:** Auto-deletes ideas older than 7 days (unless starred)

### `onDemandBrainstorm` (Callable)

- **Trigger:** Called from app via `firebase.functions().httpsCallable('onDemandBrainstorm')`
- **Purpose:** Generate ideas on user request
- **Params:**
  ```typescript
  {
    category?: 'features' | 'ui' | 'optimization' | 'integration' | 'custom',
    customPrompt?: string  // if category='custom'
  }
  ```

### `toggleStarIdea` (Callable)

- **Trigger:** Called from app
- **Purpose:** Star/unstar ideas to save permanently
- **Params:**
  ```typescript
  {
    ideaId: string,
    starred: boolean
  }
  ```

## Firestore Schema

### `brainstorm_cache` Collection

```typescript
{
  category: string,           // 'features' | 'ui' | 'optimization' | 'integration'
  prompt: string,             // The prompt sent to AI
  ideas: string,              // Generated ideas (markdown/text)
  generated_at: Timestamp,
  user_id?: string,           // Set for on-demand requests
  status: 'active',
  starred: boolean,
  starred_at?: Timestamp,
  starred_by?: string,
  on_demand?: boolean         // true if user-triggered
}
```

## Local Testing

```bash
# Start emulator
npm run serve

# View logs
npm run logs
```

## Cost Optimization

- Uses **Gemini 1.5 Flash** (~$0.0001/request)
- Daily runs: 4 prompts × 30 days = ~$0.012/month
- Auto-cleanup prevents storage bloat
- Rate limiting via Firebase quotas

## Integration with Kael-OS

Add to `src-tauri/src/services/brainstorm.rs`:

```rust
pub async fn fetch_brainstorm_ideas(user: &User) -> Result<Vec<BrainstormIdea>> {
    // Fetch from Firestore
    let ideas = firebase_get_collection(user, "brainstorm_cache").await?;
    Ok(ideas)
}

pub async fn request_new_ideas(user: &User, category: &str) -> Result<BrainstormIdea> {
    // Call onDemandBrainstorm function
    let functions = get_firebase_functions();
    let result = functions.call("onDemandBrainstorm", json!({ "category": category })).await?;
    Ok(result)
}
```

## UI Integration Ideas

1. **Ideas Panel** - Show cached ideas in sidebar/panel
2. **Notification Badge** - "3 new ideas!" when fresh ones arrive
3. **Category Tabs** - Filter by features/UI/optimization/integration
4. **Star Button** - Save favorite ideas permanently
5. **Refresh Button** - Trigger `onDemandBrainstorm` with custom prompt
