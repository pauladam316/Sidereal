use iced::{
    alignment, mouse,
    widget::canvas::{self, Cache, Geometry, Path, Program, Stroke, Text},
    Color, Point, Rectangle, Renderer, Size, Theme,
};
use std::collections::VecDeque;

/// A data point in the plot
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub timestamp: f64,
    pub value: f64,
}

/// Plot series data - stored in state
#[derive(Debug, Clone)]
pub struct PlotSeries {
    pub name: String,
    pub color: Color,
    pub data: VecDeque<DataPoint>,
}

impl PlotSeries {
    pub fn new(name: impl Into<String>, color: Color) -> Self {
        Self {
            name: name.into(),
            color,
            data: VecDeque::new(),
        }
    }

    pub fn add_point(&mut self, point: DataPoint, max_points: usize) {
        self.data.push_back(point);
        while self.data.len() > max_points {
            self.data.pop_front();
        }
    }
}

/// Plot data container - stores only data, no rendering state
#[derive(Debug, Clone)]
pub struct LivePlotData {
    pub series: Vec<PlotSeries>,
    pub max_points: usize,
    pub padding: f32,
}

impl LivePlotData {
    pub fn new(max_points: usize, padding: f32) -> Self {
        Self {
            series: Vec::new(),
            max_points,
            padding,
        }
    }

    pub fn add_series(&mut self, name: impl Into<String>, color: Color) -> usize {
        let id = self.series.len();
        self.series.push(PlotSeries::new(name, color));
        id
    }

    #[allow(dead_code)]
    pub fn series_mut(&mut self, index: usize) -> Option<&mut PlotSeries> {
        self.series.get_mut(index)
    }

    pub fn add_data_point(&mut self, series_index: usize, point: DataPoint) {
        if let Some(series) = self.series.get_mut(series_index) {
            series.add_point(point, self.max_points);
        }
    }
}

/// The canvas program
pub struct LivePlotProgram {
    data: LivePlotData,
    cache: Cache,
}

impl<Message> Program<Message> for LivePlotProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return vec![];
        }

        let size = bounds.size();
        // Add extra padding for axis labels
        let left_padding = self.data.padding + 50.0; // Space for Y-axis labels
        let right_padding = self.data.padding + 10.0; // Space for legend
        let top_padding = self.data.padding + 10.0;
        let bottom_padding = self.data.padding + 25.0; // Space for X-axis labels

        let plot_width = (bounds.width - left_padding - right_padding).max(1.0);
        let plot_height = (bounds.height - top_padding - bottom_padding).max(1.0);
        let plot_x = left_padding;
        let plot_y = top_padding;

        const TIME_WINDOW: f64 = 1800.0; // 30 minutes

        // Find max timestamp
        let mut absolute_max_time = f64::NEG_INFINITY;
        let mut has_any_data = false;
        for series in &self.data.series {
            if !series.data.is_empty() {
                has_any_data = true;
                for point in &series.data {
                    absolute_max_time = absolute_max_time.max(point.timestamp);
                }
            }
        }

        let window_start = if has_any_data && absolute_max_time != f64::NEG_INFINITY {
            absolute_max_time - TIME_WINDOW
        } else {
            0.0
        };

        // Calculate bounds
        let (min_val, max_val, min_time, max_time) =
            if !has_any_data || absolute_max_time == f64::NEG_INFINITY {
                (0.0, 100.0, 0.0, 100.0)
            } else {
                let mut min_val = f64::INFINITY;
                let mut max_val = f64::NEG_INFINITY;
                let mut min_time = f64::INFINITY;
                let mut max_time = f64::NEG_INFINITY;

                for series in &self.data.series {
                    for point in &series.data {
                        if point.timestamp >= window_start {
                            min_val = min_val.min(point.value);
                            max_val = max_val.max(point.value);
                            min_time = min_time.min(point.timestamp);
                            max_time = max_time.max(point.timestamp);
                        }
                    }
                }

                if min_time == f64::INFINITY {
                    (0.0, 100.0, window_start.max(0.0), absolute_max_time)
                } else {
                    let val_range = max_val - min_val;
                    let val_padding = if val_range > 0.0 {
                        val_range * 0.1
                    } else {
                        1.0
                    };
                    min_val -= val_padding;
                    max_val += val_padding;

                    let time_range = max_time - min_time;
                    if time_range <= 0.0 {
                        (min_val, max_val, min_time, min_time + 100.0)
                    } else {
                        (min_val, max_val, min_time, max_time)
                    }
                }
            };

        // Draw grid (cached)
        let grid = self.cache.draw(renderer, size, |frame| {
            // Background
            let background = Path::rectangle(
                Point::new(plot_x, plot_y),
                Size::new(plot_width, plot_height),
            );
            frame.fill(&background, Color::from_rgba(0.1, 0.1, 0.1, 1.0));

            // Grid lines
            let grid_color = Color::from_rgba(0.4, 0.4, 0.4, 0.5);
            let grid_stroke = Stroke::default().with_width(1.0).with_color(grid_color);

            // Horizontal grid lines
            for i in 0..=5 {
                let y = plot_y + (plot_height * (i as f32 / 5.0));
                let line = Path::line(Point::new(plot_x, y), Point::new(plot_x + plot_width, y));
                frame.stroke(&line, grid_stroke);
            }

            // Vertical grid lines
            for i in 0..=5 {
                let x = plot_x + (plot_width * (i as f32 / 5.0));
                let line = Path::line(Point::new(x, plot_y), Point::new(x, plot_y + plot_height));
                frame.stroke(&line, grid_stroke);
            }

            // Border
            let border = Path::rectangle(
                Point::new(plot_x, plot_y),
                Size::new(plot_width, plot_height),
            );
            frame.stroke(
                &border,
                Stroke::default()
                    .with_width(2.0)
                    .with_color(Color::from_rgba(0.7, 0.7, 0.7, 1.0)),
            );
        });

        // Draw plot lines and labels (dynamic)
        let mut plot_frame = canvas::Frame::new(renderer, size);

        // Draw plot lines
        for series in &self.data.series {
            if series.data.len() < 2 {
                continue;
            }

            let mut points = Vec::new();
            for point in &series.data {
                if point.timestamp >= window_start {
                    let time_range = max_time - min_time;
                    let val_range = max_val - min_val;
                    if time_range > 0.0 && val_range > 0.0 {
                        let x = plot_x
                            + plot_width * ((point.timestamp - min_time) / time_range) as f32;
                        let y = plot_y
                            + plot_height * (1.0 - ((point.value - min_val) / val_range) as f32);
                        points.push(Point::new(x, y));
                    }
                }
            }

            if points.len() >= 2 {
                let path = Path::new(|builder| {
                    builder.move_to(points[0]);
                    for point in points.iter().skip(1) {
                        builder.line_to(*point);
                    }
                });

                plot_frame.stroke(
                    &path,
                    Stroke::default().with_width(2.0).with_color(series.color),
                );
            }
        }

        // Axis labels
        let label_color = Color::from_rgba(0.8, 0.8, 0.8, 1.0);
        let label_size = iced::Pixels(12.0);

        // Y-axis labels (values)
        for i in 0..=5 {
            let value = min_val + (max_val - min_val) * (1.0 - (i as f64 / 5.0));
            let y = plot_y + (plot_height * (i as f32 / 5.0));
            let mut text = Text {
                content: format!("{:.1}", value),
                position: Point::new(plot_x - 10.0, y),
                size: label_size,
                color: label_color,
                ..Text::default()
            };
            text.horizontal_alignment = alignment::Horizontal::Right;
            text.vertical_alignment = alignment::Vertical::Center;
            plot_frame.fill_text(text);
        }

        // X-axis labels (time)
        for i in 0..=5 {
            let time = min_time + (max_time - min_time) * (i as f64 / 5.0);
            let x = plot_x + (plot_width * (i as f32 / 5.0));
            let mut text = Text {
                content: format!("{:.1}s", time),
                position: Point::new(x, plot_y + plot_height + 15.0),
                size: label_size,
                color: label_color,
                ..Text::default()
            };
            text.horizontal_alignment = alignment::Horizontal::Center;
            text.vertical_alignment = alignment::Vertical::Top;
            plot_frame.fill_text(text);
        }

        // Legend
        let legend_x = plot_x + plot_width - 100.0;
        let mut legend_y = plot_y + 10.0;
        for series in &self.data.series {
            // Skip heater 3 in the legend
            if series.name == "Heater 3" {
                continue;
            }

            // Color indicator
            let indicator = Path::circle(Point::new(legend_x, legend_y), 4.0);
            plot_frame.fill(&indicator, series.color);

            // Series name
            let mut text = Text {
                content: series.name.clone(),
                position: Point::new(legend_x + 10.0, legend_y),
                size: label_size,
                color: label_color,
                ..Text::default()
            };
            text.vertical_alignment = alignment::Vertical::Center;
            plot_frame.fill_text(text);

            legend_y += 18.0;
        }

        let plot_geom = plot_frame.into_geometry();
        vec![grid, plot_geom]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        _event: canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        (canvas::event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

/// Create a live plot canvas widget
pub fn live_plot<'a, Message>(data: &'a LivePlotData) -> canvas::Canvas<LivePlotProgram, Message>
where
    Message: 'a + Clone + 'static,
{
    canvas::Canvas::new(LivePlotProgram {
        data: data.clone(),
        cache: Cache::new(),
    })
}

/// Helper to create new plot data
pub fn create_live_plot(max_points: usize, padding: f32) -> LivePlotData {
    LivePlotData::new(max_points, padding)
}

// Re-export for compatibility (may be used by external code)
#[allow(unused_imports)]
pub use LivePlotData as LivePlot;
