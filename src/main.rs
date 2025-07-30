use iced::event::{self, Event};
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
    // 文件队列
    file_queue: Vec<PathBuf>,
    output_folder: Option<PathBuf>,
    // 处理选项
    include_subtitles: bool,
    frame_rate: FrameRate,
    // 状态
    processing: bool,
    current_file_index: usize,
    progress: f32,
    log_messages: Vec<String>,
    // 新增：终端日志
    terminal_logs: Vec<String>,
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
            file_queue: Vec::new(),
            output_folder: None,
            include_subtitles: false,
            frame_rate: FrameRate::Film23976,
            processing: false,
            current_file_index: 0,
            progress: 0.0,
            log_messages: Vec::new(),
            terminal_logs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectInputFiles,
    InputFilesSelected(Vec<PathBuf>),
    FilesDropped(Vec<PathBuf>),
    RemoveFileFromQueue(usize),
    ClearQueue,
    SelectOutputFolder,
    OutputFolderSelected(Option<PathBuf>),
    ToggleSubtitles(bool),
    FrameRateSelected(FrameRate),
    StartProcessing,
    ProcessingStep(String),
    ProcessingProgress(f32),
    ProcessingComplete(Result<(), String>),
    ClearLog,
    // 新增：终端日志消息
    TerminalOutput(String),
    ClearTerminal,
    ProcessingCompleteWithLogs((Result<(), String>, Vec<String>)),
}

impl App {
    fn subscription(&self) -> iced::Subscription<Message> {
        event::listen().map(|event| match event {
            Event::Window(iced::window::Event::FileDropped(path)) => {
                if let Some(extension) = path.extension() {
                    if extension.to_string_lossy().to_lowercase() == "mkv" {
                        return Message::FilesDropped(vec![path]);
                    }
                }
                Message::FilesDropped(vec![])
            }
            _ => Message::FilesDropped(vec![]),
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectInputFiles => {
                Task::perform(select_input_files(), Message::InputFilesSelected)
            }
            Message::InputFilesSelected(files) => {
                self.file_queue.extend(files);
                Task::none()
            }
            Message::FilesDropped(files) => {
                self.file_queue.extend(files);
                Task::none()
            }
            Message::RemoveFileFromQueue(index) => {
                if index < self.file_queue.len() {
                    self.file_queue.remove(index);
                }
                Task::none()
            }
            Message::ClearQueue => {
                self.file_queue.clear();
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
                if !self.file_queue.is_empty() && self.output_folder.is_some() {
                    self.processing = true;
                    self.current_file_index = 0;
                    self.progress = 0.0;
                    self.log_messages.clear();
                    self.terminal_logs.clear();

                    let files = self.file_queue.clone();
                    let output = self.output_folder.as_ref().unwrap().clone();
                    let frame_rate = self.frame_rate.clone();
                    let include_subtitles = self.include_subtitles;

                    Task::perform(
                        process_video_queue_with_logs(files, output, frame_rate, include_subtitles),
                        Message::ProcessingCompleteWithLogs,
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
            Message::TerminalOutput(output) => {
                self.terminal_logs.push(output);
                Task::none()
            }
            Message::ClearTerminal => {
                self.terminal_logs.clear();
                Task::none()
            }
            Message::ProcessingCompleteWithLogs((result, logs)) => {
                self.processing = false;
                // 将终端日志添加到terminal_logs
                self.terminal_logs.extend(logs);
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
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("Dolby Vision MKV to MP4 Converter")
            .size(32)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.palette().primary),
            });

        let queue_header = row![
            text("File Queue:").size(16),
            Space::with_width(Length::Fill),
            text(format!("{} files", self.file_queue.len())).size(14),
            button("Select Files").on_press(Message::SelectInputFiles),
            button("Clear Queue").on_press(Message::ClearQueue)
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let queue_list =
            if self.file_queue.is_empty() {
                container(
                text("No files. Drag and drop MKV files here or click the button above to select")
                    .size(14)
                    .style(|_theme: &Theme| text::Style {
                        color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                    })
            )
            .center_x(Length::Fill)
            .padding(20)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.05, 0.05, 0.05))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
            } else {
                container(
                    scrollable(
                        column(
                            self.file_queue
                                .iter()
                                .enumerate()
                                .map(|(index, file)| {
                                    row![
                                        text(format!(
                                            "{}. {}",
                                            index + 1,
                                            file.file_name().unwrap_or_default().to_string_lossy()
                                        ))
                                        .size(12)
                                        .width(Length::Fill),
                                        button("Remove")
                                            .on_press(Message::RemoveFileFromQueue(index))
                                            .style(|theme: &Theme, _status| {
                                                button::Style {
                                                    background: Some(iced::Background::Color(
                                                        iced::Color::from_rgb(0.8, 0.2, 0.2),
                                                    )),
                                                    text_color: iced::Color::WHITE,
                                                    ..button::primary(theme, _status)
                                                }
                                            })
                                    ]
                                    .spacing(10)
                                    .align_y(Alignment::Center)
                                    .into()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .spacing(5),
                    )
                    .height(Length::Fixed(150.0)),
                )
                .padding(10)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.05, 0.05, 0.05,
                    ))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
            };

        let input_section = column![queue_header, queue_list].spacing(10);

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
                button("Start Batch Processing")
                    .on_press_maybe(
                        if !self.file_queue.is_empty() && self.output_folder.is_some() {
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
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.1, 0.1, 0.1
                    ))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.3, 0.3, 0.3),
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

        // 新增：终端显示区域
        let terminal_section = column![
            row![
                text("Terminal:").size(16),
                Space::with_width(Length::Fill),
                button("Clear Terminal").on_press(Message::ClearTerminal)
            ]
            .align_y(Alignment::Center),
            container(
                scrollable(
                    column(
                        self.terminal_logs
                            .iter()
                            .map(|cmd| text(cmd).size(11).font(iced::Font::MONOSPACE).into())
                            .collect::<Vec<_>>()
                    )
                    .spacing(2)
                )
                .height(Length::Fixed(350.0))
                .width(Length::Fill)
            )
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.1, 0.1, 0.1
                ))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .padding(10)
            .width(Length::Fill)
        ]
        .spacing(5);

        container(
            column![
                title,
                input_section,
                output_section,
                options_section,
                process_section,
                log_section,
                terminal_section
            ]
            .spacing(20)
            .max_width(1200),
        )
        .padding(20)
        .center_x(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

async fn select_input_files() -> Vec<PathBuf> {
    FileDialog::new()
        .add_filter("MKV Video Files", &["mkv"])
        .set_title("Select Input MKV Files")
        .pick_files()
        .unwrap_or_default()
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

// 新增：带有终端日志记录的命令执行函数
async fn execute_command_with_logging(
    command: &str,
    args: &[&str],
) -> (Result<std::process::Output, String>, Vec<String>) {
    let mut logs = Vec::new();

    // 记录要执行的命令
    let full_command = if args.is_empty() {
        format!("$ {command}")
    } else {
        format!("$ {command} {}", args.join(" "))
    };

    logs.push(full_command);

    // 执行命令
    let result = execute_command(command, args);

    // 记录执行结果
    match &result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.trim().is_empty() {
                    logs.push(format!("Error: {}", stderr.trim()));
                }
            } else {
                logs.push("✓ Command completed successfully".to_string());
            }
        }
        Err(e) => {
            logs.push(format!("Error: {e}"));
        }
    }

    (result, logs)
}

// 新增：带有日志收集的视频处理函数
async fn process_video_with_logs(
    input_file: PathBuf,
    output_folder: PathBuf,
    frame_rate: FrameRate,
    include_subtitles: bool,
) -> (Result<(), String>, Vec<String>) {
    let input_stem = input_file.file_stem().unwrap().to_string_lossy();
    let temp_dir = std::env::temp_dir();
    let mut all_logs = Vec::new();

    // Step 1: Extract video stream
    all_logs.push("Extracting video stream...".to_string());
    let video_file = temp_dir.join(format!("{input_stem}_DV.hevc"));

    let (output, mut logs) = execute_command_with_logging(
        "mkvextract",
        &[
            "tracks",
            &input_file.to_string_lossy(),
            &format!("0:{}", video_file.to_string_lossy()),
        ],
    )
    .await;
    all_logs.append(&mut logs);

    match output {
        Ok(out) if !out.status.success() => {
            return (
                Err(format!(
                    "Video extraction failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                )),
                all_logs,
            );
        }
        Err(e) => return (Err(e), all_logs),
        _ => {}
    }

    // Step 2: Extract audio
    all_logs.push("Extracting audio stream...".to_string());
    let audio_file = temp_dir.join(format!("{input_stem}_audio.ec3"));

    let (output, mut logs) = execute_command_with_logging(
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
    )
    .await;
    all_logs.append(&mut logs);

    match output {
        Ok(out) if !out.status.success() => {
            return (
                Err(format!(
                    "Audio extraction failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                )),
                all_logs,
            );
        }
        Err(e) => return (Err(e), all_logs),
        _ => {}
    }

    // Step 3: Extract subtitles (if needed)
    let subtitle_file = if include_subtitles {
        all_logs.push("Extracting subtitles...".to_string());
        let subs = temp_dir.join(format!("{input_stem}_subs.srt"));

        let (output, mut logs) = execute_command_with_logging(
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
        )
        .await;
        all_logs.append(&mut logs);

        match output {
            Ok(out) if out.status.success() => Some(subs),
            _ => {
                all_logs.push("Subtitle extraction failed, continuing...".to_string());
                None
            }
        }
    } else {
        None
    };

    // Step 4: Remux using mp4muxer
    all_logs.push("Remuxing to MP4...".to_string());
    let output_file = output_folder.join(format!("{input_stem}_dvh1.mp4"));

    let (output, mut logs) = execute_command_with_logging(
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
    )
    .await;
    all_logs.append(&mut logs);

    match output {
        Ok(out) if !out.status.success() => {
            return (
                Err(format!(
                    "MP4 muxing failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                )),
                all_logs,
            );
        }
        Err(e) => return (Err(e), all_logs),
        _ => {}
    }

    // Step 5: Process subtitles (if available)
    if let Some(ref subtitle_file) = subtitle_file {
        all_logs.push("Processing subtitles...".to_string());
        let subs_mp4 = temp_dir.join(format!("{input_stem}_subs.mp4"));
        let final_output = output_folder.join(format!("{input_stem}_dvh1_with_subs.mp4"));

        // Convert subtitle format
        let (output, mut logs) = execute_command_with_logging(
            "ffmpeg",
            &[
                "-i",
                &subtitle_file.to_string_lossy(),
                "-c:s",
                "mov_text",
                &subs_mp4.to_string_lossy(),
                "-y",
            ],
        )
        .await;
        all_logs.append(&mut logs);

        if let Ok(out) = output {
            if out.status.success() {
                // Merge subtitles
                let (output, mut logs) = execute_command_with_logging(
                    "MP4Box",
                    &[
                        "-add",
                        &output_file.to_string_lossy(),
                        "-add",
                        &subs_mp4.to_string_lossy(),
                        "-new",
                        &final_output.to_string_lossy(),
                    ],
                )
                .await;
                all_logs.append(&mut logs);

                if let Ok(out) = output {
                    if !out.status.success() {
                        return (
                            Err(format!(
                                "Subtitle merging failed: {}",
                                String::from_utf8_lossy(&out.stderr)
                            )),
                            all_logs,
                        );
                    }
                }
            }
        }
    }

    // Clean up temporary files
    all_logs.push("Cleaning up temporary files...".to_string());
    let _ = std::fs::remove_file(video_file);
    let _ = std::fs::remove_file(audio_file);
    if let Some(subtitle_file) = subtitle_file {
        let _ = std::fs::remove_file(subtitle_file);
    }

    all_logs.push("Processing completed!".to_string());
    (Ok(()), all_logs)
}

// 新增：批量处理视频队列的函数
async fn process_video_queue_with_logs(
    files: Vec<PathBuf>,
    output_folder: PathBuf,
    frame_rate: FrameRate,
    include_subtitles: bool,
) -> (Result<(), String>, Vec<String>) {
    let mut all_logs = Vec::new();
    let total_files = files.len();

    all_logs.push(format!(
        "Starting batch processing of {total_files} files..."
    ));

    for (index, file) in files.iter().enumerate() {
        all_logs.push(format!(
            "Processing file {}/{}: {}",
            index + 1,
            total_files,
            file.file_name().unwrap_or_default().to_string_lossy()
        ));

        let (result, mut logs) = process_video_with_logs(
            file.clone(),
            output_folder.clone(),
            frame_rate.clone(),
            include_subtitles,
        )
        .await;

        all_logs.append(&mut logs);

        if let Err(e) = result {
            all_logs.push(format!("File processing failed: {e}"));
            return (
                Err(format!(
                    "Batch processing failed at file {}: {}",
                    index + 1,
                    e
                )),
                all_logs,
            );
        }

        all_logs.push(format!("✅ File {}/{} completed", index + 1, total_files));
    }

    all_logs.push(format!(
        "🎉 All {total_files} files processed successfully!"
    ));
    (Ok(()), all_logs)
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
        .subscription(App::subscription)
        .theme(|_| Theme::CatppuccinMocha)
        .window(iced::window::Settings {
            icon: load_svg_icon(),
            ..Default::default()
        })
        .run()
}
