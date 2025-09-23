Blend three RGBA masks by user-supplied weights and export multi-scale PNGs.

## Why?
I wrote `smix` to automate the boring part of mod-making for **_`Slay the Spire`_**: creating the coloured, multi-resolution **`card-background textures`** the game expects.
Instead of hand-painting dozens of variants, I drop three `official card-back images` into the mask folder, tweak a few weights, and let the tool spit out crisp PNGs at every scale the engine needs.
Perfect for batch-producing glossy green Skill backs, fiery red Attack backs, or any custom colour you can dream up.

## What it does
Blends three RGBA masks  with user-supplied RGB weights (0~1)
Exports multiple resolutions in one run (1x, 2x, 0.5x ...)
Lets you pick the resize filter (nearest, bilinear, lanczos3 ...)

# Quick Start
```bash
# 1. clone this repository
git clone ...
cd smix

# 2. build
cargo install --path ./cli

# 3. extract or copy the mask images (r.png, g.png, b.png) so you have:
#   mask/r.png
#   mask/g.png
#   mask/b.png
# All must be the same size.

# 4. run
smix 1.0 0.15 0.04 \
    --mask-directories mask \
    --scale 1 2 0.5 \
    --filter lanczos3 \
    --output ./results
```

## Contributing
Pull requests welcome — especially presets for other Sts mods!