# 🧪 Manual Test Guide - Kael-OS v0.3.0 Beta

**Build Status**: ✅ SUCCESSFUL  
**Binary**: `/home/leetheorc/Kael-os/Kael-OS-AI/target/release/kael-os`  
**Date**: December 14, 2025

---

## 🚀 To Launch the App

```bash
/home/leetheorc/Kael-os/Kael-OS-AI/target/release/kael-os
```

Or from the source directory:
```bash
cd /home/leetheorc/Kael-os/Kael-OS-AI/src-tauri
cargo run --release
```

---

## 🔐 TEST 1: Key Storage & Encryption

### What to Test
1. **First Launch**: Add your API keys (Ollama URL, Mistral key, etc.)
2. **Close App**: Completely exit the application
3. **Relaunch**: Open app again
4. **Verify Keys**: Check if keys are remembered (auto-filled)

### What You Should See
```
✅ Keys are filled in automatically (encrypted storage working)
✅ No plaintext keys visible anywhere
✅ Decryption successful on app restart
```

### If Keys Missing
```
⚠️ Check: ~/.config/kael-os/keys.db exists?
⚠️ Check: Encryption test: ./test_key_storage.sh
⚠️ Report: Keys not persisting across restarts
```

---

## 💬 TEST 2: Local AI (Should Handle Locally)

### Test Command #1: "How do I install Discord?"

```
Input:  "How do I install Discord?"
Expected Behavior:
  ✅ Shows: "Handling locally with Ollama"
  ✅ Shows: "Smart context detected your system"
  ✅ Shows: "Detected: paru, 16 cores, NVMe SSD, wlan0, fish shell"
  ✅ Response: Comes from local Ollama (should be quick)
  ✅ No network request made
```

### Why This Should Be Local
- Installation help is in Ollama's knowledge base
- Doesn't require specialized reasoning
- Can be answered with shell commands
- Fast local response expected

### What to Look For
```
Chat shows: "via Ollama" or "Provider: Ollama"
Response time: <5 seconds (local)
No cloud processing indicator
```

---

## 🌐 TEST 3: Cloud AI (Should Escalate)

### Test Command #2: "Write a Rust function that sorts arrays efficiently"

```
Input:  "Write a Rust function that sorts arrays efficiently"
Expected Behavior:
  ✅ Shows: "Escalating to cloud AI"
  ✅ Shows: "This requires deeper reasoning - using Mistral"
  ✅ Shows: Smart context detected (hardware info)
  ✅ Shows: Any command rewrites if applicable
  ✅ Response: Comes from Mistral (should include explanation + code)
  ✅ Network request made (visible in chat)
```

### Why This Should Go to Cloud
- Requires deep algorithmic reasoning
- Complex code generation task
- Not a simple help/troubleshoot question
- Mistral excels at code generation

### What to Look For
```
Chat shows: "via Mistral" or "Provider: Mistral"
Response time: 2-5 seconds (network + processing)
Shows complete Rust implementation
Includes efficiency explanation
```

---

## 🎯 Smart Reformatting Tests

### Test 3A: Network Interface Auto-Fix

**Input**: 
```
"ip link set wlan0 up"
```

**Expected**:
```
Shows correction note:
  "Updated network interface: wlan0 → wlp4s0 (your actual interface)"
Actual command sent:
  "ip link set wlp4s0 up"
```

### Test 3B: Package Manager Auto-Fix

**Input**:
```
"yay -S nginx"
```

**Expected**:
```
Shows correction note:
  "Changed yay → paru (your preferred AUR helper)"
Actual command sent:
  "paru -S nginx"
```

### Test 3C: Shell Syntax Auto-Fix

**Input**:
```
"how to export PATH in fish shell?"
```

**Expected**:
```
Shows correction note:
  "Detected fish shell - bash export syntax won't work"
Response includes:
  "set -x PATH /new/path" (fish syntax)
  Not: "export PATH=/new/path" (bash syntax)
```

---

## ⚡ Performance Tests

### Load Time
```
Measure: Time from app launch to ready
Expected: < 2 seconds to chat screen
Actual: _____ seconds
```

### First Message Latency
```
Measure: Time from typing first message to response
Expected: ~50ms (hardware detection) + response time
Actual: _____ ms
```

### Subsequent Messages
```
Measure: Time for second message (cached detection)
Expected: <1ms overhead + response time
Actual: _____ ms
```

### Hardware Detection
```
Should see in logs/chat:
  ✅ "Detected: 16 cores"
  ✅ "Storage: nvme"
  ✅ "GPU: nvidia"
  ✅ "Network: wlan0"
  ✅ "Shell: fish"
```

---

## 🔌 Provider Switching Test

### Test 4: Try Next Provider Button

**Scenario 1: Ollama Down**
```
1. Make sure Ollama is NOT running
2. Type: "how do i install something?"
3. See: Ollama error
4. Click: "Try Next Provider" button
5. Expected: Automatically switches to Mistral
6. Shows: Response from cloud AI
```

**Scenario 2: Provider Preference**
```
1. Ask: "explain recursion"
2. See: "via Ollama" or "via Mistral"
3. Click: "Try Next Provider"
4. See: Same question answered by different AI
5. Provider preference should be remembered
```

---

## 📊 Ideas Panel Test

### Test 5: Brainstorm Feature

```
1. Open: Ideas panel (if accessible)
2. Look for: Auto-populated brainstorm ideas
3. Filter by: Category (Features, UI/UX, Optimize, Integrate)
4. Click: Star icon on an idea
5. Expected:
   ✅ Star fills in
   ✅ Idea persists in panel
   ✅ On relaunch: Starred ideas still starred
```

---

## 🔑 Complete Test Sequence

### Phase 1: Launch & Setup (5 min)
- [ ] Launch app: `/home/leetheorc/Kael-os/Kael-OS-AI/target/release/kael-os`
- [ ] Add API keys if needed
- [ ] Verify hardware detection shows in chat
- [ ] Close and reopen to test key persistence

### Phase 2: Local AI Test (2 min)
- [ ] Type: "How do I install Discord?"
- [ ] Verify: Response is from Ollama
- [ ] Check: No "correction notes" needed (already valid)
- [ ] Record: Response time

### Phase 3: Cloud AI Test (3 min)
- [ ] Type: "Write a Rust function that sorts arrays efficiently"
- [ ] Verify: Escalates to cloud (Mistral)
- [ ] Check: Shows smart context detection
- [ ] Record: Response time and code quality

### Phase 4: Smart Reformatting (3 min)
- [ ] Type command with wrong interface name
- [ ] Verify: Correction note shows
- [ ] Type command with yay instead of paru
- [ ] Verify: Package manager fix applied
- [ ] Type shell-related question
- [ ] Verify: Shell syntax is correct

### Phase 5: Robustness (3 min)
- [ ] Close/reopen app - keys still there?
- [ ] Try provider switch - works?
- [ ] Check logs - any errors?
- [ ] Verify: Clean shutdown

**Total Time**: ~15 minutes

---

## 🐛 If Something Goes Wrong

### Keys Not Saving
```
1. Check: File exists at ~/.config/kael-os/keys.db
2. Run: ./test_key_storage.sh
3. Verify: Encryption is working
4. Report: Error message and system info
```

### Local AI Not Working
```
1. Check: Is Ollama running?
   ollama serve
2. Check: Is model available?
   ollama list
3. Check: Is localhost:11434 accessible?
   curl http://localhost:11434/api/tags
4. Report: Error messages to us
```

### Cloud AI Not Working
```
1. Check: API keys correct?
2. Check: Internet connection?
3. Check: API rate limits?
4. Try: Test with simple question first
5. Report: Exact error message
```

### Hardware Detection Wrong
```
1. Run: ./test_hardware_detection.sh
2. Check: What was actually detected
3. Run: Manually check (lspci, nproc, etc.)
4. Report: Difference between detected vs actual
```

---

## 📋 Results Template

When you're done testing, share:

```
TEST RESULTS - Kael-OS v0.3.0 Beta
====================================

System Info:
- OS: Arch Linux
- CPU: 16 cores
- Storage: NVMe SSD
- GPU: NVIDIA
- Shell: fish

KEY STORAGE:
- Keys saved on first run? [YES/NO]
- Keys remembered after restart? [YES/NO]
- Any errors? [NONE / describe]

LOCAL AI TEST:
- Command: "How do I install Discord?"
- Handled by: [Ollama / Mistral / Neither]
- Response time: ___ seconds
- Quality: [Good / OK / Poor]

CLOUD AI TEST:
- Command: "Write a Rust function that sorts arrays"
- Escalated correctly? [YES / NO]
- Response time: ___ seconds
- Code quality: [Good / OK / Poor]

SMART REFORMATTING:
- Interface fix: [Worked / Failed]
- Package manager fix: [Worked / Failed]
- Shell syntax fix: [Worked / Failed]

ISSUES FOUND:
[List any bugs, crashes, or weird behavior]

FEEDBACK:
[What was good? What could improve?]
```

---

## 🚀 Next Steps After Testing

1. **Share Results** → Tell us what you found
2. **Report Issues** → Any crashes or wrong behavior
3. **Suggest Improvements** → What would help?
4. **Beta Feedback** → Ready for v0.3.1?

---

## 📞 Questions?

If anything is confusing or not working:
1. Check the test command output
2. Run the hardware detection script
3. Report the exact error message
4. Include your system specs

**Happy testing!** 🎉

---

*Kael-OS v0.3.0-beta*  
*Test Guide Generated: December 14, 2025*
