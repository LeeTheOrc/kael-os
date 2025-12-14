#!/bin/bash

echo "🔧 Testing Hardware Detection & Storage-Aware Optimization"
echo "==========================================================="
echo

# Test 1: Storage Type Detection
echo "1️⃣ Storage Type Detection:"
echo "   Detecting storage type (SSD/HDD/NVMe)..."

if lsblk -d -no NAME,ROTA &>/dev/null; then
    echo "   ✅ lsblk available - can detect storage type"
    lsblk -d -no NAME,ROTA | head -3
else
    echo "   ⚠️ lsblk not available"
fi

if [ -r /sys/block/sda/queue/rotational ]; then
    ROTA=$(cat /sys/block/sda/queue/rotational)
    if [ "$ROTA" = "0" ]; then
        echo "   ✅ Primary disk is SSD (rotational=0)"
    else
        echo "   ✅ Primary disk is HDD (rotational=1)"
    fi
else
    echo "   ℹ️ Could not read rotational flag"
fi

echo

# Test 2: CPU Core Detection
echo "2️⃣ CPU Core Detection:"
echo "   Detecting available CPU cores..."

if command -v nproc &>/dev/null; then
    CORES=$(nproc)
    echo "   ✅ nproc found: $CORES cores"
fi

if [ -r /proc/cpuinfo ]; then
    CORES_CPUINFO=$(grep -c "^processor" /proc/cpuinfo)
    echo "   ✅ /proc/cpuinfo: $CORES_CPUINFO cores"
fi

echo

# Test 3: Package Manager Detection
echo "3️⃣ Package Manager Detection:"
if command -v paru &>/dev/null; then
    echo "   ✅ paru found (preferred)"
elif command -v yay &>/dev/null; then
    echo "   ✅ yay found (fallback)"
else
    echo "   ℹ️ Neither paru nor yay found (no AUR helper)"
fi

echo

# Test 4: WiFi Interface Detection
echo "4️⃣ Network Interface Detection:"
echo "   Looking for wireless interfaces..."
if command -v ip &>/dev/null; then
    ip link show | grep -E "wlp|wlan|iwl" | head -3
    echo "   ✅ Found wireless interfaces using 'ip link show'"
else
    echo "   ⚠️ 'ip' command not available"
fi

echo

# Test 5: GPU Driver Detection
echo "5️⃣ GPU Driver Detection:"
if command -v lspci &>/dev/null; then
    if lspci | grep -i nvidia &>/dev/null; then
        echo "   ✅ NVIDIA GPU detected"
    elif lspci | grep -i amd &>/dev/null; then
        echo "   ✅ AMD GPU detected"
    elif lspci | grep -i intel &>/dev/null; then
        echo "   ✅ Intel GPU detected"
    else
        echo "   ℹ️ Generic/unknown GPU"
    fi
else
    echo "   ⚠️ lspci not available (install pciutils)"
fi

echo

# Test 6: Optimization Suggestions
echo "6️⃣ System-Aware Optimization Recommendations:"
echo

CORES=$(nproc 2>/dev/null || echo "unknown")
if [ "$CORES" != "unknown" ] && [ "$CORES" -gt 1 ]; then
    echo "   📊 Multi-core detected ($CORES cores):"
    echo "   → Cargo/Make commands will use: -j$CORES"
    echo "   → Example: cargo build -j$CORES"
    echo "   → Example: make -j$CORES"
fi

echo

if [ -r /sys/block/sda/queue/rotational ]; then
    ROTA=$(cat /sys/block/sda/queue/rotational)
    if [ "$ROTA" = "0" ]; then
        echo "   💾 SSD Detected:"
        echo "   → Scheduler optimizations: 'mq-deadline' or 'none'"
        echo "   → Mount options: defaults,noatime,discard"
    else
        echo "   💾 HDD Detected:"
        echo "   → Recommended scheduler: 'bfq'"
        echo "   → Avoid aggressive background tasks"
    fi
fi

echo

if lsblk | grep -i nvme &>/dev/null; then
    echo "   ⚡ NVMe Drive Detected:"
    echo "   → Scheduler optimization: set to 'none'"
    echo "   → No need for rotational parameters"
fi

echo

echo "✅ Hardware Detection Test Complete!"
echo "==========================================================="
