#!/bin/bash

# Firebase Configuration Validator
# This script checks if your Firebase setup is complete

echo "🔥 Firebase Setup Validator for Kael OS"
echo "======================================="
echo ""

# Check if .env.local exists
if [ ! -f .env.local ]; then
    echo "❌ .env.local file not found"
    echo "   Please create .env.local with Firebase config"
    echo "   Copy from .env.example and fill in your values"
    exit 1
fi

echo "✅ .env.local file found"
echo ""

# Load env file
set -a
source .env.local
set +a

# Check each required variable
echo "Checking Firebase Environment Variables:"
echo "----------------------------------------"

required_vars=(
    "VITE_FIREBASE_API_KEY"
    "VITE_FIREBASE_AUTH_DOMAIN"
    "VITE_FIREBASE_PROJECT_ID"
    "VITE_FIREBASE_STORAGE_BUCKET"
    "VITE_FIREBASE_MESSAGING_SENDER_ID"
    "VITE_FIREBASE_APP_ID"
)

missing_vars=0

for var in "${required_vars[@]}"; do
    value="${!var}"
    if [ -z "$value" ]; then
        echo "❌ Missing: $var"
        missing_vars=$((missing_vars + 1))
    else
        # Show first 10 and last 10 chars to verify without exposing full key
        length=${#value}
        if [ $length -gt 20 ]; then
            display="${value:0:10}...${value:(-10)}"
        else
            display="[configured]"
        fi
        echo "✅ $var: $display"
    fi
done

echo ""
echo "Summary:"
echo "--------"

if [ $missing_vars -eq 0 ]; then
    echo "✅ All Firebase environment variables are configured!"
    echo ""
    echo "Next steps:"
    echo "1. Run: npm run dev"
    echo "2. Visit: http://localhost:5173"
    echo "3. Test OAuth login with Google or GitHub"
    echo ""
else
    echo "❌ Missing $missing_vars environment variable(s)"
    echo ""
    echo "To fix:"
    echo "1. Open .env.local"
    echo "2. Add missing variables from Firebase Console"
    echo "3. Firebase Console → Project Settings → Web app config"
    echo ""
fi

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "⚠️  node_modules not found"
    echo "   Run: npm install"
fi

# Check if Firebase is installed
if grep -q '"firebase"' package.json; then
    echo "✅ Firebase package is in package.json"
else
    echo "⚠️  Firebase package not in package.json"
    echo "   Run: npm install firebase"
fi

echo ""
echo "All checks complete! 🎉"
