use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, MouseButton, PathBuilder, Pixels,
    SharedString, point, Window, WindowBounds, WindowOptions,
};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct StrokePoint {
    x: f32,
    y: f32,
    pressure: f32,
}

#[derive(Clone)]
struct Stroke {
    points: Vec<StrokePoint>,
    color: u32,
    size: f32,
    path: Option<gpui::Path<Pixels>>,
}

struct Annotator {
    strokes: Vec<Stroke>,
    text: SharedString,
    brush_size: f32,
    brush_color: u32,
    is_drawing: bool,
    last_sample_time: Option<Instant>,
    sample_interval: Duration,
    min_sample_distance: f32,
}

impl Annotator {
    fn new() -> Self {
        Self {
            strokes: Vec::new(),
            text: "OrbInk".into(),
            brush_size: 1.0,
            brush_color: 0x00ff00,
            is_drawing: false,
            last_sample_time: None,
            sample_interval: Duration::from_millis(16),
            min_sample_distance: 1.5,
        }
    }

    fn build_path_from(stroke: &Stroke) -> Option<gpui::Path<Pixels>> {
        if stroke.points.len() < 2 {
            return None;
        }
        let mut builder = PathBuilder::fill();
        for w in stroke.points.windows(2) {
            let a = &w[0];
            let b = &w[1];
            let width = ((a.pressure + b.pressure) * 0.5 * 12.0 * stroke.size).max(1.0);
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let nx = -dy / len * width * 0.5;
            let ny = dx / len * width * 0.5;
            let p0 = point(px(a.x - nx), px(a.y - ny));
            let p1 = point(px(a.x + nx), px(a.y + ny));
            let p2 = point(px(b.x + nx), px(b.y + ny));
            let p3 = point(px(b.x - nx), px(b.y - ny));
            builder.add_polygon(&[p0, p1, p2, p3], true);
        }
        builder.build().ok()
    }

    fn append_segment(path: &mut gpui::Path<Pixels>, a: &StrokePoint, b: &StrokePoint, size: f32) {
        let width = ((a.pressure + b.pressure) * 0.5 * 12.0 * size).max(1.0);
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let nx = -dy / len * width * 0.5;
        let ny = dx / len * width * 0.5;
        let p0 = point(px(a.x - nx), px(a.y - ny));
        let p1 = point(px(a.x + nx), px(a.y + ny));
        let p2 = point(px(b.x + nx), px(b.y + ny));
        let p3 = point(px(b.x - nx), px(b.y - ny));
        path.push_triangle((p0, p1, p2), (point(0., 1.), point(0., 1.), point(0., 1.)));
        path.push_triangle((p0, p2, p3), (point(0., 1.), point(0., 1.), point(0., 1.)));
    }
}

impl Render for Annotator {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let draws: Vec<(gpui::Path<Pixels>, u32)> = self
            .strokes
            .iter()
            .filter_map(|s| s.path.clone().map(|p| (p, s.color)))
            .collect();
        let canvas = gpui::canvas(
            move |_bounds, _window, _cx| draws.clone(),
            move |_, draws, window, _| {
                for (path, color) in draws {
                    window.paint_path(path, rgb(color));
                }
            },
        )
        .bg(rgb(0x1e1e1e))
        .size_full();

        div()
            .flex()
            .flex_col()
            .size_full()
            .on_any_mouse_down(cx.listener(|this, e: &gpui::MouseDownEvent, _window, _cx| {
                let p = StrokePoint { x: e.position.x.into(), y: e.position.y.into(), pressure: 1.0 };
                this.strokes.push(Stroke { points: vec![p], color: this.brush_color, size: this.brush_size, path: None });
                this.is_drawing = true;
                this.last_sample_time = Some(Instant::now());
            }))
            .on_mouse_move(cx.listener(|this, e: &gpui::MouseMoveEvent, _window, _cx| {
                if this.is_drawing || e.dragging() {
                    let now = Instant::now();
                    let p = StrokePoint { x: e.position.x.into(), y: e.position.y.into(), pressure: 1.0 };
                    if let Some(stroke) = this.strokes.last_mut() {
                        let should_sample = if let Some(last) = stroke.points.last() {
                            let dx = p.x - last.x;
                            let dy = p.y - last.y;
                            let dist2 = dx * dx + dy * dy;
                            dist2 >= this.min_sample_distance * this.min_sample_distance
                        } else {
                            true
                        };
                        let time_ok = this
                            .last_sample_time
                            .map(|t| now.duration_since(t) >= this.sample_interval)
                            .unwrap_or(true);
                        if !(should_sample || time_ok) {
                            return;
                        }
                        this.last_sample_time = Some(now);
                        stroke.points.push(p);
                        let n = stroke.points.len();
                        if n >= 2 {
                            let a = &stroke.points[n - 2];
                            let b = &stroke.points[n - 1];
                            if stroke.path.is_none() {
                                let start = point(px(a.x), px(a.y));
                                stroke.path = Some(gpui::Path::new(start));
                            }
                            if let Some(ref mut path) = stroke.path {
                                Annotator::append_segment(path, a, b, stroke.size);
                            }
                        }
                    }
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _e: &gpui::MouseUpEvent, _window, _cx| {
                if let Some(stroke) = this.strokes.last_mut() {
                    stroke.points.clear();
                }
                this.is_drawing = false;
                this.last_sample_time = None;
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x2b2b2b))
                    .text_color(rgb(0xffffff))
                    .child(format!("{}", &self.text))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(rgb(0x3a3a3a))
                                    .hover(|s| s.bg(rgb(0x4a4a4a)))
                                    .child("清空")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e: &gpui::MouseDownEvent, _w, _cx| {
                                            this.strokes.clear();
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(rgb(0x3a3a3a))
                                    .hover(|s| s.bg(rgb(0x4a4a4a)))
                                    .child("粗细-")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e: &gpui::MouseDownEvent, _w, _cx| {
                                            this.brush_size = (this.brush_size - 0.2).max(0.2);
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(rgb(0x3a3a3a))
                                    .hover(|s| s.bg(rgb(0x4a4a4a)))
                                    .child("粗细+")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e: &gpui::MouseDownEvent, _w, _cx| {
                                            this.brush_size = (this.brush_size + 0.2).min(5.0);
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(rgb(0x3a3a3a))
                                    .hover(|s| s.bg(rgb(0x4a4a4a)))
                                    .child("绿色")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e: &gpui::MouseDownEvent, _w, _cx| {
                                            this.brush_color = 0x00ff00;
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(rgb(0x3a3a3a))
                                    .hover(|s| s.bg(rgb(0x4a4a4a)))
                                    .child("蓝色")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e: &gpui::MouseDownEvent, _w, _cx| {
                                            this.brush_color = 0x3399ff;
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(rgb(0x3a3a3a))
                                    .hover(|s| s.bg(rgb(0x4a4a4a)))
                                    .child("红色")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e: &gpui::MouseDownEvent, _w, _cx| {
                                            this.brush_color = 0xff3333;
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .ml_2()
                                    .child(format!("粗细: {:.1}", self.brush_size)),
                            ),
                    ),
            )
            .child(canvas)
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Annotator::new()),
        )
        .unwrap();
    });
}

