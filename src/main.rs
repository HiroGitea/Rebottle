use iced::widget::{
    Space, button, checkbox, column, container, pick_list, progress_bar, row, scrollable, text,
};
use iced::{Alignment, Element, Length, Task, Theme};
use resvg::usvg;
use rfd::FileDialog;
use std::path::PathBuf;
use std::process::Command;
use tiny_skia::Pixmap;

#[derive(Debug, Clone)]
pub struct App {
    // 文件路径
    input_file: Option<PathBuf>,
    output_folder: Option<PathBuf>,
    // 处理选项
    include_subtitles: bool,
    frame_rate: FrameRate,
    // 状态
    processing: bool,
    progress: f32,
    log_messages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrameRate {
    Film23976, // 24000/1001
    Film24,    // 24
    Tv29970,   // 30000/1001
    Tv25,      // 25
    Hfr60,     // 60
    Hfr59940,  // 60000/1001
}

impl FrameRate {
    fn to_string(&self) -> &'static str {
        match self {
            FrameRate::Film23976 => "23.976 (24000/1001)",
            FrameRate::Film24 => "24.000 (24)",
            FrameRate::Tv29970 => "29.970 (30000/1001)",
            FrameRate::Tv25 => "25.000 (25)",
            FrameRate::Hfr60 => "60.000 (60)",
            FrameRate::Hfr59940 => "59.940 (60000/1001)",
        }
    }

    fn to_value(&self) -> &'static str {
        match self {
            FrameRate::Film23976 => "24000/1001",
            FrameRate::Film24 => "24",
            FrameRate::Tv29970 => "30000/1001",
            FrameRate::Tv25 => "25",
            FrameRate::Hfr60 => "60",
            FrameRate::Hfr59940 => "60000/1001",
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            input_file: None,
            output_folder: None,
            include_subtitles: false,
            frame_rate: FrameRate::Film23976,
            processing: false,
            progress: 0.0,
            log_messages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectInputFile,
    InputFileSelected(Option<PathBuf>),
    SelectOutputFolder,
    OutputFolderSelected(Option<PathBuf>),
    ToggleSubtitles(bool),
    FrameRateSelected(FrameRate),
    StartProcessing,
    ProcessingStep(String),
    ProcessingProgress(f32),
    ProcessingComplete(Result<(), String>),
    ClearLog,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectInputFile => {
                Task::perform(select_input_file(), Message::InputFileSelected)
            }
            Message::InputFileSelected(path) => {
                self.input_file = path;
                Task::none()
            }
            Message::SelectOutputFolder => {
                Task::perform(select_output_folder(), Message::OutputFolderSelected)
            }
            Message::OutputFolderSelected(path) => {
                self.output_folder = path;
                Task::none()
            }
            Message::ToggleSubtitles(enabled) => {
                self.include_subtitles = enabled;
                Task::none()
            }
            Message::FrameRateSelected(frame_rate) => {
                self.frame_rate = frame_rate;
                Task::none()
            }
            Message::StartProcessing => {
                if let (Some(input), Some(output)) = (&self.input_file, &self.output_folder) {
                    self.processing = true;
                    self.progress = 0.0;
                    self.log_messages.clear();

                    let input = input.clone();
                    let output = output.clone();
                    let frame_rate = self.frame_rate.clone();
                    let include_subtitles = self.include_subtitles;

                    Task::perform(
                        process_video(input, output, frame_rate, include_subtitles),
                        Message::ProcessingComplete,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ProcessingStep(step) => {
                self.log_messages.push(step);
                Task::none()
            }
            Message::ProcessingProgress(progress) => {
                self.progress = progress;
                Task::none()
            }
            Message::ProcessingComplete(result) => {
                self.processing = false;
                match result {
                    Ok(_) => {
                        self.log_messages
                            .push("✅ Processing completed successfully!".to_string());
                        self.progress = 1.0;
                    }
                    Err(err) => {
                        self.log_messages
                            .push(format!("❌ Processing failed: {err}"));
                        self.progress = 0.0;
                    }
                }
                Task::none()
            }
            Message::ClearLog => {
                self.log_messages.clear();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("Dolby Vision MKV to MP4 Converter")
            .size(32)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.palette().primary),
            });

        let input_section = column![
            text("Input File:").size(16),
            row![
                text(
                    self.input_file
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "No file selected".to_string())
                )
                .width(Length::Fill),
                button("Select MKV File").on_press(Message::SelectInputFile)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5);

        let output_section = column![
            text("Output Folder:").size(16),
            row![
                text(
                    self.output_folder
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "No folder selected".to_string())
                )
                .width(Length::Fill),
                button("Select Output Folder").on_press(Message::SelectOutputFolder)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5);

        let options_section = column![
            text("Options:").size(16),
            checkbox("Include Subtitles", self.include_subtitles)
                .on_toggle(Message::ToggleSubtitles),
            row![
                text("Frame Rate:"),
                pick_list(
                    vec![
                        FrameRate::Film23976,
                        FrameRate::Film24,
                        FrameRate::Tv29970,
                        FrameRate::Tv25,
                        FrameRate::Hfr60,
                        FrameRate::Hfr59940,
                    ],
                    Some(self.frame_rate.clone()),
                    Message::FrameRateSelected
                )
                .text_size(14)
                .placeholder("Select Frame Rate")
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(10);

        let process_section = column![if self.processing {
            column![
                text("Processing...").size(16),
                progress_bar(0.0..=1.0, self.progress)
            ]
            .spacing(5)
        } else {
            column![
                button("Start Processing")
                    .on_press_maybe(
                        if self.input_file.is_some() && self.output_folder.is_some() {
                            Some(Message::StartProcessing)
                        } else {
                            None
                        }
                    )
                    .style(|theme: &Theme, status| {
                        button::Style {
                            background: Some(iced::Background::Color(theme.palette().primary)),
                            text_color: theme.palette().background,
                            ..button::primary(theme, status)
                        }
                    })
            ]
        }];

        let log_section = if !self.log_messages.is_empty() {
            column![
                row![
                    text("Processing Log:").size(16),
                    Space::with_width(Length::Fill),
                    button("Clear Log").on_press(Message::ClearLog)
                ]
                .align_y(Alignment::Center),
                container(
                    scrollable(
                        column(
                            self.log_messages
                                .iter()
                                .map(|msg| text(msg).size(12).into())
                                .collect::<Vec<_>>()
                        )
                        .spacing(2)
                    )
                    .height(Length::Fixed(150.0))
                )
                .style(|theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(theme.palette().background)),
                    border: iced::Border {
                        color: theme.palette().text,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .padding(10)
            ]
            .spacing(5)
        } else {
            column![]
        };

        container(
            column![
                title,
                input_section,
                output_section,
                options_section,
                process_section,
                log_section
            ]
            .spacing(20)
            .max_width(800),
        )
        .padding(20)
        .center_x(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

async fn select_input_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("MKV Video Files", &["mkv"])
        .set_title("Select Input MKV File")
        .pick_file()
}

async fn select_output_folder() -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select Output Folder")
        .pick_folder()
}

// 跨平台命令执行函数
fn execute_command(command: &str, args: &[&str]) -> Result<std::process::Output, String> {
    #[cfg(windows)]
    {
        let full_command = format!("{} {}", command, args.join(" "));
        Command::new("cmd")
            .args(["/C", &full_command])
            .output()
            .map_err(|e| format!("Failed to execute command: {e}"))
    }

    #[cfg(not(windows))]
    {
        Command::new(command)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute command {}: {}", command, e))
    }
}

async fn process_video(
    input_file: PathBuf,
    output_folder: PathBuf,
    frame_rate: FrameRate,
    include_subtitles: bool,
) -> Result<(), String> {
    let input_stem = input_file.file_stem().unwrap().to_string_lossy();
    let temp_dir = std::env::temp_dir();

    // 第1步: 提取视频流
    let video_file = temp_dir.join(format!("{input_stem}_DV.hevc"));

    let output = execute_command(
        "mkvextract",
        &[
            "tracks",
            &input_file.to_string_lossy(),
            &format!("0:{}", video_file.to_string_lossy()),
        ],
    )?;

    if !output.status.success() {
        return Err(format!(
            "Video extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Step 2: Extract audio
    let audio_file = temp_dir.join(format!("{input_stem}_audio.ec3"));

    let output = execute_command(
        "ffmpeg",
        &[
            "-i",
            &input_file.to_string_lossy(),
            "-map",
            "0:a:0",
            "-c",
            "copy",
            &audio_file.to_string_lossy(),
            "-y",
        ],
    )?;

    if !output.status.success() {
        return Err(format!(
            "Audio extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Step 3: Extract subtitles (if needed)
    let subtitle_file = if include_subtitles {
        let subs = temp_dir.join(format!("{input_stem}_subs.srt"));

        let output = execute_command(
            "ffmpeg",
            &[
                "-i",
                &input_file.to_string_lossy(),
                "-map",
                "0:s:0",
                "-c",
                "copy",
                &subs.to_string_lossy(),
                "-y",
            ],
        );

        match output {
            Ok(out) if out.status.success() => Some(subs),
            _ => None, // Subtitle extraction failed, continue without subtitles
        }
    } else {
        None
    };

    // Step 4: Remux using mp4muxer
    let output_file = output_folder.join(format!("{input_stem}_dvh1.mp4"));

    let output = execute_command(
        "mp4muxer",
        &[
            "-o",
            &output_file.to_string_lossy(),
            "-i",
            &video_file.to_string_lossy(),
            "--input-video-frame-rate",
            frame_rate.to_value(),
            "-i",
            &audio_file.to_string_lossy(),
            "--dv-profile",
            "5",
            "--dvh1flag",
            "0",
        ],
    )?;

    if !output.status.success() {
        return Err(format!(
            "MP4 muxing failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Step 5: Process subtitles (if available)
    if let Some(ref subtitle_file) = subtitle_file {
        let subs_mp4 = temp_dir.join(format!("{input_stem}_subs.mp4"));
        let final_output = output_folder.join(format!("{input_stem}_dvh1_with_subs.mp4"));

        // Convert subtitle format
        let output = execute_command(
            "ffmpeg",
            &[
                "-i",
                &subtitle_file.to_string_lossy(),
                "-c:s",
                "mov_text",
                &subs_mp4.to_string_lossy(),
                "-y",
            ],
        )?;

        if output.status.success() {
            // Merge subtitles
            let output = execute_command(
                "MP4Box",
                &[
                    "-add",
                    &output_file.to_string_lossy(),
                    "-add",
                    &subs_mp4.to_string_lossy(),
                    "-new",
                    &final_output.to_string_lossy(),
                ],
            )?;

            if !output.status.success() {
                return Err(format!(
                    "Subtitle merging failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }

    // Clean up temporary files
    let _ = std::fs::remove_file(video_file);
    let _ = std::fs::remove_file(audio_file);
    if let Some(subtitle_file) = subtitle_file {
        let _ = std::fs::remove_file(subtitle_file);
    }

    Ok(())
}

impl std::fmt::Display for FrameRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn load_svg_icon() -> Option<iced::window::Icon> {
    // 读取SVG文件
    let svg_data = std::fs::read_to_string("assets/icons/icon.svg").ok()?;

    // 解析SVG
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg_data, &options).ok()?;

    // 创建32x32的画布
    let size = 32;
    let mut pixmap = Pixmap::new(size, size)?;

    // 渲染SVG到画布
    let scale = size as f32 / tree.size().width().max(tree.size().height());
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // 转换为RGBA数据
    let rgba_data = pixmap.take();

    // 创建窗口图标
    iced::window::icon::from_rgba(rgba_data, size, size).ok()
}

fn main() -> iced::Result {
    iced::application("Dolby Vision Converter", App::update, App::view)
        .theme(|_| Theme::CatppuccinMocha)
        .window(iced::window::Settings {
            icon: load_svg_icon(),
            ..Default::default()
        })
        .run()
}
