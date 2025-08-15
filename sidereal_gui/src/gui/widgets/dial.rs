// dial.rs
use iced::mouse;
use iced::widget::canvas as canvas_widget;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text};
use iced::{alignment, Color, Point, Rectangle, Renderer, Size, Theme};

/// A circular dial with:
/// - tick marks
/// - filled arrow for `current`
/// - outlined arrow for `setpoint` (drag to move)
pub struct Dial<Message> {
    current_deg: f32,
    setpoint_deg: f32,
    on_setpoint: Box<dyn Fn(f32) -> Message + Send + Sync + 'static>,
    static_cache: Cache,
}

impl<Message: Clone + 'static> Dial<Message> {
    pub fn new<F>(
        current_deg: f32,
        setpoint_deg: f32,
        on_setpoint: F,
    ) -> canvas_widget::Canvas<Self, Message>
    where
        F: Fn(f32) -> Message + Send + Sync + 'static,
    {
        canvas_widget::Canvas::new(Self {
            current_deg: wrap_deg(current_deg),
            setpoint_deg: wrap_deg(setpoint_deg),
            on_setpoint: Box::new(on_setpoint),
            static_cache: Cache::new(),
        })
    }

    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.static_cache.clear();
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DialState {
    dragging: bool,
}

impl<Message> canvas_widget::Program<Message> for Dial<Message>
where
    Message: Clone + 'static,
{
    type State = DialState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let size = Size::new(bounds.width, bounds.height);

        // ----- background (cached) -----
        let bg = self.static_cache.draw(renderer, size, |frame| {
            let center = frame.center();
            let radius = size.width.min(size.height) * 0.42;

            // Dial base ring
            let dial_circle = Path::circle(center, radius);
            frame.stroke(
                &dial_circle,
                Stroke::default()
                    .with_width(2.0)
                    .with_color(Color::from_rgba(0.7, 0.7, 0.7, 1.0)),
            );

            // Ticks (every 10°, thicker every 30°)
            for deg in (0..360).step_by(10) {
                let (inner, outer, width, alpha) = if deg % 30 == 0 {
                    (radius * 0.78, radius * 0.96, 3.0, 1.0)
                } else {
                    (radius * 0.86, radius * 0.96, 1.5, 0.8)
                };

                let a = deg as f32;
                let p1 = polar(center, inner, a);
                let p2 = polar(center, outer, a);
                let tick = Path::line(p1, p2);
                frame.stroke(
                    &tick,
                    Stroke::default()
                        .with_width(width)
                        .with_color(Color::from_rgba(0.6, 0.6, 0.6, alpha)),
                );
            }
        });

        // ----- foreground (dynamic: arrows + text) -----
        let mut fg_frame = Frame::new(renderer, size);
        let center = Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );
        let radius = size.width.min(size.height) * 0.42;

        // Current arrow (filled)
        draw_arrow(
            &mut fg_frame,
            center,
            radius * 0.9,
            self.current_deg,
            6.0,
            Color::from_rgb(0.20, 0.55, 0.95),
        );

        // Setpoint arrow (outline)
        draw_arrow_outline(
            &mut fg_frame,
            center,
            radius * 0.9,
            self.setpoint_deg,
            6.0,
            Color::from_rgb(0.95, 0.40, 0.20),
        );

        // Center text (current)
        let mut text = Text {
            content: format!("{:.1}°", self.current_deg),
            position: Point::new(center.x, center.y + 4.0),
            size: iced::Pixels(28.0),
            color: Color::from_rgb(0.9, 0.9, 0.9),
            ..Text::default()
        };
        text.horizontal_alignment = alignment::Horizontal::Center;
        text.vertical_alignment = alignment::Vertical::Center;
        fg_frame.fill_text(text);

        // Legend (setpoint)
        let mut legend_text = Text {
            content: format!("setpoint: {:.1}°", self.setpoint_deg),
            position: Point::new(center.x, center.y + 28.0),
            size: iced::Pixels(16.0),
            color: Color::from_rgba(0.8, 0.8, 0.8, 0.85),
            ..Text::default()
        };
        legend_text.horizontal_alignment = alignment::Horizontal::Center;
        fg_frame.fill_text(legend_text);

        let fg = fg_frame.into_geometry();

        vec![bg, fg]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let center = Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );
        let radius = bounds.width.min(bounds.height) * 0.42;

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let d = distance(pos, center);
                    if d <= radius * 1.1 && d >= radius * 0.55 {
                        state.dragging = true;
                        let angle = point_angle_deg(center, pos);
                        return (
                            canvas::event::Status::Captured,
                            Some((self.on_setpoint)(angle)),
                        );
                    }
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging {
                    if let Some(pos) = cursor.position_in(bounds) {
                        let angle = point_angle_deg(center, pos);
                        return (
                            canvas::event::Status::Captured,
                            Some((self.on_setpoint)(angle)),
                        );
                    }
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging {
                    state.dragging = false;
                    return (canvas::event::Status::Captured, None);
                }
            }
            _ => {}
        }

        (canvas::event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            return mouse::Interaction::Grabbing;
        }
        if let Some(pos) = cursor.position_in(bounds) {
            let center = Point::new(
                bounds.x + bounds.width / 2.0,
                bounds.y + bounds.height / 2.0,
            );
            let radius = bounds.width.min(bounds.height) * 0.42;
            let d = distance(pos, center);
            if d <= radius * 1.1 && d >= radius * 0.55 {
                return mouse::Interaction::Grab;
            }
        }
        mouse::Interaction::Idle
    }
}

/* ---------- helpers ---------- */

fn wrap_deg(a: f32) -> f32 {
    let mut x = a % 360.0;
    if x < 0.0 {
        x += 360.0;
    }
    x
}

// 0° at top, increasing clockwise (screen coords)
fn polar(center: Point, r: f32, deg: f32) -> Point {
    let rad = (deg - 90.0).to_radians();
    Point::new(center.x + r * rad.cos(), center.y + r * rad.sin())
}

fn distance(a: Point, b: Point) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

fn point_angle_deg(center: Point, p: Point) -> f32 {
    let dx = p.x - center.x;
    let dy = p.y - center.y;
    let rad = dy.atan2(dx); // -pi..pi, 0 at +X, CCW
    let deg = rad.to_degrees(); // -180..180
    wrap_deg(deg + 90.0) // 0° at top, clockwise positive
}

fn draw_arrow(frame: &mut Frame, center: Point, r: f32, deg: f32, width: f32, color: Color) {
    let tip = polar(center, r, deg);
    let base = polar(center, r * 0.65, deg);
    let left = polar(center, r * 0.78, deg - 6.0);
    let right = polar(center, r * 0.78, deg + 6.0);

    let path = Path::new(|b| {
        b.move_to(base);
        b.line_to(left);
        b.line_to(tip);
        b.line_to(right);
        b.close();
    });

    frame.fill(&path, color);

    // optional stem
    let stem = Path::line(base, tip);
    frame.stroke(
        &stem,
        Stroke::default().with_width(width * 0.3).with_color(color),
    );
}

fn draw_arrow_outline(
    frame: &mut Frame,
    center: Point,
    r: f32,
    deg: f32,
    width: f32,
    color: Color,
) {
    let tip = polar(center, r, deg);
    let base = polar(center, r * 0.65, deg);
    let line = Path::line(base, tip);
    frame.stroke(&line, Stroke::default().with_width(width).with_color(color));
}
