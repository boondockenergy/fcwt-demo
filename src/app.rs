use fcwt::{scales::LinFreqs, wavelet::Wavelet, CwtResult, FastCwt, MorletWavelet};
use egui_plot::{Line, Plot, PlotPoints};
use egui::{epaint::Hsva, load::SizedTexture, CentralPanel, Color32, ColorImage, Context, Image, ImageSource, RichText, SidePanel, TextureHandle};

use crate::{audio, worklet};


pub struct WaveletDemo {
    texture: Option<TextureHandle>,
    image: Option<ColorImage>,

    fs: usize,
    sigma: f32,
    size: usize,
    scale: f32,

    f0: f32,
    f1: f32,

    signal: Vec<f32>,
    signal_size: usize,

    normalize: bool,

    fcwt: FastCwt<MorletWavelet, LinFreqs>,
    output: Option<CwtResult<f32>>,

    audio_worker: Option<super::worklet::AudioWorker>,
    audio_handle: Option<super::audio::Handle>,
}

impl Default for WaveletDemo {
    fn default() -> Self {
        let sigma = 2.0;
        let fs: usize = 2000;
        let size: usize = 300;
        let scale: f32 = 100.0;

        let f0 = 1f32;
        let f1 = 50.0f32;

        let wavelet = MorletWavelet::new(sigma);
        let scales = LinFreqs::new(&wavelet, fs, f0, f1*2.0, size);

        let signal_size = 8192u32.next_power_of_two() as usize;

        let signal = fcwt::util::chirp(fs as f32, signal_size, f0, f1);

        let normalize = true;

        let fcwt = FastCwt::new(wavelet, scales, normalize);

        Self {
            texture: None,
            image: None,
            fs,
            sigma,
            size,
            f0,
            f1,
            signal,
            fcwt,
            scale,
            signal_size,
            output: None,
            normalize,
            audio_worker: None,
            audio_handle: None,
        }
    }
}

impl WaveletDemo {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        /*
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        */


        Default::default()
    }
}

impl eframe::App for WaveletDemo {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if self.texture.is_none() & !self.image.is_none(){
            // Allocate a new texture
            if let Some(image) = &self.image {
                let texture = ctx.load_texture("cwt", image.clone(), Default::default());
                self.texture = Some(texture);
            }
        }

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        SidePanel::left("Left").show(ctx, |ui| {
            ui.label(RichText::new("Common").color(Color32::LIGHT_BLUE));
            ui.add(egui::Slider::new(&mut self.fs, 1000..=120000).text("Sample Rate"));
            ui.separator();

            ui.label(RichText::new("Wavelet").color(Color32::LIGHT_BLUE));
            ui.add(egui::Slider::new(&mut self.sigma, 1.0..=16.0).text("Wavelet Sigma"));
            ui.add(egui::Slider::new(&mut self.size, 1..=1000).text("Size"));
            let wave = self.fcwt.wavelet().generate(self.size, self.scale);
            let wave_points: PlotPoints = wave.iter().enumerate().map(|(x,&v)| {
                [x as f64, v.re as f64]
            }).collect();

            let wave_line = Line::new(wave_points);

            Plot::new("wavelet_plot")
                .view_aspect(3.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(wave_line);
                });
            ui.separator();

            ui.label(RichText::new("Chirp Signal").color(Color32::LIGHT_BLUE));
            ui.add(egui::Slider::new(&mut self.f0, 1.0..=100.0).text("Start Freq"));
            ui.add(egui::Slider::new(&mut self.f1, self.f0..=(self.fs as f32 / 4.0)).text("End Freq"));

            ui.separator();
            if ui.button("Update Transform").clicked() {
                //self.audio_handle = Some(crate::audio::beep());
                let output = self.fcwt.cwt(&self.signal);
                self.output = Some(output);
                self.update_image();

                // Save to transform.csv
                //save_csv("transform.csv".to_string(), &output);
                //save_signal_csv("signal.csv".to_string(), &self.signal);
            };

            if ui.button("Start Audio Worker").clicked() {
                worklet::audio_init();
            }

        });

        /*
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("fCWT Demo");

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/boondockenergy/fcwt-demo",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        */

        CentralPanel::default().show(ctx, |ui| {

            let signal_points: PlotPoints = self.signal.iter().enumerate().map(|(x,&v)| {
                [x as f64, v as f64]
            }).collect();
            let signal_line = Line::new(signal_points);


            Plot::new("signal_plot")
                .view_aspect(3.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(signal_line);
                });

            if let Some(handle) = &self.texture {
                let texture = SizedTexture::from_handle(handle);
                let image = Image::new(ImageSource::Texture(texture))
                    .shrink_to_fit();
                ui.add(image);
            }

        });

        if self.f1 <= self.fs as f32 / 2.0 {
            let wavelet = MorletWavelet::new(self.sigma);
            let scales = LinFreqs::new(&wavelet, self.fs, self.f0, self.f1*2.0, self.size);
            self.fcwt = FastCwt::new(wavelet, scales, self.normalize);

            self.signal = fcwt::util::chirp(self.fs as f32, self.signal_size, self.f0, self.f1);
        }

    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}


impl WaveletDemo {
    fn update_image(&mut self) {

        // Get pixel value from the fCWT result
        if let Some(output) = &self.output {

            if self.image.is_none() || self.image.as_ref().unwrap().height() != output.num_scales() {
                self.image = Some(egui::ColorImage::new([output[0].len(),output.num_scales()], Color32::LIGHT_YELLOW));
            }

            for y in 0..output.num_scales() {
                for x in 0..output[0].len() {
                    let val = output[y][x];
                    let c = Hsva::new(val.norm(), 1.0, 1.0, 1.0);
                    if let Some(image) = &mut self.image {
                        image.pixels[y * output[0].len() + x] = c.into();
                    }
                }
            }

            if let Some(handle) = &mut self.texture {
                if let Some(img) = &self.image {
                    handle.set(img.clone(), Default::default());
                }
            }
        }
    }
}