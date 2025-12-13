# Kael-OS Android Implementation Plan

## 📱 Platform Decision: React Native vs Flutter vs Kotlin

### Comparison:
```
                React Native    Flutter         Kotlin/Jetpack
─────────────────────────────────────────────────────────────
Existing Code   JavaScript/TS   Dart            From scratch
Learning Curve  Moderate        High            High
Performance     Good            Excellent       Native
Firebase SDK    ✅ Full         ✅ Full         ✅ Full
TTM (Time)      2-3 weeks       3-4 weeks       4-6 weeks
Code Sharing    Partial (logic) Partial (logic) No
─────────────────────────────────────────────────────────────
```

### Recommendation: **React Native** (leverages your TypeScript skills)

**Why:**
- You already know JavaScript/TypeScript
- Can share business logic with Dioxus
- Strong Firebase integration
- Active community
- Faster development time

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────┐
│     Kael-OS Android (React Native)                  │
├─────────────────────────────────────────────────────┤
│                                                       │
│  ┌──────────────────────────────────────────────┐  │
│  │      React Native Screens                    │  │
│  │  - Chat (Firestore real-time)                │  │
│  │  - Terminal (Android PTY library)            │  │
│  │  - Settings (OAuth via ASWebAuthenticationUI)│  │
│  │  - GPG/SSL Manager                           │  │
│  └──────────────────────────────────────────────┘  │
│                    ▼                                 │
│  ┌──────────────────────────────────────────────┐  │
│  │      Firebase Integration                    │  │
│  │  - Cloud Firestore (chat history)            │  │
│  │  - Authentication (Google/GitHub OAuth)      │  │
│  │  - Cloud Storage (GPG keys, certs)           │  │
│  │  - Cloud Messaging (notifications)           │  │
│  └──────────────────────────────────────────────┘  │
│                    ▼                                 │
│  ┌──────────────────────────────────────────────┐  │
│  │      Native Modules                          │  │
│  │  - PTY (via react-native-pty)                │  │
│  │  - GPG (via native RN bridge)                │  │
│  │  - Secure Storage (keystore)                 │  │
│  │  - File System Access                        │  │
│  └──────────────────────────────────────────────┘  │
│                                                       │
└─────────────────────────────────────────────────────┘
       ▼
   Cloud Services (same as desktop)
```

## 📋 Implementation Timeline

### Week 1-2: Project Setup
- [ ] Initialize React Native project: `npx react-native init KaelOS`
- [ ] Install core dependencies:
  - `@react-navigation/native` (navigation)
  - `firebase` (Firebase SDK)
  - `@react-native-async-storage/async-storage` (local persistence)
  - `react-native-keychain` (secure credential storage)
  - `react-native-webview` (OAuth handling)

### Week 2-3: Authentication & Core UI
- [ ] Implement OAuth flow:
  - Google login with `react-native-google-signin`
  - GitHub login via WebView
  - Store tokens in Android Keystore
- [ ] Create main tab navigation:
  - Chat tab
  - Terminal tab (placeholder for now)
  - Settings tab
  - GPG/Security tab
- [ ] Setup dark theme (match desktop)

### Week 3-4: Firebase Integration
- [ ] Connect Firestore for chat history
- [ ] Implement real-time message sync
- [ ] Add offline persistence
- [ ] Store API keys encrypted in Cloud Storage

### Week 4-5: Terminal & Advanced Features
- [ ] Research Android PTY libraries (harder than desktop)
- [ ] Implement limited terminal (commands only, no full shell)
- [ ] Add command suggestions
- [ ] Show output in scrollable view

### Week 5-6: GPG & SSL Management
- [ ] Create native bridge to Android's GPG libraries
- [ ] List system GPG keys
- [ ] Export/import encrypted keys via Firebase
- [ ] Show SSL certificate info

### Week 6-7: Testing & Distribution
- [ ] Build APK for sideloading
- [ ] Create AAB for Google Play
- [ ] Setup Firebase App Distribution for beta
- [ ] Write user documentation

## 🛠️ Project Structure

```
kael-os-mobile/
├── ios/                      (iOS later)
├── android/
│   ├── app/src/main/
│   │   ├── AndroidManifest.xml
│   │   └── java/
│   │       └── com/leetheorc/kaelosmobile/
│   │           ├── MainActivity.kt
│   │           ├── modules/
│   │           │   ├── PTYModule.kt
│   │           │   ├── GPGModule.kt
│   │           │   └── SecureStorageModule.kt
│   │           └── utils/
│   └── build.gradle
├── src/
│   ├── screens/
│   │   ├── ChatScreen.tsx
│   │   ├── TerminalScreen.tsx
│   │   ├── SettingsScreen.tsx
│   │   └── SecurityScreen.tsx
│   ├── components/
│   │   ├── ChatMessage.tsx
│   │   ├── MessageInput.tsx
│   │   └── TerminalOutput.tsx
│   ├── services/
│   │   ├── authService.ts
│   │   ├── firestoreService.ts
│   │   ├── updateService.ts
│   │   └── encryptionService.ts
│   ├── hooks/
│   │   ├── useAuth.ts
│   │   ├── useChat.ts
│   │   └── useUpdate.ts
│   ├── types/
│   │   └── index.ts
│   ├── App.tsx
│   └── index.ts
├── .env.local
├── app.json
├── package.json
└── tsconfig.json
```

## 📦 Dependencies

### Core
```json
{
  "react": "^18.2.0",
  "react-native": "^0.72.0",
  "typescript": "^5.0.0"
}
```

### Navigation & UI
```json
{
  "@react-navigation/native": "^6.1.9",
  "@react-navigation/bottom-tabs": "^6.5.9",
  "react-native-screens": "^3.24.0",
  "react-native-safe-area-context": "^4.7.2",
  "react-native-gesture-handler": "^2.14.0",
  "@react-native-community/blur": "^4.3.4"
}
```

### Firebase
```json
{
  "firebase": "^10.6.0",
  "@react-native-firebase/app": "^18.3.0",
  "@react-native-firebase/auth": "^18.3.0",
  "@react-native-firebase/firestore": "^18.3.0",
  "@react-native-firebase/storage": "^18.3.0",
  "@react-native-firebase/messaging": "^18.3.0"
}
```

### Authentication
```json
{
  "react-native-google-signin": "^12.2.0",
  "react-native-webview": "^13.6.3",
  "react-native-keychain": "^8.1.2"
}
```

### Storage & Encryption
```json
{
  "@react-native-async-storage/async-storage": "^1.21.0",
  "react-native-crypto": "^2.2.0",
  "crypto-js": "^4.1.1"
}
```

### Development
```json
{
  "@react-native-community/eslint-config": "^3.2.0",
  "babel-plugin-module-resolver": "^5.0.0",
  "@testing-library/react-native": "^12.3.0"
}
```

## 🔐 Security on Android

### Secure Storage:
```typescript
import * as Keychain from 'react-native-keychain';

// Store Firebase id_token securely
await Keychain.setGenericPassword('user_id_token', token, {
  service: 'com.leetheorc.kaelosmobile.auth',
  securityLevel: Keychain.SecurityLevel.VeryStrong,
});

// Retrieve for encryption
const token = await Keychain.getGenericPassword({
  service: 'com.leetheorc.kaelosmobile.auth',
});
```

### Encryption:
```typescript
import { encrypt, decrypt } from './encryptionService';

// Encrypt API keys before storing
const encrypted = await encrypt(apiKey, idToken);
await Firestore.collection('users')
  .doc(uid)
  .collection('api_keys')
  .doc('mistral')
  .set({ value: encrypted });
```

### APK Signing:
```bash
# Generate keystore
keytool -genkey -v -keystore ~/kael-os.keystore \
  -keyalg RSA -keysize 2048 -validity 10000 \
  -alias kael-os-key

# Sign APK
jarsigner -verbose -sigalg SHA256withRSA -digestalg SHA-256 \
  -keystore ~/kael-os.keystore app-release-unsigned.apk kael-os-key
```

## 📤 Distribution

### Google Play Store:
1. Create Google Play developer account ($25 one-time)
2. Generate AAB (Android App Bundle):
   ```bash
   cd android && ./gradlew bundleRelease
   ```
3. Upload to Play Store
4. Set up staged rollout (5% → 25% → 100%)

### Firebase App Distribution (Beta):
```bash
firebase appdistribution:distribute app-release.apk \
  --app 1:623895641528:android:abc123... \
  --testers-file testers.txt
```

### F-Droid (Open Source):
1. Fork F-Droid repo
2. Add metadata.json
3. Submit to F-Droid review

### Sideload (Direct download):
- Host APK on GitHub releases
- Users download + install manually
- Update via in-app update mechanism

## 🔄 Shared Code Strategy

### Services (shared logic):
```typescript
// src/services/authService.ts (shared between React/Dioxus)
export interface User {
  uid: string;
  email: string;
  idToken: string;
}

export async function loginWithGoogle(idToken: string): Promise<User> {
  // Shared auth logic
}
```

### Types (shared):
```typescript
// src/types/index.ts
export interface Message {
  id: string;
  sender: string;
  content: string;
  timestamp: Date;
}

export interface ChatSession {
  id: string;
  messages: Message[];
}
```

## 🚀 Native Modules (Kotlin)

### PTY Module (Limited terminal):
```kotlin
// android/app/src/main/java/com/leetheorc/kaelosmobile/modules/PTYModule.kt
package com.leetheorc.kaelosmobile.modules

import com.facebook.react.bridge.*

class PTYModule(reactContext: ReactApplicationContext) : ReactContextBaseJavaModule(reactContext) {
    override fun getName() = "PTYModule"

    @ReactMethod
    fun executeCommand(command: String, promise: Promise) {
        try {
            val process = Runtime.getRuntime().exec(arrayOf("sh", "-c", command))
            val output = process.inputStream.bufferedReader().readText()
            promise.resolve(output)
        } catch (e: Exception) {
            promise.reject("EXEC_ERROR", e.message)
        }
    }
}
```

### Secure Storage Module:
```kotlin
// android/app/src/main/java/com/leetheorc/kaelosmobile/modules/SecureStorageModule.kt
import android.security.keystore.KeyGenParameterSpec
import javax.crypto.Cipher
import javax.crypto.KeyGenerator

class SecureStorageModule(reactContext: ReactApplicationContext) : ReactContextBaseJavaModule(reactContext) {
    override fun getName() = "SecureStorageModule"

    @ReactMethod
    fun storeEncrypted(key: String, value: String, promise: Promise) {
        // Use Android Keystore for secure storage
        try {
            // Implementation using Android Keystore
            promise.resolve("Stored successfully")
        } catch (e: Exception) {
            promise.reject("STORE_ERROR", e.message)
        }
    }
}
```

## 🧪 Testing Strategy

### Unit Tests:
```typescript
// src/services/__tests__/authService.test.ts
import { loginWithGoogle } from '../authService';

describe('Auth Service', () => {
  test('should login with valid Google token', async () => {
    const user = await loginWithGoogle('valid_token');
    expect(user.uid).toBeDefined();
  });
});
```

### E2E Tests:
- Use Detox for automated testing
- Test chat message flow
- Test OAuth flow
- Test offline sync

## 🔄 Update Path (Android)

1. User opens app → checks `yourdomain.com/api/check?platform=android`
2. If new version available:
   - Download APK from GitHub releases
   - Verify SHA256
   - Request install permission
   - Install in background via WorkManager
3. App restarts with new version

## 📋 Android Specific Considerations

1. **Permissions** (AndroidManifest.xml):
   ```xml
   <uses-permission android:name="android.permission.INTERNET" />
   <uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" />
   <uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
   <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
   ```

2. **Background execution**: Firebase Cloud Messaging for push notifications

3. **Data storage**: SharedPreferences + EncryptedSharedPreferences

4. **Minimum API level**: 24 (Android 7.0+, released 2016)

## 🎯 Roadmap

- **v0.1.0** (Q1 2025): MVP with chat + settings + basic terminal
- **v0.2.0** (Q2 2025): GPG/SSL management + auto-update
- **v0.3.0** (Q3 2025): Full terminal support + offline sync
- **v1.0.0** (Q4 2025): Production release on Google Play + F-Droid
- **iOS** (2026): Port to Swift/SwiftUI if demand exists

## 🤝 Contribution Path

1. Start with ChatScreen (most familiar)
2. Move to SettingsScreen (OAuth flow)
3. Implement Firebase integration
4. Add security features (GPG/SSL)
5. Build terminal last (most complex)
