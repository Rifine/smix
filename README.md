根据用户提供的权重值混合三个RGBA蒙版，并导出多分辨率PNG图像。

## 为什么开发这个工具？
我开发了`smix`，旨在自动化《Slay the Spire》模组开发中繁琐的部分：生成游戏所需的彩色多分辨率卡牌背景纹理。
以往，为了制作各种卡牌背景，我需要手动制作。
现在，我只需将三个官方卡牌背景图像放入蒙版文件夹，调整几个权重值，`smix`就能自动生成所有所需分辨率的清晰PNG图像。
非常适合批量生成各种颜色的卡牌背景。尽管没什么很大的用处！

## 功能概述
根据用户提供的RGB权重值（0~1）混合三个RGBA蒙版。
一次性导出多种分辨率的图像（1倍、2倍、0.5倍等）。
支持选择不同的图像缩放算法（最近邻、双线性、Lanczos3等）。

# 快速入门
```bash
# 1. 克隆此仓库
git clone https://github.com/Rifine/smix.git
cd smix

# 2. 构建
cargo install --path ./cli

# 3. 提取或复制蒙版图像（r.png、g.png、b.png），确保目录结构如下：
#   mask/r.png
#   mask/g.png
#   mask/b.png
# 所有图像必须尺寸相同。

# 4. 运行
smix 1.0 0.15 0.04 \
--mask-directories mask \
--scale 1 2 0.5 \
--filter lanczos3 \
--output ./results
```