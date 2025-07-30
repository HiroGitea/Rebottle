# 杜比视界 MKV 转 MP4 工具
![图片](assets/icons/icon.svg "Icon")

这是一个基于 Rust 和 Iced GUI 框架开发的工具，用于将杜比视界 Profile 5 的 MKV 文件转换为支持 QuickTime `dvh1` 格式的 MP4 文件。

## 功能特性

- 🎬 支持杜比视界 Profile 5 视频文件转换
- 🎵 自动提取音频轨道（支持杜比全景声）
- 📝 可选的字幕处理和集成
- ⚡ 多种帧率支持（23.976、24、25、29.970、60、59.940 fps）
- 🖥️ 现代化的图形用户界面
- 📊 实时处理进度显示
- 📝 详细的处理日志

## 系统要求

### 必需的外部工具

在使用此工具之前，请确保系统中已安装以下工具：

1. **MKVToolNix** - 用于提取 MKV 文件内容
   - 下载地址: https://mkvtoolnix.download/downloads.html
   - 确保 `mkvextract` 命令在 PATH 中可用

2. **mp4muxer** - 杜比官方工具，用于杜比视界封装
   - 这是处理杜比视界的核心工具
   - 需要从杜比获取或使用兼容工具

3. **FFmpeg** - 用于音频和字幕处理
   - 下载地址: https://ffmpeg.org/download.html
   - 确保 `ffmpeg` 命令在 PATH 中可用

4. **GPAC (MP4Box)** - 用于字幕集成（可选）
   - 下载地址: https://gpac.wp.imt.fr/downloads/
   - 仅在需要字幕功能时必需

### 编译要求

- Rust 1.70 或更高版本
- 支持的操作系统：
  - Windows 10/11
  - Linux (Ubuntu 18.04+, 其他现代发行版)
  - macOS 10.15+ (Intel 和 Apple Silicon)

### 跨平台编译说明

**注意：** 跨平台编译可能需要额外的链接器和工具链：

- **在 Windows 上编译 Linux 目标**：需要安装 WSL 或相应的交叉编译工具
- **在 Linux 上编译 Windows 目标**：需要安装 `mingw-w64`
- **在任何平台编译 macOS 目标**：需要 macOS SDK（通常需要在 macOS 上编译）

最简单的方法是在目标平台上直接编译。

## 编译和运行

### 快速开始

1. 克隆项目：
```bash
git clone <repository-url>
cd dv2macdv
```

2. 编译当前平台：
```bash
cargo build --release
```

3. 运行程序：
```bash
cargo run --release
```

### 跨平台编译

本项目支持以下平台的交叉编译：

- **Windows** (x86_64-pc-windows-msvc)
- **Linux** (x86_64-unknown-linux-gnu)  
- **macOS Intel** (x86_64-apple-darwin)
- **macOS Apple Silicon** (aarch64-apple-darwin)

#### Windows (PowerShell)

```powershell
# 构建所有平台
.\build.ps1 all -Release

# 构建特定平台
.\build.ps1 windows -Release
.\build.ps1 linux -Release
.\build.ps1 macos -Release
.\build.ps1 macos-arm -Release

# 调试构建
.\build.ps1 all
```

#### Linux/macOS (Bash)

```bash
# 给脚本执行权限
chmod +x build.sh

# 构建所有平台
./build.sh all --release

# 构建特定平台
./build.sh windows --release
./build.sh linux --release
./build.sh macos --release
./build.sh macos-arm --release

# 调试构建
./build.sh all
```

#### 手动交叉编译

```bash
# 添加目标平台
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# 编译特定平台
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

编译完成后，可执行文件将位于标准的 `target/[目标三元组]/[构建模式]/` 目录中，同时会在 `target/[目标三元组]/[构建模式]-release/` 目录中创建带平台名称的副本。

## 使用说明

1. **选择输入文件**：点击"选择 MKV 文件"按钮选择要转换的杜比视界 MKV 文件

2. **选择输出文件夹**：点击"选择输出文件夹"按钮选择转换后文件的保存位置

3. **配置选项**：
   - **包含字幕**：勾选此选项将同时处理字幕轨道
   - **帧率**：选择正确的帧率（如果不确定，使用 23.976 fps）

4. **开始处理**：点击"开始处理"按钮开始转换过程

5. **监控进度**：在处理日志区域查看详细的处理步骤和进度

## 输出文件

### 应用程序输出
- 基本输出：`[原文件名]_dvh1.mp4` - 包含杜比视界视频和音频
- 带字幕输出：`[原文件名]_dvh1_with_subs.mp4` - 包含杜比视界、音频和字幕

### 构建输出位置
- **标准位置**：`target/[目标三元组]/[debug|release]/dv2macdv[.exe]`
- **平台命名副本**：`target/[目标三元组]/[debug|release]-release/dv2macdv-[平台名][.exe]`

例如：
- Windows: `target/x86_64-pc-windows-msvc/release/dv2macdv.exe`
- Linux: `target/x86_64-unknown-linux-gnu/release/dv2macdv`
- macOS: `target/x86_64-apple-darwin/release/dv2macdv`

## 技术说明

### 处理流程

1. **视频提取**：使用 `mkvextract` 从 MKV 文件中提取杜比视界 HEVC 流
2. **音频提取**：使用 `ffmpeg` 提取音频轨道（通常是 E-AC-3 格式）
3. **字幕提取**：（可选）使用 `ffmpeg` 提取 SRT 字幕
4. **重新封装**：使用 `mp4muxer` 将视频和音频封装为支持 `dvh1` 的 MP4 文件
5. **字幕集成**：（可选）将字幕转换为 `mov_text` 格式并集成到最终文件中

### 支持的帧率

| 标准名称 | 分数表示 | 小数值 |
|---------|----------|--------|
| 胶片 (NTSC) | 24000/1001 | 23.976 |
| 胶片 (PAL) | 24 | 24.000 |
| 电视 (NTSC) | 30000/1001 | 29.970 |
| 电视 (PAL) | 25 | 25.000 |
| 高帧率 | 60 | 60.000 |
| NTSC 高帧率 | 60000/1001 | 59.940 |

## 故障排除

### 常见问题

1. **"无法执行 mkvextract"错误**
   - 确保 MKVToolNix 已正确安装
   - 检查 `mkvextract` 是否在 PATH 中

2. **"无法执行 ffmpeg"错误**
   - 确保 FFmpeg 已正确安装
   - 检查 `ffmpeg` 是否在 PATH 中

3. **"无法执行 mp4muxer"错误**
   - 确保 mp4muxer 工具可用
   - 检查工具是否在 PATH 中

4. **字幕处理失败**
   - 确保 GPAC (MP4Box) 已安装
   - 检查输入文件是否包含字幕轨道

### 日志信息

程序会在处理日志区域显示详细的错误信息，请根据具体错误消息进行排查。

## 许可证

此项目基于 MIT 许可证开源。

## 贡献

欢迎提交 Issue 和 Pull Request 来改进这个工具。

## 免责声明

- 此工具仅供学习和个人使用
- 请确保您有权处理相关的媒体文件
- 杜比视界是 Dolby Laboratories 的注册商标 
