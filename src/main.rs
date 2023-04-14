// Arknights Recruit Helper
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod vision;
mod custom_widgets;

use eframe::egui::*;
use eframe::egui::RichText;
use serde::{Serialize, Deserialize};
use std::io::Write;
use std::io::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use crate::Mode::{DisplayOptions, DisplayResults};
use crate::vision::Vision;

const PADDING: f32 = 10.0;
const LABELBGCOLOR: Color32 = Color32::from_rgb(25, 25, 25);
const NAPTIME: std::time::Duration = std::time::Duration::from_millis(20);

#[derive(PartialEq)]
enum Mode {
    DisplayResults,
    DisplayOptions,
}

#[derive(Serialize, Deserialize)]
pub struct Options{
    window_name: String,
    title_color: Color32,
    initial_window_size: (f32, f32),
    initial_window_pos: (f32, f32),
    tag_pos: (f32, f32, f32, f32),
    adjuster_1: f32,
    adjuster_2: f32,
    adjuster_3: f32,
    adjuster_4: f32,
    adjuster_5: f32,
    adjuster_6: f32,
}

impl Options {
    fn new() -> Self{
        let mut file = std::fs::File::open("./res/opt.txt").expect("/res/opt.txt not found");
        let mut data = String::new();
        file.read_to_string(&mut data).expect("Error while reading /res/opt.txt");
        serde_json::from_str(&data).expect("Error while interpreting /res/opt.txt")
    }

    // fn reset_tag_pos(&mut self) {
    //     self.tag_pos = (0.2875, 0.5163, 0.5780, 0.3980);
    //     self.adjuster_1 = 1.4449;
    //     self.adjuster_2 = 1.3206;
    //     self.adjuster_3 = 1.8890;
    //     self.adjuster_4 = 1.6416;
    //     self.adjuster_5 = 1.1865;
    //     self.adjuster_6 = 1.1660;
    // }

    fn get_pos(&self, pos: usize) -> (f32, f32, f32, f32) {
        match pos {
            1 => self.tag_pos,
            2 => {
                let withe = self.tag_pos.0 * self.adjuster_1;
                let height = self.tag_pos.1;
                let top = self.tag_pos.2;
                let left = self.tag_pos.3 * self.adjuster_2;
                (withe, height, top, left)
            },
            3 => {
                let withe = self.tag_pos.0 * self.adjuster_3;
                let height = self.tag_pos.1;
                let top = self.tag_pos.2;
                let left = self.tag_pos.3 * self.adjuster_4;
                (withe, height, top, left)
            },
            4 => {
                let withe = self.tag_pos.0;
                let height = self.tag_pos.1 * self.adjuster_5;
                let top = self.tag_pos.2 * self.adjuster_6;
                let left = self.tag_pos.3;
                (withe, height, top, left)
            },
            5 => {
                let withe = self.tag_pos.0 * self.adjuster_1;
                let height = self.tag_pos.1 * self.adjuster_5;
                let top = self.tag_pos.2 * self.adjuster_6;
                let left = self.tag_pos.3 * self.adjuster_2;
                (withe, height, top, left)
            },
            _ => (0.0, 0.0, 1.0, 1.0),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct Operator {
    index: usize,
    name: String,
    tags: Vec<String>,
    rar: i32,
}

impl Operator {
    fn contains(&self, other_tag: &String) -> bool {
        for tag in &self.tags {
            if tag == other_tag {
                return true
            }
        }
        false
    }
}

#[derive(Debug)]
struct OpResult {
    tag_combs: Vec<Vec<String>>,
    operators:  Vec<Operator>,
    min_rar: i32,
    // max_rar: i32,
}

#[derive(Serialize, Deserialize)]
struct Data {
    tags: Vec<String>,
    operators: Vec<Operator>,
    operators_without_top_operators: Vec<Operator>,
    tag_combs: Vec<Vec<usize>>,
    version: String,
}

impl Data {
    fn new() -> Self {
        let mut file = std::fs::File::open("./res/data.txt").expect("/res/data.txt not found");
        let mut data = String::new();
        file.read_to_string(&mut data).expect("Error while reading /res/data.txt");
        serde_json::from_str(&data).expect("Error while interpreting /res/data.txt")
    }

    fn get_raw_possible_combinations(&self, tags: &[String]) -> Vec<OpResult> {
        let mut results: Vec<OpResult> = Vec::new();

        for tag_comb in &self.tag_combs {
            let mut current_tags: Vec<String> = Vec::new();

            for comb in tag_comb {
                current_tags.push(tags[*comb].clone());
            }

            let mut op = if current_tags.contains(&"Top Operator".to_string()){
                self.operators.clone()
            } else {
                self.operators_without_top_operators.clone()
            };

            let mut min_rar = 6;
            // let mut max_rar = 1;
            let mut insert = false;

            for tag in &current_tags {
                op = op.into_iter().filter_map(|op| {
                    if op.contains(tag) {
                        Some(op)
                    } else {
                        None
                    }
                }).collect();
            }

            if !op.is_empty() {
                for o in &op {
                    // filter out robots because they are special
                    if o.rar < min_rar && o.rar != 1 {
                        min_rar = o.rar;
                    }
                    // if o.rar > max_rar {
                    //     max_rar = o.rar;
                    // }
                }
                let mut i = 0;
                for result in &results {
                    if result.operators == op {
                        insert = true;
                        break;
                    }
                    i += 1;
                }
                if insert {
                    results[i].tag_combs.push(current_tags);
                } else {
                    results.push( OpResult {
                        tag_combs: vec![current_tags],
                        operators: op,
                        min_rar,
                        // max_rar,
                    });
                }
            }

        }

    results.sort_by_key(|k| {match k.min_rar{
        // 1 => 6,
        // 2 => 5,
        3 => 4,
        4 => 3,
        5 => 2,
        6 => 1,
        _ => 0, // should never happen
    }});
    // for result in &results {
    //     std::println!("{:?} min: {} max: {}", result.tag_combs, result.min_rar, result.max_rar);
    //     for op in &result.operators {
    //         std::println!("- {} | rar: {}", op.name, op.rar);
    //     }
    // }
    results
    }
}

struct ArknightsRecruitHelper {
    mode: Mode,
    options: Options,
    vision: Vision,
    data: Data,
    operator_images: Vec<TextureHandle>,
    tag_image_1: TextureHandle,
    tag_image_2: TextureHandle,
    tag_image_3: TextureHandle,
    tag_image_4: TextureHandle,
    tag_image_5: TextureHandle,
    last_op_result: Vec<OpResult>,
    time_now: Instant,
    time_last: Instant,
}

impl ArknightsRecruitHelper {
    fn new(cc: &eframe::CreationContext<'_>, options: Options) -> Self {
        let start = Instant::now();
        let img = ColorImage::new([1, 1], Color32::BLACK);
        let dummy_image = cc.egui_ctx.load_texture("dummy", img, Default::default());

        let mut style = (cc.egui_ctx.style()).as_ref().clone();
        style.text_styles = [(TextStyle::Button, FontId::new(25.0, FontFamily::Proportional)),
                             (TextStyle::Body, FontId::new(25.0, FontFamily::Proportional)),
                             (TextStyle::Heading, FontId::new(25.0, FontFamily::Proportional)),
                             (TextStyle::Monospace, FontId::new(25.0, FontFamily::Proportional)),].into();
        cc.egui_ctx.set_style(style);

        let mut data = Data::new();
        let mut operator_images: Vec<TextureHandle> = Vec::new();

        for i in 0..data.operators.len() {
            let path = format!("./res/img/operator/{}.png", data.operators[i].name.as_str());
            let image = load_image_from_path(path.as_str());

            // let mut image = load_image_from_path(path.as_str()).unwrap_or_default();
            // let bg_color_by_rar = get_bg_color_by_rarity(data.operators[i].rar);
            // for pixel in image.pixels.iter_mut() {
            //     if pixel[3] == 0 {
            //         // pixel = gb_color_by_rar;
            //         pixel[0] = bg_color_by_rar[0];
            //         pixel[1] = bg_color_by_rar[1];
            //         pixel[2] = bg_color_by_rar[2];
            //         pixel[3] = bg_color_by_rar[3];
            //     }
            // }

            let image = cc.egui_ctx.load_texture(path.as_str(), image, Default::default());
            operator_images.push(image);
            data.operators[i].index = i;
        }

        let operators = data.operators.clone();
        data.operators_without_top_operators = operators.into_iter().filter_map(|operator| {
            if operator.rar != 6 {
                Some(operator)
            } else {
                None
            }
            }).collect();

        Self {
            mode: DisplayResults,
            options,
            vision: Vision::new(),
            data,
            operator_images,
            tag_image_1: dummy_image.clone(),
            tag_image_2: dummy_image.clone(),
            tag_image_3: dummy_image.clone(),
            tag_image_4: dummy_image.clone(),
            tag_image_5: dummy_image,
            last_op_result: Vec::new(),
            time_now: Instant::now(),
            time_last: start,
        }
    }

    fn render_top_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            Frame::none().show(ui, |ui| {
                // let title = format!("Arknights Recruit Helper Beta - FPS: {}", 1000 / self.time_now.checked_duration_since(self.time_last).expect("time error").as_millis());
                // std::println!("{}", 1000 / self.time_now.checked_duration_since(self.time_last).expect("time error").as_millis());
                let title = "Arknights Recruit Helper";
                let text_color = self.options.title_color;
                let height = 35.0;

                let mut rect = ui.max_rect();
                rect.min.y += 5.0;
                rect.max.y = rect.min.y + height;

                let painter = ui.painter();

                // Paint the frame:
                painter.rect(
                    rect,
                    10.0,
                    ctx.style().visuals.window_fill(),
                    Stroke::new(1.0, text_color),
                );

                // Paint the title:
                painter.text(
                    rect.center_top() + vec2(0.0, height / 2.0),
                    Align2::CENTER_CENTER,
                    title,
                    FontId::proportional(height - 2.0),
                    text_color,
                );
            //
            // // Paint the line under the title:
            // painter.line_segment(
            //     [
            //         rect.left_top() + vec2(2.0, height),
            //         rect.right_top() + vec2(-2.0, height),
            //     ],
            //     Stroke::new(1.0, text_color),
            // );

            let title_bar_response =
                ui.interact(rect, Id::new("title_bar"), Sense::click());
            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
                let win_pos = eframe::Frame::info(frame).window_info.position.unwrap();
                self.options.initial_window_pos = (win_pos.x, win_pos.y);
            }

            // Add the close button:
            let close_response = ui.put(
                Rect::from_min_size(rect.right_top(), Vec2::splat(-height)),
                Button::new(RichText::new("❌").size(height - 4.0)).frame(false),
            );
            if close_response.clicked() {
                frame.close();
            }

            });

            let image_height = 80.0;
            let width = (ui.available_width() - (2.0 * PADDING)) / 3.0;
            let box_width = width - 8.0;

            ui.add_space(PADDING);
            Grid::new("my_grid")
                .num_columns(3)
                // .spacing([PADDING, PADDING])
                .striped(false)
                .show(ui, |ui| {
                    ui.image(&self.tag_image_1, [width, image_height]);
                    ui.image(&self.tag_image_2, [width, image_height]);
                    ui.image(&self.tag_image_3, [width, image_height]);
                    ui.end_row();
                    self.draw_combo_box(ui, 0, box_width);
                    self.draw_combo_box(ui, 1, box_width);
                    self.draw_combo_box(ui, 2, box_width);
                    ui.end_row();
                    ui.add_space(PADDING / 2.0);
                    ui.end_row();
                    ui.image(&self.tag_image_4, [width, image_height]);
                    ui.image(&self.tag_image_5, [width, image_height]);
                    ui.end_row();
                    self.draw_combo_box(ui, 3, box_width);
                    self.draw_combo_box(ui, 4, box_width);
                    ui.label(format!("{} ms", self.vision.last_time))
                });
            ui.add_space(PADDING);
        });
    }

    fn render_options(&mut self, ctx: &Context) {
        self.update_tag_image(ctx);
        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(PADDING);
            ui.label("If Arknights Recruit Helper is not able to find your emulator:\n\n\
            1: Press the \"Find Emulator Window\" button.\n\
            2: Click into the window of your Arknights emulator.\n\
            3: Do 1st and 2nd within 3 seconds.\n");
            if ui.button("Find Emulator Window").clicked() {
                self.options.window_name = ArknightsRecruitHelper::find_emulator_window();
            }
            ui.label("\nUse the 4 sliders below to adjust the tag images.");
            Grid::new("my_grid")
                // .num_columns(5)
                // .spacing([40.0, 4.0])
                .striped(false)
                .show(ui, |ui| {
                    ui.style_mut().spacing.slider_width = 500.0;
                    ui.end_row();
                    ui.label("left:");
                    ui.add(Slider::new(&mut self.options.tag_pos.0, 0.3071..=0.2671).show_value(false));
                    ui.end_row();
                    ui.label("top:");
                    ui.add(Slider::new(&mut self.options.tag_pos.1, 0.4965..=0.5365).show_value(false));
                    ui.end_row();
                    ui.label("bottom:");
                    ui.add(Slider::new(&mut self.options.tag_pos.2, 0.5588..=0.5988).show_value(false));
                    ui.end_row();
                    ui.label("right:");
                    ui.add(Slider::new(&mut self.options.tag_pos.3, 0.4175..=0.3775).show_value(false));
                });

            ui.add_space(PADDING);
            ui.horizontal(|ui| {
                ui.label("Title Color:");
                ui.color_edit_button_srgba(&mut self.options.title_color);
            })



        });
    }

    fn render_bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(PADDING);
            ui.horizontal(|ui| {
                let width = (ui.available_width() - PADDING) / 2.0;

                let mut style = (*ctx.style()).clone();
                style.text_styles = [(TextStyle::Button, FontId::new(45.0, FontFamily::Proportional))].into();
                ui.style_mut().text_styles = style.text_styles;

                if ui.add_sized([width, 60.0], Button::new("Scan")).clicked() {
                    self.clicked_scan_button(ctx);
                }
                if ui.add_sized([width, 60.0], Button::new("Option")).clicked() {
                    self.clicked_option_button();
                }
            });
            ui.add_space(PADDING);
        });
    }

    fn render_result(&mut self, ctx: &Context) {
        self.update_tag_image(ctx);
        if self.last_op_result.is_empty() {return;}

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                // Start
                Grid::new("result_grid")
                    .num_columns(2)
                    .spacing([5.0, 20.0])
                    .min_col_width(285.0)
                    .striped(true)
                    .show(ui, |ui| {
                        for (cycle, result) in self.last_op_result.iter().enumerate() {

                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    for tags in result.tag_combs.iter() {
                                        ui.horizontal(|ui| {
                                            for tag in tags {
                                                ui.label(RichText::new(tag)
                                                    .background_color(LABELBGCOLOR)
                                                    .size(17.0));
                                            }
                                        });
                                    }
                                });
                            });

                            let mut min_rar_group: Vec<&Operator> = Vec::new();
                            let mut remaining_group: Vec<&Operator> = Vec::new();
                            let mut robot_group: Vec<&Operator> = Vec::new();

                            for operator in result.operators.iter() {
                                if operator.rar == result.min_rar {
                                    min_rar_group.push(operator);
                                    continue
                                }

                                if operator.rar == 1 {
                                    robot_group.push(operator);
                                    continue
                                }
                                remaining_group.push(operator);
                                // if ui.add(ImageButton::new(&self.operator_images[operator.index], [60.0, 60.0]))
                                //     .on_hover_text(&operator.name)
                                //     .clicked() {
                                //     ui.ctx().output().open_url = Some(output::OpenUrl {
                                //         url: format!("https://aceship.github.io/AN-EN-Tags/akhrchars.html?opname={}", operator.name),
                                //         new_tab: true,
                                //     });
                                // }

                            }
                            remaining_group.sort_by_key(|x| x.rar);

                            ui.group(|ui| {
                                let mut i = 0;
                                ui.vertical(|ui| {
                                    Grid::new(format!("operator_grid_big_{cycle}"))
                                        .spacing([2.0, 2.0])
                                        .show(ui, |ui| {
                                            for operator in &min_rar_group {
                                                if ui.add(custom_widgets::ImageButton::new(self.operator_images[operator.index].id(), [60.0, 60.0], get_bg_color_by_rarity(operator.rar)))
                                                    .on_hover_text(&operator.name)
                                                    .clicked() {
                                                        ui.ctx().output_mut(|o| {
                                                            o.open_url = Some(output::OpenUrl {
                                                                url: format!("https://aceship.github.io/AN-EN-Tags/akhrchars.html?opname={}", operator.name),
                                                                new_tab: true,
                                                            });
                                                        });
                                                    }
                                                i += 1;
                                                if i > 4 {
                                                    ui.end_row();
                                                    i = 0;
                                                }
                                            } // end min_rar_group
                                            ui.end_row();
                                            i = 0;
                                            if !robot_group.is_empty() {
                                                for operator in &robot_group {
                                                    if ui.add(custom_widgets::ImageButton::new(self.operator_images[operator.index].id(), [60.0, 60.0], get_bg_color_by_rarity(operator.rar)))
                                                        .on_hover_text(&operator.name)
                                                        .clicked() {
                                                        ui.ctx().output_mut(|o| {
                                                            o.open_url = Some(output::OpenUrl {
                                                                url: format!("https://aceship.github.io/AN-EN-Tags/akhrchars.html?opname={}", operator.name),
                                                                new_tab: true,
                                                            });
                                                        });
                                                    }
                                                    i += 1;
                                                    if i > 4 {
                                                        ui.end_row();
                                                        i = 0;
                                                    }
                                                } // end robot_group
                                            } // end if robot_group
                                        }); // end "operator_grid_big_{cycle}"
                                    i = 0;
                                    if !remaining_group.is_empty() {
                                        Grid::new(format!("operator_grid_small_{cycle}"))
                                            .spacing([0.3, 0.3])
                                            .show(ui, |ui| {
                                                for operator in &remaining_group {
                                                    if ui.add(custom_widgets::ImageButton::new(self.operator_images[operator.index].id(), [41.75, 41.75], get_bg_color_by_rarity(operator.rar)))
                                                        .on_hover_text(&operator.name)
                                                        .clicked() {
                                                        ui.ctx().output_mut(|o| {
                                                            o.open_url = Some(output::OpenUrl {
                                                                url: format!("https://aceship.github.io/AN-EN-Tags/akhrchars.html?opname={}", operator.name),
                                                                new_tab: true,
                                                            });
                                                        });
                                                    }
                                                    i += 1;
                                                    if i > 6 {
                                                        ui.end_row();
                                                        i = 0;
                                                    }
                                                } // end remaining_group
                                            }); // end "operator_grid_small_{cycle}"
                                    } // end if remaining_group
                                }); // end vertical
                            }); // end group
                            ui.end_row();
                            // ui.add_space(20.0);
                            // ui.end_row();
                            } // end Result
                    }); // end result_grid
                // End
                ui.add_space(200.0);
                }); // end ScrollArea
            // ui.add_space(PADDING);
        });
    }

    fn clicked_scan_button(&mut self, ctx: &Context) {
        let start_time = Instant::now();
        self.mode = DisplayResults;

        let mut image_1 = Arc::new((ColorImage::new([1, 1], Color32::BLACK), "".to_string()));
        let mut image_2 = Arc::new((ColorImage::new([1, 1], Color32::BLACK), "".to_string()));
        let mut image_3 = Arc::new((ColorImage::new([1, 1], Color32::BLACK), "".to_string()));
        let mut image_4 = Arc::new((ColorImage::new([1, 1], Color32::BLACK), "".to_string()));
        let mut image_5 = Arc::new((ColorImage::new([1, 1], Color32::BLACK), "".to_string()));

        std::thread::scope(|s| {
            s.spawn(|| {
                image_1 = Arc::new(self.vision.get_image_and_tag(self.options.window_name.as_str(), self.options.get_pos(1)));
            });

            s.spawn(|| {
                image_2 = Arc::new(self.vision.get_image_and_tag(self.options.window_name.as_str(), self.options.get_pos(2)));
            });

            s.spawn(|| {
                image_3 = Arc::new(self.vision.get_image_and_tag(self.options.window_name.as_str(), self.options.get_pos(3)));
            });

            s.spawn(|| {
                image_4 = Arc::new(self.vision.get_image_and_tag(self.options.window_name.as_str(), self.options.get_pos(4)));
            });

            s.spawn(|| {
                image_5 = Arc::new(self.vision.get_image_and_tag(self.options.window_name.as_str(), self.options.get_pos(5)));
            });
        });

        let (image_1, tag_1) = Arc::try_unwrap(image_1).unwrap_or_default();
        let (image_2, tag_2) = Arc::try_unwrap(image_2).unwrap_or_default();
        let (image_3, tag_3) = Arc::try_unwrap(image_3).unwrap_or_default();
        let (image_4, tag_4) = Arc::try_unwrap(image_4).unwrap_or_default();
        let (image_5, tag_5) = Arc::try_unwrap(image_5).unwrap_or_default();

        self.vision.current_tags[0] = tag_1;
        self.vision.current_tags[1] = tag_2;
        self.vision.current_tags[2] = tag_3;
        self.vision.current_tags[3] = tag_4;
        self.vision.current_tags[4] = tag_5;

        self.tag_image_1 = ctx.load_texture("tag_image_1", image_1, Default::default());
        self.tag_image_2 = ctx.load_texture("tag_image_2", image_2, Default::default());
        self.tag_image_3 = ctx.load_texture("tag_image_3", image_3, Default::default());
        self.tag_image_4 = ctx.load_texture("tag_image_4", image_4, Default::default());
        self.tag_image_5 = ctx.load_texture("tag_image_5", image_5, Default::default());

        self.last_op_result = self.data.get_raw_possible_combinations(&self.vision.current_tags);
        self.vision.last_time = Instant::now().duration_since(start_time).as_millis();
    }

    fn update_tag_image(&mut self, ctx: &Context) {
        self.tag_image_1 = ctx.load_texture(
            "tag_image_1",
            self.vision.get_image(self.options.window_name.as_str(), self.options.get_pos(1)),
            Default::default()
        );

        self.tag_image_2 = ctx.load_texture(
            "tag_image_2",
            self.vision.get_image(self.options.window_name.as_str(), self.options.get_pos(2)),
            Default::default()
        );

        self.tag_image_3 = ctx.load_texture(
            "tag_image_3",
            self.vision.get_image(self.options.window_name.as_str(), self.options.get_pos(3)),
            Default::default()
        );

        self.tag_image_4 = ctx.load_texture(
            "tag_image_4",
            self.vision.get_image(self.options.window_name.as_str(), self.options.get_pos(4)),
            Default::default()
        );

        self.tag_image_5 = ctx.load_texture(
            "tag_image_5",
            self.vision.get_image(self.options.window_name.as_str(), self.options.get_pos(5)),
            Default::default()
        );
    }

    fn clicked_option_button(&mut self) {
        self.mode = DisplayOptions;
        // let (buffer, width, height) = Vision::get_screenshot_(self.options.window_name.as_str());
        //
        // // how to save
        // let i = std::fs::read_dir("./").unwrap().count() - 6;
        // let path = format!("test{i}.png");
        // image::save_buffer_with_format(path, &buffer, width as u32, height as u32, image::ColorType::Rgba8, image::ImageFormat::Png).unwrap();

        // use win_screenshot::capture::capture_window;
        // use win_screenshot::addon::find_window;
        //
        // let s = capture_window(win_screenshot::addon::find_window("LDPlayer").unwrap(),
        //                        win_screenshot::capture::Area::ClientOnly).unwrap();
        //
        // s.save("screenshot.png").unwrap();

        // for tag in &self.data.tags {
        //     std::println!("{tag}");
        // }
        //
        // for operator in &self.data.operators {
        //     std::println!("{}", operator.name);
        // }
    }

    fn draw_combo_box(&mut self, ui: &mut Ui, id: usize, box_width: f32) {
            // let mut selected_text = LayoutJob::default();
            // selected_text.append(&tag_combobox, 0.0,
            //     TextFormat {
            //         font_id: FontId::new(25.0, FontFamily::Proportional),
            //         color: Color32::WHITE,
            //         ..Default::default()
            //         },
            //     );
            // selected_text.halign = Align::Center;

            ComboBox::width(ComboBox::from_id_source(id), box_width)
                .selected_text(self.vision.current_tags[id].clone())
                .show_ui(ui, |ui| {
                    for tag in &self.data.tags {
                        ui.selectable_value(&mut self.vision.current_tags[id], tag.clone(), tag);
                    }
                });
    }

    // fn draw_operator(&mut self, ui: &mut Ui, operator: &Operator, size: f32) {
    //        if ui.add(ImageButton::new(&self.operator_images[operator.index], [size, size]))
    //             .on_hover_text(&operator.name)
    //             .clicked() {
    //                 ui.ctx().output().open_url = Some(output::OpenUrl {
    //                     url: format!("https://aceship.github.io/AN-EN-Tags/akhrchars.html?opname={}", operator.name),
    //                     new_tab: true,
    //             });
    //        }
    //
    // }

    fn check_for_changes_tags(&mut self) {
        if self.vision.current_tags != self.vision.last_tags {
            self.vision.last_tags = self.vision.current_tags.clone();
            self.last_op_result = self.data.get_raw_possible_combinations(&self.vision.current_tags);
        }
    }

    fn find_emulator_window() -> String {
        std::thread::sleep(std::time::Duration::from_secs(3));
        Vision::get_window_name_of_foreground_window()
    }
}

impl eframe::App for ArknightsRecruitHelper {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        self.check_for_changes_tags();
        self.render_top_panel(ctx, frame);
        self.render_bottom_panel(ctx);

        match self.mode {
            DisplayResults => self.render_result(ctx),
            DisplayOptions => self.render_options(ctx),
        }

        ctx.request_repaint();

        self.time_last = self.time_now;
        self.time_now = Instant::now();

        std::thread::sleep(NAPTIME);

    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let to_save = serde_json::to_string(&self.options).unwrap();
        // I can't use replace() because it is overwritten by TextBuffer
        let to_save = to_save.replacen(",\"", ",\n\"", 999);
        let mut file = std::fs::File::create("./res/opt.txt").unwrap();
        file.write_all(to_save.as_bytes()).unwrap();
    }

    // fn clear_color(&self, _visuals: &Visuals) -> Rgba {
    //     Rgba::TRANSPARENT // Make sure we don't paint anything behind the rounded corners
    // }
}

fn get_bg_color_by_rarity(rar: i32) -> Color32 {
    match rar {
        6 => Color32::from_rgb(255, 102,   0),
        5 => Color32::from_rgb(255, 174,   0),
        4 => Color32::from_rgb(219, 177, 219),
        3 => Color32::from_rgb(  0, 178, 246),
        2 => Color32::from_rgb(220, 229,  55),
        1 => Color32::from_gray(159),
        _ => Color32::from_gray(0),  // should never happen
    }
}

fn load_image_from_path(path: &str) -> ColorImage {
    let img_path = std::path::Path::new(path);
    let open_path_result = image::io::Reader::open(img_path);
    let image_result = match open_path_result {
        Ok(open_path) => open_path.decode(),
        Err(_) => return ColorImage::new([1, 1], Color32::BLACK),
    };
    let image = match image_result {
        Ok(image) => image,
        Err(_) => return ColorImage::new([1, 1], Color32::BLACK),
    };
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
}

fn load_icon(path: &str) -> eframe::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

fn main() {
    let app_option = Options::new();
    let win_option = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(app_option.initial_window_size.0, app_option.initial_window_size.1)),
        initial_window_pos: Some(Pos2::new(app_option.initial_window_pos.0, app_option.initial_window_pos.1)),
        resizable: false,
        always_on_top: true,
        icon_data: Some(load_icon("./res/img/Icon.png")),
        decorated: false,
        // transparent: true,
        ..Default::default()
    };

    eframe::run_native(
        "Arknights Recrut Helper",
        win_option,
        Box::new(|cc| Box::new(ArknightsRecruitHelper::new(cc, app_option)))).expect("TODO: panic message");
}