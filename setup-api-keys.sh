#!/usr/bin/env bash
# 🔑 Quick API Key Setup Script for Kael-OS
# Sets up your AI provider keys for both app usage and VSCode delegation

set -e

echo "🔑 Kael-OS API Key Setup"
echo "========================="
echo ""

# Check if running in Kael-OS directory
if [ ! -f "src-tauri/Cargo.toml" ]; then
    echo "❌ Error: Run this script from Kael-OS-AI directory"
    exit 1
fi

# Function to prompt for key
prompt_key() {
    local provider=$1
    local var_name=$2
    local current_val="${!var_name}"
    
    echo ""
    echo "📌 $provider"
    if [ -n "$current_val" ]; then
        echo "   Current: ${current_val:0:20}...${current_val: -4}"
        read -p "   Update? (y/N): " update
        if [[ ! $update =~ ^[Yy]$ ]]; then
            return
        fi
    fi
    
    read -sp "   Enter API key: " key
    echo ""
    
    if [ -n "$key" ]; then
        export $var_name="$key"
        echo "export $var_name=\"$key\"" >> ~/.kael_api_keys
        echo "   ✅ Saved!"
    fi
}

# Create/update API keys file
touch ~/.kael_api_keys
chmod 600 ~/.kael_api_keys

# Source existing keys
if [ -f ~/.kael_api_keys ]; then
    source ~/.kael_api_keys
fi

echo "Enter your API keys (press Enter to skip):"
echo ""

# Prompt for each provider
prompt_key "Google Gemini" "GEMINI_API_KEY"
prompt_key "Minstrel AI" "MINSTREL_API_KEY"
prompt_key "Mistral AI" "MISTRAL_API_KEY"
prompt_key "GitHub Copilot" "GITHUB_COPILOT_KEY"

echo ""
echo "🎯 Setup Complete!"
echo ""
echo "Keys saved to: ~/.kael_api_keys"
echo ""
echo "To use in current shell:"
echo "  source ~/.kael_api_keys"
echo ""
echo "To use in Kael-OS app:"
echo "  1. Launch: cargo run"
echo "  2. Settings ⚙️ → AI Providers"
echo "  3. Keys will auto-load from ~/.kael_api_keys"
echo ""
echo "To make permanent, add to your shell profile:"
echo "  echo 'source ~/.kael_api_keys' >> ~/.bashrc  # or ~/.zshrc"
echo ""

# Test keys
echo "🧪 Testing API keys..."
echo ""

if [ -n "$GEMINI_API_KEY" ]; then
    echo "✅ Gemini: ${GEMINI_API_KEY:0:20}...${GEMINI_API_KEY: -4}"
else
    echo "⚠️  Gemini: Not set"
fi

if [ -n "$MINSTREL_API_KEY" ]; then
    echo "✅ Minstrel: ${MINSTREL_API_KEY:0:20}...${MINSTREL_API_KEY: -4}"
else
    echo "⚠️  Minstrel: Not set"
fi

if [ -n "$MISTRAL_API_KEY" ]; then
    echo "✅ Mistral: ${MISTRAL_API_KEY:0:20}...${MISTRAL_API_KEY: -4}"
else
    echo "⚠️  Mistral: Not set"
fi

echo ""
echo "🚀 Ready to use! Run: cargo run"
