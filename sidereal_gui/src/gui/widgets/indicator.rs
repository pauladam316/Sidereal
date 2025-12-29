use crate::gui::styles;
use iced::{
    mouse,
    widget::canvas::{self, Cache, Geometry, Path, Program, Stroke},
    Color, Length, Point, Rectangle, Renderer, Theme,
};

/// Indicator color state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndicatorColor {
    #[allow(dead_code)]
    Green,
    #[allow(dead_code)]
    Yellow,
    Red,
}

/// Indicator widget - displays a colored light indicator
pub struct Indicator {
    color: IndicatorColor,
    size: f32,
    cache: Cache,
}

impl Indicator {
    pub fn new(color: IndicatorColor) -> Self {
        Self {
            color,
            size: 16.0,
            cache: Cache::new(),
        }
    }

    #[allow(dead_code)]
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self.cache.clear();
        self
    }
}

impl<Message> Program<Message> for Indicator {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            let radius = self.size.min(bounds.width.min(bounds.height) / 2.0 - 2.0);

            // Get the color based on state
            let color = match self.color {
                IndicatorColor::Green => styles::GREEN_INDICATOR_COLOR,
                IndicatorColor::Yellow => styles::AMBER_INDICATOR_COLOR,
                IndicatorColor::Red => styles::RED_INDICATOR_COLOR,
            };

            // Draw the light circle
            let light_path = Path::circle(center, radius);
            frame.fill(&light_path, color);

            // Add a small subtle highlight for shine effect
            let highlight_radius = radius * 0.4;
            let highlight_offset = Point::new(-radius * 0.25, -radius * 0.25);
            let highlight_path = Path::circle(
                Point::new(center.x + highlight_offset.x, center.y + highlight_offset.y),
                highlight_radius,
            );
            frame.fill(
                &highlight_path,
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.2, // Subtle shine
                },
            );

            // Add border with standard border color
            frame.stroke(
                &light_path,
                Stroke::default()
                    .with_width(1.0)
                    .with_color(styles::ELEMENT_BORDER),
            );
        });

        vec![geometry]
    }
}

/// Create an indicator widget
pub fn indicator<'a, Message>(color: IndicatorColor) -> canvas::Canvas<Indicator, Message>
where
    Message: 'a + Clone + 'static,
{
    canvas::Canvas::new(Indicator::new(color))
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(20.0))
}
