use ::image::ImageReader;
use iced::futures::stream;
use iced::widget::image::Handle;
use iced::widget::Stack;
use iced::widget::{container, image, text};
use iced::{Alignment, Background, Color, Element, Length, Subscription, Theme};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use crate::gui::camera_display::CameraMessage;
use crate::gui::camera_display::CameraMessageType;
use crate::gui::styles::container_style::content_container;
use crate::gui::styles::container_style::ContainerLayer;
use std::io::Cursor;

/// Messages produced by the AllSkyCamera component.
#[derive(Debug, Clone)]
pub enum AllSkyCameraMessage {
    FrameReady { handle: Handle, image_hash: u64 },
    Error(String),
    TimerTick,
    Noop,
}

/// A widget that displays images fetched from a URL, updating once per second.
#[derive(Debug, Clone, PartialEq)]
pub struct AllSkyCamera {
    pub url: String,
    frame: Option<Handle>,
    status: String,
    running: bool,
    epoch: u64,                       // bump to force iced to restart the subscription
    last_image_time: Option<Instant>, // when we last received a NEW image
    last_image_hash: Option<u64>,     // hash of the last image to detect changes
}

impl Default for AllSkyCamera {
    fn default() -> Self {
        Self {
            url: "http://example.com/allsky.jpg".to_owned(),
            frame: None,
            status: "Idle".into(),
            running: false,
            epoch: 0,
            last_image_time: None,
            last_image_hash: None,
        }
    }
}

impl AllSkyCamera {
    pub fn new(url: String) -> Self {
        Self {
            url,
            frame: None,
            status: "Idle".into(),
            running: false,
            epoch: 0,
            last_image_time: None,
            last_image_hash: None,
        }
    }

    /// Begin (or force) a connection attempt. Safe to call multiple times.
    pub fn connect(&mut self) {
        self.running = true;
        // Bump epoch so iced sees a new subscription identity and restarts it.
        self.epoch = self.epoch.wrapping_add(1);
        // Reset basic UI state
        self.status = "Connecting…".into();
        self.frame = None;
    }

    pub fn subscription_with_index(&self, index: usize) -> Subscription<CameraMessage> {
        if !self.running {
            return Subscription::none();
        }

        use iced::futures::{stream::select, StreamExt};

        let url = self.url.clone();
        let image_stream = stream::unfold(AllSkyState::Connecting { url }, |state| async move {
            let (msg, next) = state.next().await;
            Some((msg, next))
        })
        .map(move |msg| CameraMessage::UpdateCamera {
            camera_index: index,
            message: CameraMessageType::AllSky(msg),
        });

        // Timer stream that ticks every second
        let index_timer = index;
        let timer_stream = stream::unfold((), move |_| async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Some((
                CameraMessage::UpdateCamera {
                    camera_index: index_timer,
                    message: CameraMessageType::AllSky(AllSkyCameraMessage::TimerTick),
                },
                (),
            ))
        });

        // Merge both streams concurrently using select
        let combined_stream = select(image_stream, timer_stream);

        Subscription::run_with_id(("allsky_cam", index, self.epoch), combined_stream)
    }

    pub fn update(&mut self, msg: AllSkyCameraMessage) {
        match msg {
            AllSkyCameraMessage::FrameReady { handle, image_hash } => {
                // Only update the timestamp if this is a new image (hash changed)
                let is_new_image = self
                    .last_image_hash
                    .map(|old_hash| old_hash != image_hash)
                    .unwrap_or(true);

                if is_new_image {
                    self.last_image_time = Some(Instant::now());
                    self.last_image_hash = Some(image_hash);
                }

                self.frame = Some(handle);
                self.status = "Connected".into();
            }
            AllSkyCameraMessage::Error(err) => {
                self.status = format!("Error: {err}");
                self.frame = None;
            }
            AllSkyCameraMessage::TimerTick => {
                // Timer tick - this will trigger a view update to refresh the counter
            }
            AllSkyCameraMessage::Noop => {}
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, AllSkyCameraMessage> {
        match &self.frame {
            Some(handle) => {
                // Calculate seconds since last image
                let seconds_ago = self
                    .last_image_time
                    .map(|t| Instant::now().duration_since(t).as_secs())
                    .unwrap_or(0);

                let counter_text = if seconds_ago == 0 {
                    "0s".to_string()
                } else if seconds_ago < 60 {
                    format!("{seconds_ago}s")
                } else {
                    let minutes = seconds_ago / 60;
                    let secs = seconds_ago % 60;
                    format!("{minutes}m {secs}s")
                };

                // Overlay the counter in the top-right corner
                Stack::new()
                    .push(
                        iced::widget::image::viewer::Viewer::new(handle.clone())
                            .width(Length::Fill),
                    )
                    .push(
                        container(text(counter_text).size(16).color(Color::WHITE))
                            .padding(8)
                            .style(move |_theme: &Theme| iced::widget::container::Style {
                                background: Some(Background::Color(Color::from_rgba(
                                    0.0, 0.0, 0.0, 0.6,
                                ))),
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: iced::Shadow::default(),
                                text_color: Some(Color::WHITE),
                            })
                            .align_x(Alignment::End)
                            .align_y(Alignment::Start),
                    )
                    .into()
            }

            None => {
                let w = 1600;
                let h = 900;
                let blank = vec![0u8; (w * h * 4) as usize];
                let handle = iced::widget::image::Handle::from_rgba(w, h, blank);
                content_container(
                    Stack::new()
                        .push(image(handle).width(Length::Fill)) // background (16:9 blank image)
                        .push(
                            container(
                                text(&self.status)
                                    .align_x(Alignment::Center)
                                    .align_y(Alignment::Center),
                            )
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center),
                        ),
                    ContainerLayer::Layer3,
                )
                .into()
            }
        }
    }
}

/// Internal state machine for the subscription.
enum AllSkyState {
    Connecting { url: String },
    Fetching { url: String, last_fetch: Instant },
    Backoff { url: String, until: Instant },
}

impl AllSkyState {
    async fn next(self) -> (AllSkyCameraMessage, AllSkyState) {
        match self {
            AllSkyState::Connecting { url } => {
                // Try to fetch immediately
                match fetch_image(&url).await {
                    Ok((handle, image_hash)) => (
                        AllSkyCameraMessage::FrameReady { handle, image_hash },
                        AllSkyState::Fetching {
                            url,
                            last_fetch: Instant::now(),
                        },
                    ),
                    Err(e) => (
                        AllSkyCameraMessage::Error(format!("Failed to fetch: {e}")),
                        AllSkyState::Backoff {
                            url,
                            until: Instant::now() + Duration::from_secs(1),
                        },
                    ),
                }
            }

            AllSkyState::Fetching { url, last_fetch } => {
                // Wait until 1 second has passed since last fetch
                let now = Instant::now();
                let elapsed = now.duration_since(last_fetch);
                if elapsed < Duration::from_secs(1) {
                    // Wait for the remaining time
                    tokio::time::sleep(Duration::from_secs(1) - elapsed).await;
                }

                // Fetch new image
                match fetch_image(&url).await {
                    Ok((handle, image_hash)) => (
                        AllSkyCameraMessage::FrameReady { handle, image_hash },
                        AllSkyState::Fetching {
                            url,
                            last_fetch: Instant::now(),
                        },
                    ),
                    Err(e) => (
                        AllSkyCameraMessage::Error(format!("Failed to fetch: {e}")),
                        AllSkyState::Backoff {
                            url,
                            until: Instant::now() + Duration::from_secs(1),
                        },
                    ),
                }
            }

            AllSkyState::Backoff { url, until } => {
                let now = Instant::now();
                if now < until {
                    tokio::time::sleep(until - now).await;
                }
                // Try connecting again
                match fetch_image(&url).await {
                    Ok((handle, image_hash)) => (
                        AllSkyCameraMessage::FrameReady { handle, image_hash },
                        AllSkyState::Fetching {
                            url,
                            last_fetch: Instant::now(),
                        },
                    ),
                    Err(e) => (
                        AllSkyCameraMessage::Error(format!("Failed to fetch: {e}")),
                        AllSkyState::Backoff {
                            url,
                            until: Instant::now() + Duration::from_secs(1),
                        },
                    ),
                }
            }
        }
    }
}

/// Fetch an image from the given URL and convert it to an Iced Handle.
/// Returns the handle and a hash of the image data to detect changes.
async fn fetch_image(url: &str) -> Result<(Handle, u64), String> {
    // Use reqwest to fetch the image
    // Accept invalid certificates for IP addresses and self-signed certs
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    let cursor = Cursor::new(&bytes);
    let img = ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess image format: {e}"))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    // Convert to RGBA
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    let pixels = rgba_img.into_raw();

    // Compute a hash of the image data to detect changes
    // Hash the slice to only hash the actual pixel data, not Vec metadata
    let mut hasher = DefaultHasher::new();
    pixels.as_slice().hash(&mut hasher);
    let image_hash = hasher.finish();

    // Create Iced Handle from RGBA data
    let handle = Handle::from_rgba(width, height, pixels);

    Ok((handle, image_hash))
}
