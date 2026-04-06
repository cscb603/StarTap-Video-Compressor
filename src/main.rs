#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use eframe::egui;
use eframe::egui::NumExt;
use fast_video_compress_rs_v2::*;
use futures::StreamExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use walkdir::WalkDir;

#[derive(PartialEq, Clone)]
enum JobStatus {
    Pending,
    Processing(f32),
    Finished,
    Failed(String),
}

struct VideoJob {
    path: PathBuf,
    status: JobStatus,
    original_size: u64,
    compressed_size: u64,
}

struct AppStats {
    total: usize,
    success: usize,
    failed: usize,
    original_total_size: u64,
    compressed_total_size: u64,
}

struct HardwareInfo {
    has_nvidia_gpu: bool,
    has_apple_gpu: bool,
    cpu_cores: usize,
}

impl HardwareInfo {
    fn detect() -> Self {
        let mut has_nvidia_gpu = false;
        let has_apple_gpu = false;
        
        #[cfg(windows)]
        {
            if let Ok(output) = std::process::Command::new("nvidia-smi")
                .arg("--query-gpu=name")
                .arg("--format=csv,noheader")
                .output()
            {
                if output.status.success() {
                    has_nvidia_gpu = true;
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            has_apple_gpu = true;
        }
        
        let cpu_cores = num_cpus::get();
        
        Self {
            has_nvidia_gpu,
            has_apple_gpu,
            cpu_cores,
        }
    }
    
    fn recommended_encoder(&self) -> Encoder {
        if self.has_nvidia_gpu {
            Encoder::HevcNvenc
        } else if self.has_apple_gpu {
            Encoder::HevcVideotoolbox
        } else {
            Encoder::LibsvtAv1
        }
    }
    
    fn recommended_concurrency(&self) -> usize {
        (self.cpu_cores / 2).clamp(1, 4)
    }
}

struct VideoCompressApp {
    config: AppConfig,
    jobs: Arc<Mutex<Vec<VideoJob>>>,
    stats: Arc<Mutex<AppStats>>,
    is_running: Arc<Mutex<bool>>,
    runtime: tokio::runtime::Runtime,
    hardware: HardwareInfo,
}

impl VideoCompressApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::light();
        let morandi_blue = egui::Color32::from_rgb(180, 195, 205);
        let morandi_bg = egui::Color32::from_rgb(245, 247, 248);
        
        visuals.panel_fill = morandi_bg;
        visuals.window_rounding = 16.0.into();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::WHITE;
        visuals.widgets.noninteractive.rounding = 12.0.into();
        visuals.widgets.inactive.bg_fill = morandi_blue.linear_multiply(0.3);
        visuals.widgets.inactive.rounding = 12.0.into();
        visuals.widgets.hovered.bg_fill = morandi_blue.linear_multiply(0.5);
        visuals.widgets.active.bg_fill = morandi_blue;
        
        cc.egui_ctx.set_visuals(visuals);

        let mut fonts = egui::FontDefinitions::default();
        let font_paths = if cfg!(target_os = "windows") {
            vec![
                "C:\\Windows\\Fonts\\msyh.ttc",
                "C:\\Windows\\Fonts\\simhei.ttf",
            ]
        } else {
            vec![
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Light.ttc",
            ]
        };

        for path in font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "chinese_font".to_owned());
                fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("chinese_font".to_owned());
                cc.egui_ctx.set_fonts(fonts);
                break;
            }
        }

        let hardware = HardwareInfo::detect();
        let mut config: AppConfig = confy::load("fast-video-compress-rs-v3", None).unwrap_or_default();
        
        if config.encoder == Encoder::default() {
            config.encoder = hardware.recommended_encoder();
        }
        if config.concurrency == 2 {
            config.concurrency = hardware.recommended_concurrency();
        }

        Self {
            config,
            jobs: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(AppStats {
                total: 0,
                success: 0,
                failed: 0,
                original_total_size: 0,
                compressed_total_size: 0,
            })),
            is_running: Arc::new(Mutex::new(false)),
            runtime: tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap(),
            hardware,
        }
    }

    fn add_paths(&mut self, paths: Vec<PathBuf>) {
        let mut jobs = self.jobs.lock().unwrap();
        for path in paths {
            if path.is_dir() {
                for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if let Some(ext) = p.extension() {
                        if ["mp4", "mkv", "mov", "avi", "wmv", "flv", "webm", "ts", "m2ts"].contains(&ext.to_string_lossy().to_lowercase().as_str()) {
                            let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
                            jobs.push(VideoJob { path: p.to_path_buf(), status: JobStatus::Pending, original_size: size, compressed_size: 0 });
                        }
                    }
                }
            } else {
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                jobs.push(VideoJob { path, status: JobStatus::Pending, original_size: size, compressed_size: 0 });
            }
        }
    }

    fn run_compression(&mut self) {
        {
            let mut is_running = self.is_running.lock().unwrap();
            if *is_running { return; }
            *is_running = true;
        }
        
        let jobs_arc = self.jobs.clone();
        let stats_arc = self.stats.clone();
        let config = self.config.clone();
        let is_running_arc = self.is_running.clone();
        let _ = confy::store("fast-video-compress-rs-v3", None, &config);

        {
            let mut stats = stats_arc.lock().unwrap();
            stats.total = 0;
            stats.success = 0;
            stats.failed = 0;
            stats.original_total_size = 0;
            stats.compressed_total_size = 0;
        }

        self.runtime.spawn(async move {
            let pending_indices: Vec<usize> = {
                let jobs = jobs_arc.lock().unwrap();
                jobs.iter().enumerate()
                    .filter(|(_, j)| j.status == JobStatus::Pending)
                    .map(|(i, _)| i).collect()
            };

            {
                let mut stats = stats_arc.lock().unwrap();
                stats.total = pending_indices.len();
            }

            let process_config = app_config_to_process_config(&config, None);
            let processor = VideoProcessor::new(process_config);

            let semaphore = Arc::new(tokio::sync::Semaphore::new(config.concurrency));
            let mut stream = futures::stream::iter(pending_indices).map(|idx| {
                let jobs = jobs_arc.clone();
                let stats = stats_arc.clone();
                let sem = semaphore.clone();
                let processor = processor.clone();
                async move {
                    let _permit = sem.acquire().await.unwrap();
                    let (path, orig_size) = {
                        let mut jobs_lock = jobs.lock().unwrap();
                        let job = &mut jobs_lock[idx];
                        job.status = JobStatus::Processing(0.0);
                        (job.path.clone(), job.original_size)
                    };

                    let result = processor.compress_video(&path).await;

                    let mut jobs_lock = jobs.lock().unwrap();
                    let mut stats_lock = stats.lock().unwrap();
                    match result {
                        Ok(r) if r.success => {
                            jobs_lock[idx].status = JobStatus::Finished;
                            jobs_lock[idx].compressed_size = r.compressed_size;
                            stats_lock.success += 1;
                            stats_lock.original_total_size += orig_size;
                            stats_lock.compressed_total_size += r.compressed_size;
                        }
                        Ok(r) => {
                            jobs_lock[idx].status = JobStatus::Failed(r.error.unwrap_or_else(|| "Unknown error".to_string()));
                            stats_lock.failed += 1;
                        }
                        Err(e) => {
                            jobs_lock[idx].status = JobStatus::Failed(e.to_string());
                            stats_lock.failed += 1;
                        }
                    };
                }
            }).buffer_unordered(config.concurrency);

            while (stream.next().await).is_some() {}
            
            if cfg!(target_os = "windows") {
                let mut cmd = Command::new("explorer");
                cmd.arg(&config.output_dir);
                #[cfg(windows)]
                cmd.creation_flags(0x08000000);
                let _ = cmd.status().await;
            } else if cfg!(target_os = "macos") {
                let _ = Command::new("open").arg(&config.output_dir).status().await;
            }

            let mut is_running = is_running_arc.lock().unwrap();
            *is_running = false;
        });
    }
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 { return "0 B".to_string(); }
    let units = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log(1024.0).floor() as usize;
    format!("{:.2} {}", bytes as f64 / 1024.0f64.powi(i as i32), units[i])
}

impl eframe::App for VideoCompressApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(ctx.style().visuals.panel_fill).inner_margin(egui::Margin {
                left: 20.0,
                right: 60.0,
                top: 25.0,
                bottom: 25.0,
            }))
            .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("🚀 星TAP 极简视频压缩 V3").size(24.0).strong().color(egui::Color32::from_rgb(70, 85, 95)));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("标准化架构 · GUI+CLI 双接口 · 智能硬件检测").weak().size(13.0));
            });
            
            ui.add_space(20.0);

            let max_content_width = 600.0;
            let available_width = ui.available_width();
            
            ui.vertical_centered(|ui| {
                let content_width = available_width.at_most(max_content_width);
                ui.set_max_width(content_width);

                // 硬件检测信息（可选折叠）
                egui::CollapsingHeader::new("🖥️ 硬件检测信息")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| { ui.label("CPU:"); ui.label(self.hardware.cpu_cores.to_string()); });
                        ui.horizontal(|ui| { 
                            ui.label("NVIDIA:"); 
                            ui.label(if self.hardware.has_nvidia_gpu { "✅" } else { "❌" }); 
                        });
                        ui.horizontal(|ui| { 
                            ui.label("Apple GPU:"); 
                            ui.label(if self.hardware.has_apple_gpu { "✅" } else { "❌" }); 
                        });
                        if ui.button("重置推荐").clicked() {
                            self.config.encoder = self.hardware.recommended_encoder();
                            self.config.concurrency = self.hardware.recommended_concurrency();
                        }
                    });

                ui.add_space(10.0);

                // 设置区域 - 完全按照原版 Grid 布局
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::WHITE)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 225, 230)))
                    .rounding(16.0)
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width()); 
                        egui::Grid::new("config_grid")
                            .num_columns(2)
                            .spacing([15.0, 18.0])
                            .show(ui, |ui| {
                                let is_running_now = *self.is_running.lock().unwrap();
                                
                                // 导出目录
                                ui.add_sized([80.0, 20.0], egui::Label::new(egui::RichText::new("导出目录:").strong()));
                                ui.horizontal(|ui| {
                                    let btn_width = 85.0;
                                    let edit_width = (ui.available_width() - btn_width - 10.0).at_least(100.0);
                                    ui.add_enabled(!is_running_now, egui::TextEdit::singleline(&mut self.config.output_dir).desired_width(edit_width).margin(egui::vec2(8.0, 4.0)));
                                    if ui.add_enabled(!is_running_now, egui::Button::new(" 选择... ")).clicked() {
                                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                            self.config.output_dir = path.display().to_string();
                                        }
                                    }
                                });
                                ui.end_row();

                                // 编码器
                                ui.add_sized([80.0, 20.0], egui::Label::new(egui::RichText::new("编码器:").strong()));
                                ui.horizontal(|ui| {
                                    let combo_box = egui::ComboBox::from_id_source("encoder_cb")
                                        .selected_text(self.config.encoder.to_string())
                                        .width(250.0);
                                    let _ = combo_box.show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.config.encoder, Encoder::LibsvtAv1, "SVT-AV1 (推荐)");
                                        ui.selectable_value(&mut self.config.encoder, Encoder::Rav1e, "rav1e (极致)");
                                        ui.selectable_value(&mut self.config.encoder, Encoder::Libx265, "x265 (兼容)");
                                        if cfg!(target_os = "windows") {
                                            ui.selectable_value(&mut self.config.encoder, Encoder::HevcNvenc, "NVENC (N卡)");
                                        }
                                        if cfg!(target_os = "macos") {
                                            ui.selectable_value(&mut self.config.encoder, Encoder::HevcVideotoolbox, "VT (苹果)");
                                        }
                                    });
                                });
                                ui.end_row();

                                // 画质/并发
                                ui.add_sized([80.0, 20.0], egui::Label::new(egui::RichText::new("画质/速度:").strong()));
                                ui.horizontal(|ui| {
                                    let quality_label = match self.config.encoder {
                                        Encoder::LibsvtAv1 | Encoder::Rav1e | Encoder::Libx265 => "CRF (越小越好)",
                                        _ => "质量 (越高越好)",
                                    };
                                    let quality_range = match self.config.encoder {
                                        Encoder::LibsvtAv1 | Encoder::Rav1e | Encoder::Libx265 => 15..=40,
                                        _ => 1..=100,
                                    };
                                    let quality_slider = egui::Slider::new(&mut self.config.quality, quality_range).show_value(true).text(quality_label);
                                    if is_running_now { ui.add_enabled(false, quality_slider); } else { ui.add(quality_slider); }

                                    let quality_text = match self.config.encoder {
                                        Encoder::LibsvtAv1 | Encoder::Rav1e | Encoder::Libx265 => {
                                            if self.config.quality <= 20 { "🎥高质" } 
                                            else if self.config.quality <= 28 { "⚖️平衡" } 
                                            else { "📦压缩" }
                                        },
                                        _ => { if self.config.quality >= 80 { "🎥高质" } else if self.config.quality >= 50 { "⚖️平衡" } else { "📦压缩" } },
                                    };
                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new(quality_text).size(12.0).weak());

                                    ui.add_space(20.0);
                                    ui.label(egui::RichText::new("并发:").strong());
                                    let concurrency_slider = egui::Slider::new(&mut self.config.concurrency, 1..=8);
                                    if is_running_now { ui.add_enabled(false, concurrency_slider); } else { ui.add(concurrency_slider); }
                                });
                                ui.end_row();

                                // 速度档位
                                ui.add_sized([80.0, 20.0], egui::Label::new(egui::RichText::new("速度档位:").strong()));
                                ui.horizontal(|ui| {
                                    let speed_slider = egui::Slider::new(&mut self.config.speed_preset, 0..=13).show_value(true);
                                    if is_running_now { ui.add_enabled(false, speed_slider); } else { ui.add(speed_slider); }
                                    let speed_text = if self.config.speed_preset <= 4 { "🐢极致" } else if self.config.speed_preset <= 8 { "⚡推荐" } else { "🚀极速" };
                                    ui.label(egui::RichText::new(speed_text).size(12.0).weak());
                                });
                                ui.end_row();

                                // 最大尺寸
                                ui.add_sized([80.0, 20.0], egui::Label::new(egui::RichText::new("最大尺寸:").strong()));
                                ui.horizontal(|ui| {
                                    let display_text = if self.config.max_height == 0 {
                                        "原始大小 - 不缩放".to_string()
                                    } else if self.config.max_height == 1080 && self.config.is_quick_share {
                                        "1080p (临时分享 · 极致压缩)".to_string()
                                    } else if self.config.max_height == 1080 {
                                        "1080p (推荐 · 平衡)".to_string()
                                    } else {
                                        format!("{}p", self.config.max_height)
                                    };
                                    let combo_box = egui::ComboBox::from_id_source("height_cb")
                                        .selected_text(display_text)
                                        .width(230.0);
                                    let _ = combo_box.show_ui(ui, |ui| {
                                        if ui.selectable_value(&mut self.config.max_height, 0, "原始大小 - 不缩放").clicked() {
                                            self.config.is_quick_share = false;
                                        }
                                        if ui.selectable_value(&mut self.config.max_height, 1080, "1080p (临时分享 · 极致压缩)").clicked() {
                                            self.config.is_quick_share = true;
                                            self.config.quality = 32;
                                            self.config.speed_preset = 10;
                                        }
                                        if ui.selectable_value(&mut self.config.max_height, 1080, "1080p (推荐 · 平衡)").clicked() {
                                            self.config.is_quick_share = false;
                                            self.config.quality = 24;
                                            self.config.speed_preset = 8;
                                        }
                                        if ui.selectable_value(&mut self.config.max_height, 1440, "1440p (2K) - 高清").clicked() {
                                            self.config.is_quick_share = false;
                                            self.config.quality = 22;
                                        }
                                        if ui.selectable_value(&mut self.config.max_height, 2160, "2160p (4K) - 原画").clicked() {
                                            self.config.is_quick_share = false;
                                            self.config.quality = 20;
                                        }
                                    });
                                    ui.add_space(10.0);
                                    let height_text = match (self.config.max_height, self.config.is_quick_share) {
                                        (0, _) => "🎬不缩放",
                                        (1080, true) => "📦临时分享",
                                        (1080, false) => "⚖️推荐",
                                        (1440, _) => "📺2K",
                                        (2160, _) => "🎬4K",
                                        _ => "",
                                    };
                                    ui.label(egui::RichText::new(height_text).size(12.0).weak());
                                });
                                ui.end_row();
                            });
                    });

                ui.add_space(15.0);

                // 拖拽区域
                let hovering = ctx.input(|i| !i.raw.dropped_files.is_empty());
                let border_color = if hovering { egui::Color32::from_rgb(150, 180, 200) } else { egui::Color32::from_rgb(210, 215, 220) };
                
                egui::Frame::canvas(ui.style())
                    .fill(if hovering { egui::Color32::from_rgb(235, 242, 248) } else { egui::Color32::WHITE })
                    .stroke(egui::Stroke::new(2.0, border_color))
                    .rounding(16.0)
                    .inner_margin(30.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("📥").size(40.0));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("将视频或文件夹拖到此处").size(14.0).strong());
                            ui.add_space(15.0);
                            
                            ui.horizontal(|ui| {
                                let total_btn_w = 200.0;
                                let space = (ui.available_width() - total_btn_w) / 2.0;
                                ui.add_space(space.at_least(0.0)); 
                                if ui.button("➕ 添加文件").clicked() {
                                    if let Some(files) = rfd::FileDialog::new()
                                        .add_filter("视频", &["mp4", "mkv", "mov", "avi", "wmv", "flv", "webm", "ts", "m2ts"])
                                        .pick_files() {
                                        self.add_paths(files);
                                    }
                                }
                                ui.add_space(10.0);
                                if ui.button("📂 添加文件夹").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.add_paths(vec![path]);
                                    }
                                }
                            });
                        });
                    });
            });

            // 处理拖放文件
            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                let paths = ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
                self.add_paths(paths);
            }

            ui.add_space(20.0);

            // 任务列表区域
            ui.vertical_centered(|ui| {
                let cw = ui.available_width().at_most(max_content_width);
                ui.set_max_width(cw);
                
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("待处理任务 ({})", self.jobs.lock().unwrap().len())).strong().size(15.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(!*self.is_running.lock().unwrap(), egui::Button::new("🗑 清空列表")).clicked() {
                            self.jobs.lock().unwrap().clear();
                            let mut ir = self.is_running.lock().unwrap();
                            *ir = false;
                        }
                    });
                });
                ui.add_space(8.0);

                let stats = self.stats.lock().unwrap();
                if stats.total > 0 {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("✅ 成功: {}", stats.success)).size(12.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(format!("❌ 失败: {}", stats.failed)).size(12.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(format!("📊 压缩率: {:.1}%", 
                            if stats.original_total_size > 0 {
                                (1.0 - (stats.compressed_total_size as f64 / stats.original_total_size as f64)) * 100.0
                            } else { 0.0 }
                        )).size(12.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(format!("💾 节省: {}", format_size(stats.original_total_size.saturating_sub(stats.compressed_total_size)))).size(12.0).weak());
                    });
                    ui.add_space(8.0);
                }
                drop(stats);
                
                let sh = ui.available_height() - 80.0; 
                egui::ScrollArea::vertical()
                    .max_height(sh.at_least(100.0))
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                    let jobs = self.jobs.lock().unwrap();
                    for job in jobs.iter() {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(250, 251, 252))
                            .rounding(8.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    let name = job.path.file_name().unwrap().to_string_lossy();
                                    ui.label(egui::RichText::new(name).size(13.0));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        match &job.status {
                                            JobStatus::Pending => { ui.label(egui::RichText::new("⏳ 等待").color(egui::Color32::GRAY)); }
                                            JobStatus::Processing(_) => { ui.spinner(); ui.label(egui::RichText::new("⚙️ 压缩中").color(egui::Color32::from_rgb(100, 150, 200))); }
                                            JobStatus::Finished => { 
                                                ui.label(egui::RichText::new("✅ 完成").color(egui::Color32::from_rgb(80, 180, 100))); 
                                                ui.label(egui::RichText::new(format!("{} → {}", format_size(job.original_size), format_size(job.compressed_size))).size(11.0).weak());
                                            }
                                            JobStatus::Failed(e) => { ui.label(egui::RichText::new(format!("❌ 失败: {}", e)).color(egui::Color32::from_rgb(220, 100, 100))); }
                                        }
                                    });
                                });
                            });
                        ui.add_space(4.0);
                    }
                });

                ui.add_space(ui.available_height() - 60.0); 
                let running_now = *self.is_running.lock().unwrap();
                let btn_text = if running_now {
                    let s = self.stats.lock().unwrap();
                    if s.success + s.failed == s.total && s.total > 0 { "🎉 完成" } else { "🚀 处理中..." }
                } else { "开始批量压缩" };
                
                let btn = egui::Button::new(egui::RichText::new(btn_text).size(18.0).strong().color(egui::Color32::WHITE))
                    .min_size(egui::vec2(280.0, 50.0))
                    .fill(egui::Color32::from_rgb(130, 155, 175))
                    .rounding(25.0);
                
                if ui.add_enabled(!running_now && !self.jobs.lock().unwrap().is_empty(), btn).clicked() {
                    self.run_compression();
                }
            });
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn main() -> Result<(), eframe::Error> {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Warn,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    ).ok();
    
    let icon_bytes = include_bytes!("../视频压缩图标.ico");
    
    let icon_data = image::load_from_memory(icon_bytes).ok().map(|img| {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([680.0, 800.0])
        .with_min_inner_size([500.0, 650.0]);
    
    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "星TAP 极简视频压缩 V3",
        options,
        Box::new(|cc| Ok(Box::new(VideoCompressApp::new(cc)))),
    )
}
