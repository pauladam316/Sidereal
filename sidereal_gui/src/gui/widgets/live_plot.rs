use iced::mouse;
use iced::widget::canvas as canvas_widget;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text};
use iced::{alignment, Color, Point, Rectangle, Renderer, Size, Theme};
use std::collections::VecDeque;

/// A data point in the plot
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub timestamp: f64, // Time in seconds (relative to start or absolute)
    pub value: f64,
}

/// Configuration for a plot line/series
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

    /// Add a new data point and maintain the sliding window
    pub fn add_point(&mut self, point: DataPoint, max_points: usize) {
        self.data.push_back(point);
        while self.data.len() > max_points {
            self.data.pop_front();
        }
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

/// A live updating plot widget that displays telemetry data
/// Supports multiple plot lines and maintains a sliding window of the past N data points
pub struct LivePlot {
    series: Vec<PlotSeries>,
    max_points: usize,
    padding: f32, // Padding around the plot area
    grid_cache: Cache,
}

// Manual Clone implementation because Cache doesn't implement Clone
impl Clone for LivePlot {
    fn clone(&self) -> Self {
        Self {
            series: self.series.clone(),
            max_points: self.max_points,
            padding: self.padding,
            grid_cache: Cache::new(), // Create new cache on clone
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct LivePlotState;

impl LivePlot {
    /// Create a new live plot widget
    ///
    /// # Arguments
    /// * `max_points` - Maximum number of data points to keep in the sliding window
    /// * `padding` - Padding around the plot area in pixels
    pub fn new(max_points: usize, padding: f32) -> Self {
        Self {
            series: Vec::new(),
            max_points,
            padding,
            grid_cache: Cache::new(),
        }
    }

    /// Add a new plot series
    pub fn add_series(&mut self, name: impl Into<String>, color: Color) -> usize {
        let id = self.series.len();
        self.series.push(PlotSeries::new(name, color));
        id
    }

    /// Get a mutable reference to a series by index
    pub fn series_mut(&mut self, index: usize) -> Option<&mut PlotSeries> {
        self.series.get_mut(index)
    }

    /// Get a reference to a series by index
    pub fn series(&self, index: usize) -> Option<&PlotSeries> {
        self.series.get(index)
    }

    /// Add a data point to a series
    pub fn add_data_point(&mut self, series_index: usize, point: DataPoint) {
        if let Some(series) = self.series.get_mut(series_index) {
            series.add_point(point, self.max_points);
            // Clear grid cache when data changes
            self.grid_cache.clear();
        }
    }

    /// Clear all data from all series
    pub fn clear_all(&mut self) {
        for series in &mut self.series {
            series.clear();
        }
        self.grid_cache.clear();
    }

    /// Clear the grid cache (call when you want to force a redraw of the grid)
    pub fn clear_cache(&mut self) {
        self.grid_cache.clear();
    }
}

impl<Message> canvas_widget::Program<Message> for LivePlot {
    type State = LivePlotState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let size = Size::new(bounds.width, bounds.height);
        let plot_width = bounds.width - self.padding * 2.0;
        let plot_height = bounds.height - self.padding * 2.0;
        let plot_x = bounds.x + self.padding;
        let plot_y = bounds.y + self.padding;

        // Calculate data bounds (min/max values across all series)
        let (min_val, max_val, min_time, max_time) =
            if self.series.is_empty() || self.series.iter().all(|s| s.data.is_empty()) {
                // Default bounds if no data
                (0.0, 100.0, 0.0, 100.0)
            } else {
                let mut min_val = f64::INFINITY;
                let mut max_val = f64::NEG_INFINITY;
                let mut min_time = f64::INFINITY;
                let mut max_time = f64::NEG_INFINITY;

                for series in &self.series {
                    for point in &series.data {
                        min_val = min_val.min(point.value);
                        max_val = max_val.max(point.value);
                        min_time = min_time.min(point.timestamp);
                        max_time = max_time.max(point.timestamp);
                    }
                }

                // Add some padding to the value range
                let val_range = max_val - min_val;
                let val_padding = if val_range > 0.0 {
                    val_range * 0.1
                } else {
                    1.0
                };
                min_val -= val_padding;
                max_val += val_padding;

                // Handle time range
                let time_range = max_time - min_time;
                if time_range <= 0.0 {
                    (min_val, max_val, min_time, min_time + 100.0)
                } else {
                    (min_val, max_val, min_time, max_time)
                }
            };

        // Draw grid (cached)
        let grid = self.grid_cache.draw(renderer, size, |frame| {
            // Draw grid lines
            let grid_color = Color::from_rgba(0.4, 0.4, 0.4, 0.5);
            let grid_stroke = Stroke::default().with_width(1.0).with_color(grid_color);

            // Horizontal grid lines (value axis)
            for i in 0..=5 {
                let y = plot_y + (plot_height * (i as f32 / 5.0));
                let line = Path::line(Point::new(plot_x, y), Point::new(plot_x + plot_width, y));
                frame.stroke(&line, grid_stroke);
            }

            // Vertical grid lines (time axis)
            for i in 0..=5 {
                let x = plot_x + (plot_width * (i as f32 / 5.0));
                let line = Path::line(Point::new(x, plot_y), Point::new(x, plot_y + plot_height));
                frame.stroke(&line, grid_stroke);
            }

            // Draw border
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

        // Draw plot lines (dynamic)
        let mut plot_frame = Frame::new(renderer, size);

        for series in &self.series {
            if series.data.len() < 2 {
                continue;
            }

            // Build path for this series
            let mut points = Vec::new();
            for point in &series.data {
                let x = plot_x
                    + plot_width * ((point.timestamp - min_time) / (max_time - min_time)) as f32;
                let y = plot_y
                    + plot_height * (1.0 - ((point.value - min_val) / (max_val - min_val)) as f32);
                points.push(Point::new(x, y));
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

        // Draw axis labels
        let label_color = Color::from_rgba(0.8, 0.8, 0.8, 1.0);
        let label_size = iced::Pixels(12.0);

        // Y-axis labels (values)
        for i in 0..=5 {
            let value = min_val + (max_val - min_val) * (1.0 - (i as f64 / 5.0));
            let y = plot_y + (plot_height * (i as f32 / 5.0));
            let mut text = Text {
                content: format!("{:.1}", value),
                position: Point::new(plot_x - 5.0, y),
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

        // Draw legend
        let legend_x = plot_x + plot_width - 100.0;
        let mut legend_y = plot_y + 10.0;
        for series in &self.series {
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

impl LivePlot {
    /// Convert this LivePlot into a canvas widget
    pub fn into_widget<Message>(self) -> canvas_widget::Canvas<LivePlot, Message>
    where
        Message: Clone + 'static,
    {
        canvas_widget::Canvas::new(self)
    }
}

/// Helper function to create a new live plot widget with default settings
///
/// # Example
/// ```rust,ignore
/// let mut plot = create_live_plot(1000, 20.0); // 1000 points max, 20px padding
/// let temp_series = plot.add_series("Temperature", Color::from_rgb(1.0, 0.0, 0.0));
/// let pressure_series = plot.add_series("Pressure", Color::from_rgb(0.0, 1.0, 0.0));
///
/// // Add data points
/// plot.add_data_point(temp_series, DataPoint { timestamp: 0.0, value: 25.5 });
/// plot.add_data_point(pressure_series, DataPoint { timestamp: 0.0, value: 1013.25 });
///
/// // Use in UI
/// plot.into_widget()
/// ```
pub fn create_live_plot(max_points: usize, padding: f32) -> LivePlot {
    LivePlot::new(max_points, padding)
}
