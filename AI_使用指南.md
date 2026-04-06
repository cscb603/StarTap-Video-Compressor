# 🎯 AI 工具使用指南 / AI Tool Integration Guide

## 📋 概述 / Overview
此 CLI 工具专为 AI 代理（如 Claude、GPT-4 等）调用而设计，支持 JSON 格式输出，便于程序化解析。

## 🚀 快速开始 / Quick Start

### 1. 验证工具是否可用 / Verify Tool Availability
```bash
fast-video-cli.exe --help
```

**预期输出 / Expected Output**:
```
星TAP 视频压缩工具 CLI 版
Usage: fast-video-cli.exe [OPTIONS] [FILE/DIR]...
...
```

### 2. 基本压缩命令 / Basic Compression
```bash
fast-video-cli.exe --input "path/to/video.mp4" --output-dir "path/to/output"
```

### 3. JSON 输出模式（推荐 AI 调用）/ JSON Output Mode (Recommended for AI)
```bash
fast-video-cli.exe --input "path/to/video.mp4" --output-dir "path/to/output" --json
```

---

## 🎛️ 档位预设 / Preset Profiles

### 5 个预设档位，一键配置 / 5 Preset Profiles, One-Click Configuration

| 档位 / Preset | 尺寸 / Size | 画质 / Quality | 速度 / Speed | 说明 / Description |
|---------------|-------------|---------------|-------------|-------------------|
| 原始大小 / Original Size | 0 | Custom | Custom | 不缩放 / No scaling |
| 临时分享 / Quick Share | 1080 | 32 | 10 | 极致压缩 / Extreme compression |
| 推荐平衡 / Recommended | 1080 | 24 | 8 | 日常使用 / Daily use |
| 2K 高清 / 2K HD | 1440 | 22 | 8 | 2K 质量 / 2K quality |
| 4K 原画 / 4K Original | 2160 | 20 | 8 | 4K 质量 / 4K quality |

---

## 📊 编码器选择 / Encoder Selection

| 编码器 | 推荐场景 | 说明 |
|--------|----------|------|
| `svt-av1` (默认) | 日常压缩 | 平衡压缩率和速度 |
| `rav1e` | 极致压缩 | 最高压缩率，较慢 |
| `x265` | 兼容性优先 | 广泛设备支持 |
| `hevc-nvenc` | NVIDIA 用户 | 极快的硬件加速 |
| `hevc-videotoolbox` | Mac 用户 | Apple 硬件加速 |

---

## 🎛️ 参数详解 / Parameter Details

### 输入输出 / Input/Output
| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| `-i, --input` | String | ✅ | - | 输入文件或目录 |
| `--output-dir` | String | ❌ | (与输入同目录) | 输出目录 |

### 视频编码 / Video Encoding
| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| `--encoder` | String | ❌ | `svt-av1` | 编码器类型 |
| `--quality` | Number | ❌ | `24` | 画质/CRF |
| `--speed-preset` | Number | ❌ | `8` | 速度预设 (0-13) |
| `--max-height` | Number | ❌ | `1080` | 最大高度 (0=原始大小) |
| `--quick-share` | Flag | ❌ | false | 临时分享模式 (极致压缩) |

### 音频编码 / Audio Encoding
| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| `--audio-codec` | String | ❌ | `opus` | 音频编码器 |
| `--audio-bitrate` | String | ❌ | `128k` | 音频比特率 |

### 其他 / Other
| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| `--concurrency` | Number | ❌ | `2` | 并发处理数 |
| `--json` | Flag | ❌ | false | 输出 JSON 格式 |
| `-q, --quiet` | Flag | ❌ | false | 静默模式 |
| `-h, --help` | Flag | ❌ | false | 显示帮助 |

---

## 📝 完整示例 / Complete Examples

### 示例 1：日常压缩（SVT-AV1，推荐）/ Example 1: Daily Compression (SVT-AV1, Recommended)
```bash
fast-video-cli.exe --input "videos" --output-dir "compressed" --encoder svt-av1 --quality 24 --speed-preset 8 --max-height 1080
```

### 示例 2：临时分享模式（极速压缩）/ Example 2: Quick Share Mode (Extreme Compression)
```bash
fast-video-cli.exe --input "videos" --output-dir "compressed" --quick-share --max-height 1080 --quality 32 --speed-preset 10
```

### 示例 3：原始大小（不缩放）/ Example 3: Original Size (No Scaling)
```bash
fast-video-cli.exe --input "videos" --output-dir "compressed" --max-height 0
```

### 示例 4：NVIDIA 硬件加速（极速）/ Example 4: NVIDIA Hardware Acceleration (Very Fast)
```bash
fast-video-cli.exe --input "videos" --output-dir "compressed" --encoder hevc-nvenc --quality 80 --max-height 1080
```

### 示例 5：AI 调用（JSON 输出）/ Example 5: AI Integration (JSON Output)
```bash
fast-video-cli.exe --input "input.mp4" --output-dir "output" --encoder svt-av1 --quality 24 --json
```

---

## 📊 JSON 输出格式 / JSON Output Format

```json
{
  "success": true,
  "total": 5,
  "completed": 5,
  "failed": 0,
  "original_size": 524288000,
  "compressed_size": 104857600,
  "compression_ratio": 0.8,
  "results": [
    {
      "input": "video1.mp4",
      "output": "video1_s.mp4",
      "success": true,
      "original_size": 104857600,
      "compressed_size": 20971520,
      "compression_ratio": 0.8
    }
  ]
}
```

---

## ⚠️ 注意事项 / Notes

1. **路径处理**：如果路径包含空格，请用双引号包裹
2. **并发数**：建议根据 CPU 核心数设置，通常 2-4 为宜
3. **硬件加速**：NVENC 需要 NVIDIA 显卡，VideoToolbox 需要 Mac
4. **错误处理**：工具会自动跳过失败文件并继续处理
5. **竖屏视频**：工具会自动识别竖屏视频并限制宽度而非高度，保持画面比例
6. **文件名后缀**：所有压缩后的文件统一使用 `_s` 后缀

---

**此工具专为 AI 代理调用设计，所有参数均可程序化配置！**
