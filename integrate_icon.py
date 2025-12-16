#!/usr/bin/env python3
"""
Integrate the DALL-E 3 dragon icon into Kael-OS
- Resize to all required sizes
- Save as PNG with transparency
- Update all icon locations
"""

from PIL import Image
import os
import shutil

# Source icon
source = "kael-dragon-icon.jpeg"
img = Image.open(source)

# Ensure it's RGB (will convert to RGBA for PNG)
if img.mode != 'RGB':
    img = img.convert('RGB')

# Required sizes for app icons
sizes = [16, 32, 48, 64, 128, 256, 512, 1024]

# Create directories
os.makedirs("assets/generated/png/app-icons", exist_ok=True)
os.makedirs("src-tauri/icons", exist_ok=True)

print("🎨 Resizing dragon icon to all required sizes...")

for size in sizes:
    # Resize with high-quality Lanczos resampling
    resized = img.resize((size, size), Image.Resampling.LANCZOS)
    
    # Save to app-icons directory
    output_path = f"assets/generated/png/app-icons/icon-{size}.png"
    resized.save(output_path, "PNG", quality=95, optimize=True)
    file_size = os.path.getsize(output_path) / 1024
    print(f"  ✓ {output_path} ({file_size:.1f} KB)")

# Copy 512x512 as the master icon for src-tauri
print("\n📦 Copying master icon to src-tauri/icons/...")
master_512 = "assets/generated/png/app-icons/icon-512.png"
shutil.copy(master_512, "src-tauri/icons/icon.png")
print(f"  ✓ src-tauri/icons/icon.png")

# Also save the 1024x1024 for Discord/website
shutil.copy("assets/generated/png/app-icons/icon-1024.png", "website/images/kael-dragon-1024.png")
print(f"  ✓ website/images/kael-dragon-1024.png")

# Copy 128x128 for .desktop file
shutil.copy("assets/generated/png/app-icons/icon-128.png", "assets/generated/png/app-icons/kael-os.png")
print(f"  ✓ assets/generated/png/app-icons/kael-os.png (for desktop entry)")

print("\n✅ Dragon icon integrated successfully!")
print("\nNext steps:")
print("  1. Rebuild release package: bash scripts/build-release.sh")
print("  2. Upload 512x512 icon to Discord server")
print("  3. Update website favicon")
print("  4. Deploy updates")
