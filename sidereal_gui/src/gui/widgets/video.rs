use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_app::AppSinkCallbacks;
use iced::futures::stream;
use iced::widget::image::Handle;
use iced::widget::Stack;
use iced::widget::{container, image, text};
use iced::{Alignment, Element, Length, Subscription};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver; // brings in AppSinkExt & friends

fn start_gst_rtsp(url: &str) -> Result<mpsc::UnboundedReceiver<(u32, u32, Vec<u8>)>, String> {
    gst::init().map_err(|e| format!("gst init: {e}"))?;

    let pipeline_str = format!(
        "rtspsrc location={} ! decodebin ! videoconvert ! video/x-raw,format=RGBA ! appsink name=sink sync=false",
        url
    );

    let pipeline = gst::parse::launch(&pipeline_str)
        .map_err(|e| format!("pipeline build: {e}"))?
        .downcast::<gst::Pipeline>()
        .map_err(|_| "not a pipeline".to_string())?;

    let appsink = pipeline
        .by_name("sink")
        .ok_or("no appsink")?
        .downcast::<AppSink>()
        .map_err(|_| "appsink wrong type")?;

    let (tx, rx) = mpsc::unbounded_channel::<(u32, u32, Vec<u8>)>();

    appsink.set_caps(Some(
        &gst::Caps::builder("video/x-raw")
            .field("format", &"RGBA")
            .build(),
    ));

    // Clone sender for the callback; channel closes only when *all* senders are dropped.
    let tx_frames = tx.clone();
    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = match sink.pull_sample() {
                    Ok(s) => s,
                    Err(_) => return Err(gst::FlowError::Eos),
                };

                let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;

                let caps = sample.caps().ok_or(gst::FlowError::Error)?;
                let s = caps.structure(0).ok_or(gst::FlowError::Error)?;
                let width: i32 = s.get("width").map_err(|_| gst::FlowError::Error)?;
                let height: i32 = s.get("height").map_err(|_| gst::FlowError::Error)?;

                // One contiguous RGBA frame
                let data = map.as_slice().to_vec();
                let _ = tx_frames.send((width as u32, height as u32, data));
                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    // Keep pipeline alive and monitor bus in a background thread.
    let bus = pipeline.bus().ok_or_else(|| "no bus".to_string())?;

    // This sender clone is *only* to keep the channel open while the pipeline is healthy.
    let tx_bus = tx.clone();

    std::thread::spawn(move || {
        // Hold these by value in the thread so they live long enough.
        let pipeline = pipeline;
        let bus = bus;

        // Start
        let _ = pipeline.set_state(gst::State::Playing);

        // Block on bus messages; exit on EOS/ERROR
        use gst::MessageView;
        loop {
            match bus.timed_pop(gst::ClockTime::from_seconds(1)) {
                Some(msg) => match msg.view() {
                    MessageView::Eos(..) => {
                        break;
                    }
                    MessageView::Error(err) => {
                        eprintln!("GStreamer error: {}", err.error());
                        break;
                    }
                    _ => {}
                },
                None => {
                    // timeout: just continue polling
                }
            }
        }

        // Tear down: stop pipeline and drop the last sender clones.
        let _ = pipeline.set_state(gst::State::Null);
        drop(tx_bus); // after this, if the appsink callback is no longer running, rx will see None
                      // The appsink callback's tx clone will be dropped when pipeline goes Null and is dropped.
    });

    Ok(rx)
}

use std::time::{Duration, Instant};

use crate::gui::styles::container_style::content_container;
use crate::gui::styles::container_style::ContainerLayer;
/// Messages produced by the IpCamera component.
#[derive(Debug, Clone)]
pub enum IpCameraMessage {
    FrameReady(u32, u32, Vec<u8>), // width, height, RGBA pixels
    Disconnected(String),
    Connected,
    Stats { fps: f32 },
    Noop,
}
/// A simple, reusable Iced component that displays an IP camera MJPEG feed.
pub struct IpCamera {
    url: String,
    frame: Option<image::Handle>,
    status: String,
    last_frame_at: Option<Instant>,
    fps: f32,
}
impl Default for IpCamera {
    fn default() -> Self {
        Self {
            url: "rtsp://192.168.1.171:8554/city-traffic".to_owned(),

            frame: None,
            status: "Idle".into(),
            last_frame_at: None,
            fps: 0.0,
        }
    }
}

impl IpCamera {
    /// `auth`: optional (username, password). If `None`, URL may already contain auth or be public.
    pub fn new(url: String, auth: Option<(String, String)>) -> Self {
        Self {
            url,
            frame: None,
            status: "Connecting…".into(),
            last_frame_at: None,
            fps: 0.0,
        }
    }

    pub fn update(&mut self, msg: IpCameraMessage) {
        match msg {
            IpCameraMessage::Connected => {
                self.status = "Connected".into();
            }
            IpCameraMessage::FrameReady(width, height, rgba) => {
                // Directly use raw pixels
                let handle = Handle::from_rgba(width, height, rgba);

                // FPS calc
                let now = Instant::now();
                if let Some(prev) = self.last_frame_at.replace(now) {
                    let dt = now.saturating_duration_since(prev).as_secs_f32();
                    if dt > 0.0 {
                        // low-pass filter to smooth FPS readout
                        let inst = 1.0 / dt;
                        self.fps = self.fps * 0.85 + inst * 0.15;
                    }
                } else {
                    self.fps = 0.0;
                }
                self.frame = Some(handle);
                self.status = "Streaming".into();
            }
            IpCameraMessage::Disconnected(err) => {
                self.status = format!("Disconnected: {err}");
                self.frame = None;
                self.fps = 0.0;
                self.last_frame_at = None;
            }
            IpCameraMessage::Stats { fps } => {
                self.fps = fps;
            }
            IpCameraMessage::Noop => {}
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, IpCameraMessage> {
        match &self.frame {
            Some(handle) => image::viewer::Viewer::new(handle.clone())
                .width(Length::Fill)
                .into(),

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

    pub fn subscription(&self) -> Subscription<IpCameraMessage> {
        let url = self.url.clone();

        // Build a stream of IpCameraMessage values
        let cam_stream = stream::unfold(State::Connecting { url }, |state| async move {
            let (msg, next) = state.next().await;
            Some((msg, next)) // unfold expects Option<(Item, State)>
        });

        // Give it an identity so iced can cache/restart correctly
        let id = self.url.clone(); // Hash + 'static

        Subscription::run_with_id(id, cam_stream)
    }
}

/// Internal state machine for the subscription.
enum State {
    Connecting {
        url: String,
    },
    Streaming {
        url: String,
        frames: UnboundedReceiver<(u32, u32, Vec<u8>)>,
        last_fps_emit: Instant,
        frame_count: u32,
    },
    Backoff {
        url: String,
        until: Instant,
        attempt: u32,
    },
}

impl State {
    async fn next(self) -> (IpCameraMessage, State) {
        match self {
            State::Connecting { url } => match start_gst_rtsp(&url) {
                Ok(mut rx) => {
                    if let Some((w, h, rgba)) = rx.recv().await {
                        (
                            IpCameraMessage::FrameReady(w, h, rgba),
                            State::Streaming {
                                url, // <—
                                frames: rx,
                                last_fps_emit: Instant::now(),
                                frame_count: 1,
                            },
                        )
                    } else {
                        (
                            IpCameraMessage::Disconnected("no frames".into()),
                            State::Backoff {
                                url, // <—
                                until: Instant::now() + Duration::from_millis(800),
                                attempt: 1,
                            },
                        )
                    }
                }
                Err(e) => (
                    IpCameraMessage::Disconnected(format!("connect error: {e}")),
                    State::Backoff {
                        url, // <—
                        until: Instant::now() + Duration::from_millis(800),
                        attempt: 1,
                    },
                ),
            },

            State::Streaming {
                mut frames,
                mut last_fps_emit,
                mut frame_count,
                url,
            } => {
                match frames.recv().await {
                    Some((w, h, rgba)) => {
                        frame_count += 1;
                        if last_fps_emit.elapsed() >= Duration::from_secs(1) {
                            let fps = frame_count as f32
                                / last_fps_emit.elapsed().as_secs_f32().max(1e-6);
                            last_fps_emit = Instant::now();
                            frame_count = 0;
                            (
                                IpCameraMessage::Stats { fps },
                                State::Streaming {
                                    url,
                                    frames,
                                    last_fps_emit,
                                    frame_count,
                                },
                            )
                        } else {
                            (
                                IpCameraMessage::FrameReady(w, h, rgba),
                                State::Streaming {
                                    url,
                                    frames,
                                    last_fps_emit,
                                    frame_count,
                                },
                            )
                        }
                    }
                    None => {
                        // Sender was dropped (EOS/ERROR) — emit Disconnected *and* backoff with the same URL
                        let attempt = 1;
                        let delay = Duration::from_millis(500 * (1u64 << (attempt.min(6)))); // 500ms,1s,2s,4s,8s,16s (cap)
                        (
                            IpCameraMessage::Disconnected("stream ended".into()),
                            State::Backoff {
                                url,
                                until: Instant::now() + delay,
                                attempt,
                            },
                        )
                    }
                }
            }

            State::Backoff {
                url,
                until,
                attempt,
            } => {
                if Instant::now() >= until {
                    (IpCameraMessage::Noop, State::Connecting { url })
                } else {
                    (
                        IpCameraMessage::Noop,
                        State::Backoff {
                            url,
                            until,
                            attempt,
                        },
                    )
                }
            }
        }
    }
}
