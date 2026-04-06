# 🚀 星TAP 极简视频压缩 V3

## 📖 中文说明

### 快速开始
1. **GUI 模式（推荐给普通用户）**：双击 `星TAP视频压缩v3.exe`
2. **CLI 模式（推荐给 AI 工具和技术用户）**：运行 `fast-video-cli.exe`

### GUI 使用说明
- 点击「添加文件」或「添加文件夹」选择视频
- 拖拽视频或文件夹到界面即可添加
- 选择「最大尺寸」档位（推荐 1080p 推荐平衡档）
- 点击「开始批量压缩」开始处理

### 档位说明（5 个预设档位）

| 档位 | 尺寸 | 画质 | 速度 | 说明 |
|------|------|------|------|------|
| 原始大小 - 不缩放 | 原尺寸 | 自定义 | 自定义 | 保持原分辨率 |
| 1080p (临时分享 · 极致压缩) | 1080p | CRF 32 | 10 | 适合微信/临时分享，文件极小 |
| 1080p (推荐 · 平衡) | 1080p | CRF 24 | 8 | 日常使用推荐 |
| 1440p (2K) - 高清 | 1440p | CRF 22 | 8 | 2K 高清质量 |
| 2160p (4K) - 原画 | 2160p | CRF 20 | 8 | 4K 原画画质 |

### CLI 使用说明

#### 基本用法
```bash
fast-video-cli.exe --input video.mp4 --output-dir ./output
```

#### 批量压缩整个文件夹
```bash
fast-video-cli.exe --input ./videos --output-dir ./output
```

#### 临时分享模式（极速压缩）
```bash
fast-video-cli.exe --input video.mp4 --quick-share --max-height 1080 --quality 32 --speed-preset 10
```

#### 选择编码器
```bash
# SVT-AV1（推荐，压缩率高，速度快）
fast-video-cli.exe --input video.mp4 --encoder svt-av1

# rav1e（极致压缩率）
fast-video-cli.exe --input video.mp4 --encoder rav1e

# x265（兼容性最好）
fast-video-cli.exe --input video.mp4 --encoder x265

# HEVC NVENC（NVIDIA 显卡硬件加速，极快）
fast-video-cli.exe --input video.mp4 --encoder hevc-nvenc

# HEVC VideoToolbox（Apple 显卡硬件加速）
fast-video-cli.exe --input video.mp4 --encoder hevc-videotoolbox
```

#### 调整画质
```bash
# CRF 模式（适用于 SVT-AV1/rav1e/x265）
# 数值越小，画质越好，文件越大
fast-video-cli.exe --input video.mp4 --quality 24

# 质量模式（适用于 NVENC/VideoToolbox）
# 数值越大，画质越好
fast-video-cli.exe --input video.mp4 --encoder hevc-nvenc --quality 80
```

#### 调整速度
```bash
# 速度预设（0-13）
# 0 = 最慢，最佳压缩率
# 8 = 推荐（平衡）
# 13 = 最快
fast-video-cli.exe --input video.mp4 --speed-preset 8
```

#### 调整最大尺寸
```bash
# 原始大小（不缩放）
fast-video-cli.exe --input video.mp4 --max-height 0

# 1080p
fast-video-cli.exe --input video.mp4 --max-height 1080

# 2K (1440p)
fast-video-cli.exe --input video.mp4 --max-height 1440

# 4K (2160p)
fast-video-cli.exe --input video.mp4 --max-height 2160
```

#### JSON 输出模式（AI 工具调用用）
```bash
fast-video-cli.exe --input video.mp4 --json
```

#### 查看帮助
```bash
fast-video-cli.exe --help
```

---

## 📖 English Documentation

### Quick Start
1. **GUI Mode (Recommended for regular users)**: Double-click `星TAP视频压缩v3.exe`
2. **CLI Mode (Recommended for AI tools and technical users)**: Run `fast-video-cli.exe`

### GUI Instructions
- Click "Add Files" or "Add Folder" to select videos
- Drag and drop videos or folders onto the interface
- Select "Max Size" preset (recommended: 1080p balanced)
- Click "Start Batch Compression" to begin

### Preset Profiles (5 Options)

| Preset | Size | Quality | Speed | Description |
|--------|------|---------|-------|-------------|
| Original Size - No Scaling | Original | Custom | Custom | Keep original resolution |
| 1080p (Quick Share · Extreme Compression) | 1080p | CRF 32 | 10 | Perfect for WeChat/quick sharing, minimal file size |
| 1080p (Recommended · Balanced) | 1080p | CRF 24 | 8 | Recommended for daily use |
| 1440p (2K) - HD | 1440p | CRF 22 | 8 | 2K high definition quality |
| 2160p (4K) - Original | 2160p | CRF 20 | 8 | 4K original quality |

### CLI Instructions

#### Basic Usage
```bash
fast-video-cli.exe --input video.mp4 --output-dir ./output
```

#### Batch Compress Entire Folder
```bash
fast-video-cli.exe --input ./videos --output-dir ./output
```

#### Quick Share Mode (Extreme Compression)
```bash
fast-video-cli.exe --input video.mp4 --quick-share --max-height 1080 --quality 32 --speed-preset 10
```

#### Select Encoder
```bash
# SVT-AV1 (Recommended, high compression, fast)
fast-video-cli.exe --input video.mp4 --encoder svt-av1

# rav1e (Maximum compression)
fast-video-cli.exe --input video.mp4 --encoder rav1e

# x265 (Best compatibility)
fast-video-cli.exe --input video.mp4 --encoder x265

# HEVC NVENC (NVIDIA GPU hardware acceleration, very fast)
fast-video-cli.exe --input video.mp4 --encoder hevc-nvenc

# HEVC VideoToolbox (Apple GPU hardware acceleration)
fast-video-cli.exe --input video.mp4 --encoder hevc-videotoolbox
```

#### Adjust Quality
```bash
# CRF mode (for SVT-AV1/rav1e/x265)
# Lower = better quality, larger file
fast-video-cli.exe --input video.mp4 --quality 24

# Quality mode (for NVENC/VideoToolbox)
# Higher = better quality
fast-video-cli.exe --input video.mp4 --encoder hevc-nvenc --quality 80
```

#### Adjust Speed
```bash
# Speed preset (0-13)
# 0 = slowest, best compression
# 8 = recommended (balanced)
# 13 = fastest
fast-video-cli.exe --input video.mp4 --speed-preset 8
```

#### Adjust Maximum Size
```bash
# Original size (no scaling)
fast-video-cli.exe --input video.mp4 --max-height 0

# 1080p
fast-video-cli.exe --input video.mp4 --max-height 1080

# 2K (1440p)
fast-video-cli.exe --input video.mp4 --max-height 1440

# 4K (2160p)
fast-video-cli.exe --input video.mp4 --max-height 2160
```

#### JSON Output Mode (For AI Tool Integration)
```bash
fast-video-cli.exe --input video.mp4 --json
```

#### Show Help
```bash
fast-video-cli.exe --help
```

---

## 📦 文件说明 / File Description
- `星TAP视频压缩v3.exe` - GUI 版本 / GUI Version
- `fast-video-cli.exe` - CLI 版本 / CLI Version
- `ffmpeg.exe` - FFmpeg 编码器 / FFmpeg Encoder
- `ffprobe.exe` - FFprobe 视频信息检测工具 / FFprobe Video Info Tool

## ⚙️ 系统要求 / System Requirements
- Windows 10 或更高版本 / Windows 10 or later
- macOS 12 或更高版本 / macOS 12 or later
- NVIDIA 显卡（使用 NVENC 需要）/ NVIDIA GPU (for NVENC)
- Apple Silicon 或 Intel Mac（使用 VideoToolbox 需要）/ Apple Silicon or Intel Mac (for VideoToolbox)

---

**StarTAP Labs © 2026**
