# 🚀 Optimized Publishing Plan - Arch Linux + Firebase Android

## 🎯 Overview

**Focus**: Arch Linux (primary) + Android (brainstorming companion with Firebase sync)

⚠️ **CRITICAL CONSTRAINT: NO NODE.JS ON DESKTOP**

```
┌─────────────────────────────────────────────────────────────┐
│ 🚫 DESKTOP (Arch Linux): ZERO Node.js/npm                  │
│    • Pure Rust backend only                                 │
│    • No npm packages, no node_modules                       │
│    • No Node.js installation anywhere on system             │
│    • Cloud Functions deploy REMOTELY to Firebase            │
│                                                             │
│ ✅ CLOUD (Firebase): Node.js 18 (managed by Google)        │
│    • Cloud Functions run on Google's infrastructure         │
│    • We never run npm locally - just deploy code           │
│    • No local Node.js binary needed                         │
│                                                             │
│ ⚠️  ANDROID: Prefer no npm, but React Native may need it   │
│    • Explore pure Kotlin/Jetpack alternatives first        │
│    • Only use npm if no other option exists                │
└─────────────────────────────────────────────────────────────┘
```

```
DESKTOP (Arch Linux)        CLOUD FUNCTIONS           MOBILE (Android)
    ┌─────────────┐         ┌──────────────┐         ┌──────────────┐
    │ Kael-OS AI  │◄───────►│ Scheduled AI │◄───────►│ Kael-OS Idea │
    │  Terminal   │         │ Brainstorming│         │  Brainstorm  │
    │  + Ideas UI │         │ (Gemini Flash│         │  + Auto-Sync │
    │  + Hybrid   │         │  Daily @ 2AM)│         │              │
    │  PURE RUST  │         │   Node.js    │         │   PURE       │
    │  NO Node.js │         │ (on Firebase)│         │   KOTLIN     │
    └─────────────┘         └──────────────┘         └──────────────┘
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  │
                  ┌───────────────▼────────────────┐
                  │      Cloud Firestore           │
                  │  • Chat history                │
                  │  • Brainstorm cache (7-day)    │
                  │  • Starred ideas (permanent)   │
                  │  • Task queue                  │
                  │  • Provider usage tracking     │
                  └────────────────────────────────┘
```

---

## 🧠 NEW FEATURE: AI BRAINSTORMING SYSTEM (COMPLETED ✅)

### Cloud Functions Architecture

**Daily Scheduled Brainstorming:**

- **Cloud Function**: `dailyBrainstorm` runs at 2 AM daily
- **AI Model**: Gemini 1.5 Flash (cheapest at ~$0.0001/request)
- **Categories**: Features, UI/UX, Optimization, Integration
- **Cost**: ~$0.012/month for daily runs
- **Auto-cleanup**: Deletes ideas older than 7 days (except starred)

**On-Demand Generation:**

- **Cloud Function**: `onDemandBrainstorm` (callable from app)
- **Custom Prompts**: Users can provide their own brainstorm prompts
- **Real-time**: Ideas appear instantly in the app

**Idea Management:**

- **Cloud Function**: `toggleStarIdea` for permanent saving
- **Local Cache**: `/tmp/kael_brainstorm_cache.json` for offline access
- **Firestore Storage**: `brainstorm_cache` collection

### Desktop Integration (Completed)

**UI Components:**

- **Ideas Panel**: Full-screen brainstorm viewer (toggle via sidebar button)
- **Category Filters**: All, Features, UI/UX, Optimize, Integrate, Starred
- **New Ideas Badge**: Shows count of unstarred fresh ideas
- **Generate Buttons**: Quick-fire idea generation per category
- **Custom Prompt Input**: User-defined brainstorm prompts
- **Star System**: Save favorite ideas permanently

**Backend Services:**

- `services/brainstorm.rs`: Firestore integration
- `components/brainstorm.rs`: Dioxus UI component
- Local caching for offline access
- Real-time sync with Firebase

**Files Created:**

```
cloud-functions/
├── index.js            # Cloud Functions (3 functions)
├── package.json        # Node.js dependencies
├── firebase.json       # Functions config
└── README.md          # Setup & deployment guide

src-tauri/src/
├── services/
│   └── brainstorm.rs  # Backend service
└── components/
    └── brainstorm.rs  # UI panel
```

**Deployment:**

```bash
# NOTE: These commands run on Firebase servers (Google Cloud), NOT on your desktop
# You don't need Node.js locally - Firebase CLI handles the deployment
# Your desktop remains pure Rust/no Node.js ✅

cd cloud-functions
# This folder is deployed TO Firebase, not run locally
firebase functions:config:set gemini.key="YOUR_GEMINI_API_KEY"
firebase deploy  # Uploads code to Google Cloud - doesn't require local npm!
```

**Cloud Functions are serverless:**

- Code runs on Google's servers, not your machine
- You only need the Firebase CLI (lightweight, no Node.js req'd)
- Deployment is one-way: upload → done
- Desktop stays 100% Node.js-free ✅

---

## 🧩 NEW FEATURE: SMART CONTEXT-AWARE REFORMATTING (IN PROGRESS)

### Local AI Intelligence Layer

**Goal**: Train local AI to handle day-to-day Arch Linux tasks with context awareness, only escalating complex requests to online AI providers.

### Feature 1: System-Aware Command Translation

**Problem**: Users copy commands from internet tutorials that don't match their system config.

**Solution**: Local AI learns user preferences and auto-corrects commands:

```rust
// src-tauri/src/services/command_rewriter.rs

pub struct UserContext {
    pub package_manager: String,        // "paru" (not yay)
    pub shell: String,                  // "fish" (not bash)
    pub init_system: String,            // "systemd"
    pub network_interface: String,      // "wlp3s0" (actual WiFi adapter)
    pub gpu_driver: String,             // "nvidia" or "amd"
    pub preferred_editor: String,       // "nvim"
    pub user_name: String,
    pub hostname: String,
}

pub async fn rewrite_command(input: &str, context: &UserContext) -> String {
    let mut output = input.to_string();

    // Rule 1: Package manager replacement
    if output.contains("yay -S") || output.contains("yay -Syu") {
        output = output.replace("yay", &context.package_manager);
    }

    // Rule 2: Shell-specific syntax
    if output.contains("export ") && context.shell == "fish" {
        // Convert: export VAR=value  →  set -x VAR value
        output = convert_bash_to_fish(&output);
    }

    // Rule 3: Network interface names
    if output.contains("wlan0") || output.contains("eth0") {
        // Auto-detect user's actual interface
        let actual_interface = detect_network_interface();
        output = output.replace("wlan0", &actual_interface)
                      .replace("eth0", &actual_interface);
    }

    // Rule 4: Hardware-specific replacements
    if output.contains("rtl8192") && context.network_interface.starts_with("iwl") {
        // User has Intel WiFi, not Realtek
        output = output.replace("rtl8192", "iwlwifi");
    }

    output
}
```

**Example Usage:**

```
User pastes:
  sudo systemctl enable NetworkManager
  export WIFI_INTERFACE=wlan0
  yay -S discord

Local AI rewrites to:
  sudo systemctl enable NetworkManager
  set -x WIFI_INTERFACE wlp3s0    # Fish syntax + actual interface
  paru -S discord                  # User's preferred AUR helper
```

### Feature 2: Hardware Detection & Code Adaptation

**Auto-detect user's hardware on first run:**

```rust
pub async fn build_user_context() -> UserContext {
    UserContext {
        package_manager: detect_aur_helper(),        // Check for paru/yay
        shell: std::env::var("SHELL").unwrap(),      // /usr/bin/fish
        network_interface: get_primary_wifi_interface(),  // ip link show
        gpu_driver: detect_gpu_driver(),             // lspci | grep VGA
        preferred_editor: std::env::var("EDITOR").unwrap_or("nvim".into()),
        user_name: std::env::var("USER").unwrap(),
        hostname: hostname::get().unwrap(),
    }
}

fn detect_aur_helper() -> String {
    if Command::new("which").arg("paru").output().is_ok() {
        "paru".to_string()
    } else if Command::new("which").arg("yay").output().is_ok() {
        "yay".to_string()
    } else {
        "paru".to_string()  // Default recommendation
    }
}

fn get_primary_wifi_interface() -> String {
    let output = Command::new("ip")
        .args(&["link", "show"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Parse for wireless interface (wlp*, wlan*)
    for line in stdout.lines() {
        if line.contains("wlp") || line.contains("wlan") {
            return line.split(':').nth(1).unwrap().trim().to_string();
        }
    }

    "wlan0".to_string()  // Fallback
}
```

### Feature 3: Local AI Decision Tree

**When to use local AI vs online AI:**

```rust
pub enum AIDecision {
    HandleLocally(String),          // Local AI can do this
    EscalateToCloud(String),        // Need online AI help
    AskForClarification(String),    // Ambiguous request
}

pub fn should_escalate(user_input: &str, context: &UserContext) -> AIDecision {
    // Pattern matching for local capabilities
    let patterns = [
        // Local AI can handle:
        (r"(install|remove|update).+(package|discord|chrome)", false),
        (r"(yay|pacman|paru) -S", false),
        (r"systemctl (start|stop|enable|disable)", false),
        (r"(wifi|network|ethernet) (setup|config)", false),
        (r"export \w+=", false),  // Simple env vars
        (r"how do i (install|setup|configure)", false),

        // Must escalate to cloud:
        (r"write (a|an) (app|program|script|function)", true),
        (r"debug (this|my) code", true),
        (r"explain (how|why|what)", true),
        (r"optimize|refactor|improve", true),
        (r"create.*from scratch", true),
    ];

    for (pattern, needs_cloud) in patterns {
        if Regex::new(pattern).unwrap().is_match(user_input) {
            if needs_cloud {
                return AIDecision::EscalateToCloud(
                    format!("This requires deep reasoning - using {}",
                            context.preferred_cloud_provider)
                );
            } else {
                return AIDecision::HandleLocally(
                    "I can handle this with my local knowledge!".to_string()
                );
            }
        }
    }

    AIDecision::AskForClarification("Could you clarify what you need?".to_string())
}
```

### Feature 4: Personality Preservation

**Ensure local AI maintains consistent personality:**

```rust
pub struct KaelOSPersonality {
    pub name: String,
    pub traits: Vec<String>,
    pub catchphrases: Vec<String>,
    pub response_style: String,
}

impl Default for KaelOSPersonality {
    fn default() -> Self {
        KaelOSPersonality {
            name: "Kael".to_string(),
            traits: vec![
                "Enthusiastic about Arch Linux".to_string(),
                "Helpful and patient".to_string(),
                "Uses emojis sparingly but effectively".to_string(),
                "Prefers command-line solutions".to_string(),
                "Always suggests best practices".to_string(),
            ],
            catchphrases: vec![
                "Let me help you with that!".to_string(),
                "That's an easy fix on Arch!".to_string(),
                "I've detected your setup and adjusted the command.".to_string(),
            ],
            response_style: "friendly-technical".to_string(),
        }
    }
}

pub fn inject_personality(base_response: &str, personality: &KaelOSPersonality) -> String {
    let mut response = base_response.to_string();

    // Add personality markers
    if base_response.contains("command") {
        response = format!("🔧 {}", response);
    }

    if base_response.contains("paru") || base_response.contains("pacman") {
        response.push_str("\n\n💡 Tip: I auto-corrected this to use your preferred package manager!");
    }

    response
}
```

### Feature 5: Training Data Collection

**Learn from user corrections:**

```rust
pub struct CommandCorrection {
    pub original: String,
    pub corrected: String,
    pub context: String,
    pub timestamp: i64,
}

pub async fn learn_from_user(correction: CommandCorrection) {
    // Store in local SQLite
    let conn = rusqlite::Connection::open("~/.config/kael-os/learning.db").unwrap();

    conn.execute(
        "INSERT INTO corrections (original, corrected, context, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            correction.original,
            correction.corrected,
            correction.context,
            correction.timestamp,
        ],
    ).unwrap();

    // After 10+ corrections of same pattern, add to automatic rules
    update_rewrite_rules(&conn);
}
```

### Implementation Plan

**Phase 1: Command Rewriter (Week 1)**

- Detect user's system config (package manager, shell, hardware)
- Build basic rewrite rules (yay→paru, bash→fish)
- Store user context in `~/.config/kael-os/context.json`

**Phase 2: Local AI Decision Tree (Week 2)**

- Pattern matching for local vs cloud tasks
- Integration with hybrid assist system
- Personality injection layer

**Phase 3: Learning System (Week 3)**

- Track user corrections
- Build adaptive rewrite rules
- Export learned patterns for community sharing

**UI Integration:**

```
User types: yay -S discord

Kael-OS shows:
┌──────────────────────────────────────┐
│ 🔧 Auto-corrected command:           │
│                                      │
│   paru -S discord                   │
│                                      │
│ (Changed yay → paru based on your   │
│  system config)                      │
│                                      │
│ [✓ Run This] [✗ Use Original]      │
└──────────────────────────────────────┘
```

---

## ⚡ PHASE 1: ARCH LINUX RELEASE (ASAP - This Week)

### 1.1 Build & Package for Arch

```bash
# 1. Build optimized release
cargo build --release --manifest-path Kael-OS-AI/src-tauri/Cargo.toml

# 2. Copy binary to Arch PKGBUILD directory
cp target/release/kael-os Kael-OS-AI/PKGBUILD-assets/

# 3. Create PKGBUILD (Arch package definition)
mkdir -p pkgbuild/
# Update PKGBUILD with:
# - version = "0.2.0"
# - sha256sum of binary
# - Install paths: /usr/bin/kael-os
```

### 1.2 Push to AUR (Arch User Repository)

```bash
# Prerequisites:
# - AUR account set up
# - SSH key configured for AUR

# Push to AUR
cd pkgbuild/
git push aur master

# NATIVE ARCH INSTALL (via paru):
# paru -S kael-os

# This installs the native compiled binary, not an AppImage!
# Binary location: /usr/bin/kael-os
# Config location: ~/.config/kael-os/
# Database location: ~/.config/kael-os/kael.db
```

### 1.3 GitHub Release

```bash
# Tag release
git tag v0.2.0
git push origin v0.2.0

# Create GitHub Release with:
# - Binary: target/release/kael-os
# - Arch PKGBUILD
# - SHA256 checksums
# - Release notes
```

**Deliverable**: Users on Arch Linux can `yay -S kael-os` → Done! ✅

---brainstorm_cache)
│ ├── 💡 AI Ideas (view Cloud Function generated ideas)
│ ├── ⭐ Starred Ideas (permanently saved)
│ ├── ⚙️ Settings (OAuth login)
│ └── 🔄 Sync Status
├── Firebase Integration
│ ├── Realtime listener on /brainstorm_cache (Cloud Function ideas)
│ ├── Realtime listener on /brainstorms/{user_id} (user ideas)
│ ├── Background sync every 15 min
│ └── Auto-execute non-sensitive tasks
├── Cloud Function Calls
│ ├── onDemandBrainstorm (trigger new AI ideas)
│ ├── toggleStarIdea (save favorites)
│ └── Real-time sync with desktop
└── Background Jobs (via react-native-background-tasks)
├── Every 15 min: Sync with Firebase
├── Every 1 hour: Download ready tasks
└── On login: Pull full brainstorm history + AI ideas
│ ├── ⚙️ Settings (OAuth login)
│ └── 🔄 Sync Status
├── Firebase Integration
│ ├── Realtime listener on /brainstorms/{user_id}
│ ├── Background sync every 15 min
│ └── Auto-execute non-sensitive tasks
└── Background Jobs (via react-native-background-tasks)
├── Every 15 min: Sync with Firebase
├── Every 1 hour: Download ready tasks
└── On login: Pull full brainstorm history

````

### 2.2 Step 1: Core Setup (Week 1)

**ANDROID ALTERNATIVE TO npm:**

Before using npm/Node.js on Android, explore these pure Kotlin solutions:

```kotlin
// Prefer: Firebase SDK for Kotlin (no npm)
implementation 'com.google.firebase:firebase-firestore-ktx'
implementation 'com.google.firebase:firebase-auth-ktx'

// Instead of: React Native (requires npm/Node.js)
// Use: Jetpack Compose (pure Kotlin, no JavaScript)
````

If React Native is necessary (last resort):

```bash
# WARNING: This brings Node.js into the project
# Only do this if Kotlin/Jetpack Compose can't work

# Prefer: Evaluate Kotlin Multiplatform Mobile (KMM)
# It avoids JavaScript entirely
```

**Pure Kotlin Firebase Setup (PREFERRED - NO npm):**

```kotlin
// File: app/build.gradle.kts
dependencies {
    implementation 'com.google.firebase:firebase-firestore-ktx'
    implementation 'com.google.firebase:firebase-auth-ktx'
    implementation 'androidx.compose.ui:ui:1.5.0'
    implementation 'androidx.lifecycle:lifecycle-runtime-ktx:2.6.0'
}
```

**File: `app/screens/BrainstormScreen.kt`**

```kotlin
@Composable
fun BrainstormScreen(userId: String) {
    val db = FirebaseFirestore.getInstance()
    var brainstorms by remember { mutableStateOf<List<Brainstorm>>(emptyList()) }
    var syncing by remember { mutableStateOf(false) }

    LaunchedEffect(userId) {
        syncing = true
        db.collection("users")
            .document(userId)
            .collection("brainstorms")
            .orderBy("timestamp", Query.Direction.DESCENDING)
            .addSnapshotListener { snapshot, error ->
                if (error != null) {
                    Log.e("BrainstormScreen", "Sync error", error)
                    syncing = false
                    return@addSnapshotListener
                }
                brainstorms = snapshot?.documents?.mapNotNull { doc ->
                    doc.toObject(Brainstorm::class.java)?.copy(id = doc.id)
                } ?: emptyList()
                syncing = false
            }
    }

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Text("💭 Brainstorm Ideas", fontSize = 18.sp, fontWeight = FontWeight.Bold)

        // Input section
        var title by remember { mutableStateOf("") }
        var content by remember { mutableStateOf("") }

        TextField(
            value = title,
            onValueChange = { title = it },
            label = { Text("Title (optional)") },
            modifier = Modifier.fillMaxWidth()
        )

        TextField(
            value = content,
            onValueChange = { content = it },
            label = { Text("Your idea...") },
            modifier = Modifier.fillMaxWidth().heightIn(min = 120.dp),
            maxLines = 5
        )

        Button(
            onClick = {
                if (content.isNotBlank()) {
                    saveBrainstorm(db, userId, title.ifBlank { "Untitled" }, content)
                    title = ""
                    content = ""
                }
            },
            modifier = Modifier.fillMaxWidth()
        ) {
            Text("Save Idea")
        }

        if (syncing) {
            Text("🔄 Syncing...", color = Color.Gray)
        }

        // List of brainstorms
        LazyColumn {
            items(brainstorms) { brainstorm ->
                BrainstormCard(brainstorm)
            }
        }
    }
}

fun saveBrainstorm(
    db: FirebaseFirestore,
    userId: String,
    title: String,
    content: String
) {
    val brainstorm = hashMapOf(
        "title" to title,
        "content" to content,
        "timestamp" to Timestamp.now(),
        "readyForExecution" to false
    )

    db.collection("users")
        .document(userId)
        .collection("brainstorms")
        .add(brainstorm)
        .addOnSuccessListener { Log.d("Brainstorm", "Saved") }
        .addOnFailureListener { e -> Log.e("Brainstorm", "Error", e) }
}
```

✅ **This approach is ZERO Node.js - pure Kotlin & Firebase**

---

### 2.3 Step 2: Firebase Real-Time Sync (Week 1-2)

for user brainstorms
export function useBrainstormSync(userId: string) {
const [brainstorms, setBrainstorms] = useState<Brainstorm[]>([]);
const [syncing, setSyncing] = useState(false);

useEffect(() => {
setSyncing(true);

    // Listen to changes in Firestore
    const unsubscribe = firestore()
      .collection("users")
      .doc(userId)
      .collection("brainstorms")
      .orderBy("timestamp", "desc")
      .onSnapshot(
        (snapshot) => {
          const data = snapshot.docs.map((doc) => ({
            id: doc.id,
            ...doc.data(),
          })) as Brainstorm[];
          setBrainstorms(data);
          setSyncing(false);
        },
        (error) => {
          console.error("Sync error:", error);
          setSyncing(false);
        }
      );

    return unsubscribe;

}, [userId]);

return { brainstorms, syncing };
}

// Real-time listener for AI-generated ideas (from Cloud Functions)
export function useAIIdeasSync() {
const [aiIdeas, setAiIdeas] = useState<Brainstorm[]>([]);
const [syncing, setSyncing] = useState(false);

useEffect(() => {
setSyncing(true);

    // Listen to Cloud Function generated ideas
    const unsubscribe = firestore()
      .collection("brainstorm_cache")
      .orderBy("generated_at", "desc")
      .limit(50) // Last 50 ideas
      .onSnapshot(
        (snapshot) => {
          const data = snapshot.docs.map((doc) => ({
            id: doc.id,
            category: doc.data().category,
            ideas: doc.data().ideas,
            generated_at: doc.data().generated_at,
            starred: doc.data().starred || false,
          }));
          setAiIdeas(data);
          setSyncing(false);
        },
        (error) => {
          console.error("AI Ideas sync error:", error);
          setSyncing(false);
        }
      );

    return unsubscribe;

}, []);

return { aiIdea {
const data = snapshot.docs.map((doc) => ({
id: doc.id,
...doc.data(),
})) as Brainstorm[];
setBrainstorms(data);
setSyncing(false);
},
(error) => {
console.error("Sync error:", error);
setSyncing(false);
}
);

    return unsubscribe;

}, [userId]);

return { brainstorms, syncing };
}

// Push new brainstorm
export async function saveBrainstorm(
userId: string,
title: string,
content: string,
tags: string[]
): Promise<void> {
await firestore()
.collection("users")
.doc(userId)
.collection("brainstorms")
.add({
title,
content,
tags,
timestamp: firestore.FieldValue.serverTimestamp(),
readyForExecution: false,
});
}

````

### 2.4 Step 3: Background Sync (Every 15 Minutes)

**File: `app/services/backgroundSync.ts`**

```typescript
import BackgroundTimer from "react-native-background-timer";
import firestore from "@react-native-firebase/firestore";

export function startBackgroundSync(userId: string) {
  // Sync every 15 minutes
  BackgroundTimer.setInterval(async () => {
    try {
      console.log("[Sync] Starting background sync...");

      // Pull latest brainstorms from Firestore
      const snapshot = await firestore()
        .collection("users")
        .doc(userId)
        .collection("brainstorms")
        .where("timestamp", ">", Date.now() - 15 * 60 * 1000) // Last 15 min
        .get();

      console.log(`[Sync] Fetched ${snapshot.docs.length} new brainstorms`);

      // Check for tasks ready to execute
      const readyTasks = snapshot.docs.filter(
        (doc) => doc.data().readyForExecution === true
      );

      if (readyTasks.length > 0) {
        console.log(`[Sync] Found ${readyTasks.length} ready tasks`);
        // Send notification to user
        notifyReadyTasks(readyTasks);
      }

      // Update last sync timestamp
      await saveLastSyncTime(userId);
    } catch (error) {
      console.error("[Sync] Error:", error);
    }
  }, 15 * 60 * 1000); // 15 minutes
}

async function notifyReadyTasks(tasks: any[]) {
  // Send local notification
  // (User gets notification: "3 tasks ready on desktop!")
}
````

### 2.5 Step 4: Auto-Execute Non-Sensitive Tasks

**File: `app/services/taskExecutor.ts`**

Non-sensitive tasks (tasks that don't need passwords/secrets):

```typescript
export interface Task {
  id: string;
  type: "brainstorm_compile" | "list_files" | "fetch_api_docs" | "create_note";
  params: any;
  requiresAuth: boolean;
  readyForExecution: boolean;
}

// Tasks that DON'T need input/passwords
export async function executeAutoTask(task: Task, userId: string) {
  switch (task.type) {
    case "brainstorm_compile":
      // Compile all brainstorms with tag 'architecture'
      const brainstorms = await getBrainstormsByTag("architecture", userId);
      const compiled = brainstorms.map((b) => ({
        title: b.title,
        content: b.content,
        timestamp: b.timestamp,
      }));

      // Save compiled version back to Firestore
      await firestore()
        .collection("users")
        .doc(userId)
        .collection("compiled_thoughts")
        .doc(task.id)
        .set({
          content: compiled,
          createdAt: firestore.FieldValue.serverTimestamp(),
        });
      break;

    case "list_files":
      // Can check file sizes, list directory (no execution)
      // Results saved to Firestore for desktop to pull
      break;

    case "fetch_api_docs":
      // Download public API documentation
      // Cache locally for offline use
      break;

    case "create_note":
      // Create note from brainstorm
      await firestore()
        .collection("users")
        .doc(userId)
        .collection("notes")
        .add({
          content: task.params.content,
          source: "brainstorm_" + task.params.brainstormId,
          timestamp: firestore.FieldValue.serverTimestamp(),
        });
      break;
  }

  // Mark task as completed
  await firestore()
    .collection("users")
    .doc(userId)
    .collection("tasks")
    .doc(task.id)
    .update({
      status: "completed",
      completedAt: firestore.FieldValue.serverTimestamp(),
    });
}
```

### 2.6 Step 5: Main UI (Week 2)

**File: `app/screens/BrainstormScreen.tsx`**

```typescript
import React from "react";
import {
  View,
  TextInput,
  FlatList,
  TouchableOpacity,
  Text,
} from "react-native";
import auth from "@react-native-firebase/auth";
import { useBrainstormSync, saveBrainstorm } from "../services/firebaseSync";

export function BrainstormScreen() {
  const user = auth().currentUser;
  const { brainstorms, syncing } = useBrainstormSync(user!.uid);
  const [title, setTitle] = React.useState("");
  const [content, setContent] = React.useState("");

  const handleSave = async () => {
    if (content.trim()) {
      await saveBrainstorm(
        user!.uid,
        title || "Untitled",
        content,
        extractTags(content)
      );
      setTitle("");
      setContent("");
    }
  };

  return (
    <View style={{ flex: 1, padding: 16 }}>
      <Text style={{ fontSize: 18, fontWeight: "bold", marginBottom: 12 }}>
        💭 Brainstorm Ideas
      </Text>

      {/* Input section */}
      <TextInput
        style={{ borderWidth: 1, padding: 8, marginBottom: 8, borderRadius: 8 }}
        placeholder="Title (optional)"
        value={title}
        onChangeText={setTitle}
      />

      <TextInput
        style={{
          borderWidth: 1,
          padding: 8,
          marginBottom: 8,
          borderRadius: 8,
          height: 120,
          textAlignVertical: "top",
        }}
        placeholder="Your idea..."
        value={content}
        onChangeText={setContent}
        multiline
      />

      <TouchableOpacity
        style={{
          backgroundColor: "#e040fb",
          padding: 12,
          borderRadius: 8,
          marginBottom: 16,
        }}
        onPress={handleSave}
      >
        <Text
          style={{ color: "white", fontWeight: "bold", textAlign: "center" }}
        >
          Save Idea
        </Text>
      </TouchableOpacity>

      {/* Sync status */}
      {syncing && (
        <Text style={{ color: "#666", marginBottom: 8 }}>🔄 Syncing...</Text>
      )}

      {/* List of brainstorms */}
      <FlatList
        data={brainstorms}
        keyExtractor={(item) => item.id}
        renderItem={({ item }) => (
          <View
            style={{
              padding: 12,
              marginBottom: 8,
              backgroundColor: "#f5f5f5",
              borderRadius: 8,
            }}
          >
            <Text style={{ fontWeight: "bold" }}>{item.title}</Text>
            <Text style={{ color: "#666", marginTop: 4 }}>{item.content}</Text>
            <Text style={{ fontSize: 12, color: "#999", marginTop: 8 }}>
              {new Date(item.timestamp).toLocaleDateString()}
    Desktop v0.2.0 Features ✅

- [x] Hybrid Assist (local → cloud fallback with provider order)
- [x] "Try next provider" button with prompt re-use
- [x] Provider usage tracking & last-used badge
- [x] Inline provider provenance ("via X")
- [x] Cloud Functions brainstorming (3 functions)
- [x] Ideas panel UI with category filters
- [x] Star/unstar idea management
- [x] Local model constraints (llama:latest, phi3)
- [x] GitHub auth shortcut button
- [x] Brainstorm service backend (Rust)
- [x] Local caching for offline access

### Desktop v0.3.0 Features (Smart Context-Aware Reformatting) 📋

- [ ] System context detection (package manager, shell, hardware)
- [ ] Command rewriter service (yay→paru, bash→fish, etc)
- [ ] Hardware-aware code adaptation (WiFi interface names, GPU drivers)
- [ ] Local AI decision tree (local vs cloud escalation)
- [ ] Personality preservation system
- [ ] Learning from user corrections
- [ ] Auto-correct UI with approval flow
- [ ] Community rule sharing (export learned patterns)

### Arch Linux Release (THIS WEEK) 🚀

- [ ] Build release binary
- [ ] Create PKGBUILD
- [ ] Push to AUR
- [ ] Deploy Cloud Functions to Firebase
- [ ] Create GitHub release v0.2.0
- [ ] Test install: `yay -S kael-os`
- [ ] Verify Cloud Functions work with desktop app

### Android Phase 1: Brainstorm + Sync (Week 1-2)

- [ ] React Native project setup
- [ ] Firebase Firestore integration
- [ ] Real-time sync listener (user brainstorms)
- [ ] Real-time sync listener (AI ideas from Cloud Functions)
- [ ] Background sync every 15 min
- [ ] Brainstorm UI screen
- [ ] AI Ideas viewer (from Cloud Functions)
- [ ] Star/unstar ideas (call toggleStarIdea function)
- [ ] OAuth login (Google/GitHub)
- [ ] Trigger on-demand ideas (call onDemandBrainstorm)

### Android Phase 2: Task Execution (Week 2-3)
View AI-generated ideas (from Cloud Functions daily run)
3. Write your own brainstorms (automatic sync to Firebase)
4. Star favorite ideas for permanent saving
5. Trigger on-demand AI idea generation
6. App auto-syncs every 15 minutes
7. Ready tasks execute without your input
8. Notifications tell you what's ready

**When You Get Back to PC:**

1. Open Kael-OS Desktop
2. Click "💡 Open Ideas Panel" in sidebar
3. View all AI-generated ideas (synced from Cloud Functions)
4. See all brainstorms from phone
5. Starred ideas are there permanently
6. Generate fresh ideas on-demand
7. Compiled ideas + executed tasks waiting
8. You pick up where Android left off

**Cloud Functions Working in Background:**

1. Daily at 2 AM: Generates 4 categories of fresh ideas
2. Auto-cleanup: Removes ideas older than 7 days (except starred)
3. Cost: ~$0.012/month (basically free)
4. On-demand: Generate ideas anytime from desktop or Androidorming complete
Week 1:     🚀 Arch Linux v0.2.0 release (AUR)
            🚀 Deploy Cloud Functions (brainstorming)
Week 2:     🔨 Android core + Firebase sync
            🔨 AI Ideas viewer + user brainstorms
Week 3:     🔨 Android task execution
            🔨 Star management + on-demand generation
Week 4:     ✅ Both platforms working together
            ✅ Desktop ↔ Android ↔ Cloud Functions sync
    let brainstorms: Vec<Brainstorm> = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(brainstorms)
}
```

---

## 📋 Implementation Checklist

### Arch Linux (THIS WEEK) ✅

- [ ] Build release binary
- [ ] Create PKGBUILD
- [ ] Push to AUR
- [ ] Create GitHub release v0.2.0
- [ ] Test install: `yay -S kael-os`

### Android Phase 1: Brainstorm + Sync (Week 1-2)

- [ ] React Native project setup
- [ ] Firebase Firestore integration
- [ ] Real-time sync listener
- [ ] Background sync every 15 min
- [ ] Brainstorm UI screen
- [ ] OAuth login (Google/GitHub)

### Android Phase 2: Task Execution (Week 2-3)

- [ ] Task executor service
- [ ] Auto-execute non-sensitive tasks
- [ ] Notification system
- [ ] Desktop integration (pull brainstorms)
- [ ] Sync with desktop chat history

---

## 📊 Timeline

```
Current:    ✅ Hybrid Assist & Brainstorming complete
Week 1:     🚀 Arch Linux v0.2.0 release (AUR)
            🚀 Deploy Cloud Functions (brainstorming)
            🔨 Smart reformatting Phase 1 (system context detection)
Week 2:     🔨 Android core + Firebase sync
            🔨 AI Ideas viewer + user brainstorms
            🔨 Smart reformatting Phase 2 (decision tree)
Week 3:     🔨 Android task execution
            🔨 Star management + on-demand generation
            🔨 Smart reformatting Phase 3 (learning system)
Week 4:     ✅ Both platforms working together
            ✅ Desktop ↔ Android ↔ Cloud Functions sync
Week 5:     ✅ v0.3.0 release with smart context-aware reformatting
```

---

## 🎯 Final Result

**When You're Away From PC:**

1. Open Kael-OS Android app
2. Brainstorm ideas (automatic sync to Firebase)
3. App auto-syncs every 15 minutes
4. Ready tasks execute without your input
5. Notifications tell you what's ready

**When You Get Back to PC:**

1. Open Kael-OS Desktop
2. All brainstorms from phone are there
3. Compiled ideas + executed tasks waiting
4. You pick up where Android left off

---

## 📊 Timeline

```
Week 1:     ✅ Arch Linux released (AUR)
Week 2:     🔨 Android core + Firebase sync
Week 3:     🔨 Android task execution
Week 4:     ✅ Both platforms working together
```

---

## 🚀 NEXT IMMEDIATE STEP

**Do this RIGHT NOW to start Arch release:**

```bash
# 1. Build release
cd Kael-OS-AI/src-tauri
cargo build --release

# 2. Check binary
ls -lh target/release/kael-os

# 3. Show me binary size & path
# Then I'll help you create the PKGBUILD
```
