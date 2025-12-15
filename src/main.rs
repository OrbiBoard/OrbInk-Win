use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};

#[derive(Clone)]
struct StrokePoint {
    x: f32,
    y: f32,
    pressure: f32,
}

struct Annotator {
    points: Vec<StrokePoint>,
    text: SharedString,
}

impl Annotator {
    fn new() -> Self {
        Self {
            points: Vec::new(),
            text: "OrbInk".into(),
        }
    }
}

impl Render for Annotator {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let canvas = gpui::canvas(|gfx| {
            let color = rgb(0x00ff00);
            for w in self.points.windows(2) {
                let a = &w[0];
                let b = &w[1];
                let width = ((a.pressure + b.pressure) * 0.5 * 12.0).max(1.0);
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                let nx = -dy / len * width * 0.5;
                let ny = dx / len * width * 0.5;
                let p0 = gpui::Point { x: a.x - nx, y: a.y - ny };
                let p1 = gpui::Point { x: a.x + nx, y: a.y + ny };
                let p2 = gpui::Point { x: b.x + nx, y: b.y + ny };
                let p3 = gpui::Point { x: b.x - nx, y: b.y - ny };
                gfx.fill_quad(p0, p1, p2, p3, color);
            }
        })
        .bg(rgb(0x1e1e1e))
        .size_full()
        .on_mouse_down(|this, e, _| {
            let p = StrokePoint { x: e.position.x, y: e.position.y, pressure: e.pressure.unwrap_or(1.0) as f32 };
            this.points.clear();
            this.points.push(p);
        })
        .on_mouse_move(|this, e, _| {
            if e.buttons.primary {
                let p = StrokePoint { x: e.position.x, y: e.position.y, pressure: e.pressure.unwrap_or(1.0) as f32 };
                this.points.push(p);
            }
        })
        .on_mouse_up(|_this, _e, _| {});

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x2b2b2b))
                    .text_color(rgb(0xffffff))
                    .child(format!("{}", &self.text)),
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

