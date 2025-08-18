use futures_timer::Delay;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_app::AppSinkCallbacks;
use gstreamer_video as gst_video;
use gstreamer_video::VideoFrameExt;
use iced::futures::stream;
use iced::widget::image::Handle;
use iced::widget::Stack;
use iced::widget::{container, image, text};
use iced::{Alignment, Element, Length, Subscription};
use tokio::sync::mpsc;

use std::time::{Duration, Instant};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::gui::camera_display::CameraMessage;
use crate::gui::camera_display::CameraMessageType;
use crate::gui::styles::container_style::content_container;
use crate::gui::styles::container_style::ContainerLayer;

/// When dropped, requests the RTSP pipeline thread to stop.
#[derive(Debug, Clone)]
struct StopHandle(Arc<AtomicBool>);
impl StopHandle {
    fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    fn should_stop(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
    fn request_stop(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
impl Drop for StopHandle {
    fn drop(&mut self) {
        self.request_stop();
    }
}

fn start_gst_rtsp(url: &str) -> Result<(mpsc::Receiver<(u32, u32, Vec<u8>)>, StopHandle), String> {
    let pipeline_str = format!(
        "rtspsrc location={} protocols=tcp latency=200 do-rtsp-keep-alive=true ! \
         decodebin ! \
         videoconvert ! videoscale ! \
         video/x-raw,format=RGBA ! \
         queue leaky=downstream max-size-buffers=1 ! \
         appsink name=sink sync=false max-buffers=1 drop=true",
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

    let (tx, rx) = tokio::sync::mpsc::channel::<(u32, u32, Vec<u8>)>(2);

    appsink.set_caps(Some(
        &gst::Caps::builder("video/x-raw")
            .field("format", &"RGBA")
            .field("memory", &"SystemMemory")
            .build(),
    ));
    // Clone sender for the callback; channel closes only when *all* senders are dropped.
    let tx_frames = tx.clone();
    let _ = appsink.set_property("max-buffers", 1u32);
    let _ = appsink.set_property("drop", true);

    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                // Never let a panic cross the FFI boundary.
                let res: Result<gst::FlowSuccess, gst::FlowError> =
                    std::panic::catch_unwind(|| {
                        let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                        let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                        let caps = sample.caps().ok_or(gst::FlowError::Error)?;
                        let info = gst_video::VideoInfo::from_caps(caps)
                            .map_err(|_| gst::FlowError::Error)?;

                        if info.format() != gst_video::VideoFormat::Rgba {
                            eprintln!("[appsink] unexpected format: {:?}", info.format());
                            return Err(gst::FlowError::Error);
                        }

                        let frame =
                            gst_video::VideoFrameRef::from_buffer_ref_readable(buffer, &info)
                                .map_err(|_| gst::FlowError::Error)?;

                        let w = info.width() as usize;
                        let h = info.height() as usize;
                        let stride = frame.plane_stride()[0] as usize;
                        let src = frame.plane_data(0).map_err(|_| gst::FlowError::Error)?;

                        // Tightly packed RGBA buffer with checks
                        let mut data = vec![0u8; w.saturating_mul(h).saturating_mul(4)];
                        for y in 0..h {
                            let start = y.checked_mul(stride).ok_or(gst::FlowError::Error)?;
                            let end = start.checked_add(w * 4).ok_or(gst::FlowError::Error)?;
                            let row = src.get(start..end).ok_or(gst::FlowError::Error)?;

                            let dst_start = y * w * 4;
                            let dst_end = dst_start + w * 4;
                            let dst = data
                                .get_mut(dst_start..dst_end)
                                .ok_or(gst::FlowError::Error)?;
                            dst.copy_from_slice(row);
                        }

                        // Try to send; drop silently if full or closed
                        let _ = tx_frames.try_send((w as u32, h as u32, data));
                        Ok(gst::FlowSuccess::Ok)
                    })
                    .unwrap_or_else(|_| {
                        eprintln!("[appsink] new_sample panicked; converting to FlowError");
                        Err(gst::FlowError::Error)
                    });

                res
            })
            .build(),
    );

    // Keep pipeline alive and monitor bus in a background thread.
    let bus = pipeline.bus().ok_or_else(|| "no bus".to_string())?;

    // This sender clone is *only* to keep the channel open while the pipeline is healthy.
    let tx_bus = tx.clone();

    let stop = StopHandle::new();
    let stop_for_thread = stop.clone();

    std::thread::spawn(move || {
        // Hold these by value in the thread so they live long enough.
        let pipeline = pipeline;
        let bus = bus;

        // Start
        let _ = pipeline.set_state(gst::State::Playing);

        // Block on bus messages; exit on EOS/ERROR or on stop request
        use gst::MessageView;
        loop {
            if stop_for_thread.should_stop() {
                break;
            }
            match bus.timed_pop(gst::ClockTime::from_mseconds(250)) {
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
        let (_current, _pending, _ret) = pipeline.state(gst::ClockTime::from_seconds(5));
        drop(tx_bus); // after this, if the appsink callback is no longer running, rx will see None
                      // The appsink callback's tx clone will be dropped when pipeline goes Null and is dropped.
    });

    Ok((rx, stop))
}

/// Messages produced by the IpCamera component.
#[derive(Debug, Clone)]
pub enum IpCameraMessage {
    FrameReady(u32, u32, Vec<u8>), // width, height, RGBA pixels
    Disconnected(String),
    Connected,
    Noop,
}

/// A simple, reusable Iced component that displays an IP camera RTSP feed.
#[derive(Debug, Clone, PartialEq)]
pub struct IpCamera {
    pub url: String,
    frame: Option<image::Handle>,
    status: String,
    last_frame_at: Option<Instant>,
    running: bool, // start idle; subscription() is none unless true
    epoch: u64,    // bump to force iced to restart the subscription
}

impl Default for IpCamera {
    fn default() -> Self {
        Self {
            url: "rtsp://192.168.1.171:8554/city-traffic".to_owned(),
            frame: None,
            status: "Idle".into(),
            last_frame_at: None,
            running: false,
            epoch: 0,
        }
    }
}

impl IpCamera {
    /// `auth`: optional (username, password). If `None`, URL may already contain auth or be public.
    pub fn new(url: String, _auth: Option<(String, String)>) -> Self {
        Self {
            url,
            frame: None,
            status: "Idle".into(),
            last_frame_at: None,
            running: false,
            epoch: 0,
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
        self.last_frame_at = None;
    }

    pub fn subscription_with_index(&self, index: usize) -> Subscription<CameraMessage> {
        if !self.running {
            return Subscription::none();
        }

        use iced::futures::StreamExt; // for .map on the STREAM we build

        let url = self.url.clone();
        let stream = stream::unfold(State::Connecting { url }, |state| async move {
            let (msg, next) = state.next().await;
            Some((msg, next))
        })
        // <-- mapping at the STREAM layer is OK (captures allowed)
        .map(move |ip| CameraMessage::UpdateCamera {
            camera_index: index,
            message: CameraMessageType::IpCamera(ip),
        });

        Subscription::run_with_id(("ip_cam_v2", index, self.epoch), stream)
    }

    pub fn update(&mut self, msg: IpCameraMessage) {
        match msg {
            IpCameraMessage::Connected => {
                self.status = "Connected".into();
            }
            IpCameraMessage::FrameReady(width, height, rgba) => {
                // 2a) sanity check length
                let expected = (width as usize)
                    .saturating_mul(height as usize)
                    .saturating_mul(4);
                if rgba.len() != expected || width == 0 || height == 0 {
                    eprintln!(
                        "[ui] dropping malformed frame: got {}, expected {} ({}x{})",
                        rgba.len(),
                        expected,
                        width,
                        height
                    );
                    return; // drop
                }

                // 2b) guard against downstream panics (alignment etc.)
                let handle =
                    match std::panic::catch_unwind(|| Handle::from_rgba(width, height, rgba)) {
                        Ok(h) => h,
                        Err(_) => {
                            eprintln!("[ui] Handle::from_rgba panicked; dropping frame");
                            return; // drop
                        }
                    };

                self.frame = Some(handle);
                self.status = "Streaming".into();
            }
            IpCameraMessage::Disconnected(err) => {
                self.status = format!("Disconnected: {err}");
                self.frame = None;
                self.last_frame_at = None;
            }
            IpCameraMessage::Noop => {}
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, IpCameraMessage> {
        match &self.frame {
            Some(handle) => iced::widget::image::viewer::Viewer::new(handle.clone())
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
        if !self.running {
            return Subscription::none();
        }

        let url = self.url.clone();

        // Build a stream of IpCameraMessage values
        let cam_stream = stream::unfold(State::Connecting { url }, |state| async move {
            let (msg, next) = state.next().await;
            Some((msg, next)) // unfold expects Option<(Item, State)>
        });

        // Identity includes epoch so `connect()` forces a clean restart.
        let id = ("ip_cam_v2", self.url.clone(), self.epoch);

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
        frames: mpsc::Receiver<(u32, u32, Vec<u8>)>,
        stop: StopHandle,
    },
    Backoff {
        url: String,
        until: Instant,
    },
}

impl State {
    async fn next(self) -> (IpCameraMessage, State) {
        match self {
            State::Connecting { url } => match start_gst_rtsp(&url) {
                Ok((mut rx, stop)) => {
                    if let Some((w, h, rgba)) = rx.recv().await {
                        (
                            IpCameraMessage::FrameReady(w, h, rgba),
                            State::Streaming {
                                url, // <—
                                frames: rx,
                                stop,
                            },
                        )
                    } else {
                        (
                            IpCameraMessage::Disconnected("no frames".into()),
                            State::Backoff {
                                url, // <—
                                until: Instant::now() + Duration::from_millis(800),
                            },
                        )
                    }
                }
                Err(e) => (
                    IpCameraMessage::Disconnected(format!("connect error: {e}")),
                    State::Backoff {
                        url, // <—
                        until: Instant::now() + Duration::from_millis(800),
                    },
                ),
            },

            State::Streaming {
                mut frames,
                url,
                stop,
            } => match frames.recv().await {
                Some((w, h, rgba)) => (
                    IpCameraMessage::FrameReady(w, h, rgba),
                    State::Streaming { url, frames, stop },
                ),
                None => {
                    // Sender was dropped (EOS/ERROR) — emit Disconnected *and* backoff with the same URL
                    // The StopHandle will be dropped as we leave this state; that requests shutdown.
                    let attempt = 1;
                    let delay = Duration::from_millis(500 * (1u64 << (attempt.min(6)))); // 500ms..16s
                    (
                        IpCameraMessage::Disconnected("stream ended".into()),
                        State::Backoff {
                            url,
                            until: Instant::now() + delay,
                        },
                    )
                }
            },

            State::Backoff { url, until } => {
                let now = Instant::now();
                if now < until {
                    Delay::new(until - now).await;
                }
                (IpCameraMessage::Noop, State::Connecting { url })
            }
        }
    }
}
