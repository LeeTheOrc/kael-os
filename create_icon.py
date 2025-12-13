#!/usr/bin/env python3
"""
Create Kael OS app icon with avatar styling
"""
from PIL import Image, ImageDraw
import math

# Create a 256x256 image with gradient background
size = 256
img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Create radial gradient background (dark purple to darker purple)
# Kael OS theme colors
for y in range(size):
    for x in range(size):
        # Distance from center
        dx = x - size // 2
        dy = y - size // 2
        distance = math.sqrt(dx * dx + dy * dy) / (size / 2)
        distance = min(1.0, distance)
        
        # Gradient from #1c162b to #0b0816
        r = int(28 * (1 - distance) + 11 * distance)
        g = int(22 * (1 - distance) + 8 * distance)
        b = int(43 * (1 - distance) + 22 * distance)
        
        img.putpixel((x, y), (r, g, b, 255))

# Draw outer circle border (accent color)
draw.ellipse([10, 10, size-10, size-10], outline=(224, 64, 251, 255), width=4)  # #e040fb

# Draw inner circle (lighter shade)
inner_margin = 20
draw.ellipse([inner_margin, inner_margin, size-inner_margin, size-inner_margin], 
             outline=(122, 238, 190, 200), width=2)  # #7aebbe with transparency

# Draw stylized "K" letter in the center for Kael
# Using colors from the theme
text_color = (255, 204, 0, 255)  # #ffcc00

# Draw a simple stylized K shape
# Vertical bar
draw.rectangle([100, 70, 115, 190], fill=text_color)

# Upper diagonal
points_upper = [(115, 70), (160, 110), (145, 125)]
draw.polygon(points_upper, fill=text_color)

# Lower diagonal  
points_lower = [(115, 130), (160, 180), (145, 195)]
draw.polygon(points_lower, fill=text_color)

# Save all required sizes
sizes = [
    ('icon.png', 256),
    ('128x128@2x.png', 256),
    ('128x128.png', 128),
    ('32x32.png', 32),
]

icon_dir = '/home/leetheorc/Kael-os/kael-os/src-tauri/icons'

for filename, size_px in sizes:
    if size_px == 256:
        # Use the full resolution image
        resized = img
    else:
        # Resize for smaller sizes
        resized = img.resize((size_px, size_px), Image.Resampling.LANCZOS)
    
    resized.save(f'{icon_dir}/{filename}')
    print(f'Created {icon_dir}/{filename} ({size_px}x{size_px})')

print('\nIcon files created successfully!')
