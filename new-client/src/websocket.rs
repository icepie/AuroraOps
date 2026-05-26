use fastwebsockets::{FragmentCollectorRead, Frame, OpCode, WebSocket, WebSocketError};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{mpsc, Arc};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::channel;
use tracing::{debug, error, trace, warn};

use crate::capturable::{get_capturables, Capturable, Recorder};
use crate::input::device::{InputDevice, InputDeviceType};
use crate::protocol::{
    ClientConfiguration, KeyboardEvent, MessageInbound, MessageOutbound, PointerEvent,
    RuntimeStatus, WeylusReceiver, WeylusSender, WheelEvent,
};

use crate::cerror::CErrorCode;
use crate::video::{EncoderOptions, VideoEncoder};

struct VideoConfig {
    capturable: Box<dyn Capturable>,
    capture_cursor: bool,
    max_width: usize,
    max_height: usize,
    frame_rate: f64,
}

enum VideoCommands {
    Start(VideoConfig),
    Pause,
    Resume,
    Restart,
}

fn send_message<S>(sender: &mut S, message: MessageOutbound)
where
    S: WeylusSender,
{
    if let Err(err) = sender.send_message(message) {
        warn!("Failed to send message to client: {err}");
    }
}

fn send_runtime_status<S>(sender: &mut S, capture_backend: &str, encoder_backend: &str)
where
    S: WeylusSender,
{
    send_message(
        sender,
        MessageOutbound::RuntimeStatus(RuntimeStatus {
            capture_backend: Some(capture_backend.to_string()),
            encoder_backend: Some(encoder_backend.to_string()),
            input_backend: None,
            pointer_backend: None,
            keyboard_backend: None,
        }),
    );
}

fn send_input_status<S>(sender: &mut S, input_backend: &str)
where
    S: WeylusSender,
{
    send_message(
        sender,
        MessageOutbound::RuntimeStatus(RuntimeStatus {
            capture_backend: None,
            encoder_backend: None,
            input_backend: Some(input_backend.to_string()),
            pointer_backend: None,
            keyboard_backend: None,
        }),
    );
}

fn send_pointer_status<S>(sender: &mut S, pointer_backend: impl Into<String>)
where
    S: WeylusSender,
{
    send_message(
        sender,
        MessageOutbound::RuntimeStatus(RuntimeStatus {
            capture_backend: None,
            encoder_backend: None,
            input_backend: None,
            pointer_backend: Some(pointer_backend.into()),
            keyboard_backend: None,
        }),
    );
}

#[cfg(target_os = "linux")]
fn has_x_display() -> bool {
    std::env::var_os("DISPLAY").is_some()
}

#[cfg(all(target_os = "linux", feature = "pipewire"))]
fn try_wayland_portal_input_device(
    capturable: Box<dyn Capturable>,
) -> Result<Box<dyn InputDevice>, String> {
    crate::input::wayland_portal_device::WaylandPortalDevice::new(capturable)
        .map(|device| Box::new(device) as Box<dyn InputDevice>)
}

pub struct WeylusClientHandler<S, R, FnUInput> {
    sender: S,
    receiver: Option<R>,
    video_sender: mpsc::Sender<VideoCommands>,
    input_device: Option<Box<dyn InputDevice>>,
    capturables: Vec<Box<dyn Capturable>>,
    on_uinput_inaccessible: FnUInput,
    config: WeylusClientConfig,
    #[cfg(target_os = "linux")]
    capture_cursor: bool,
    client_name: Option<String>,
    video_thread: JoinHandle<()>,
}

#[derive(Clone)]
pub struct WeylusClientConfig {
    pub encoder_options: EncoderOptions,
    #[cfg(target_os = "linux")]
    pub wayland_support: bool,
    #[cfg(target_os = "linux")]
    pub kms_support: bool,
    #[cfg(target_os = "linux")]
    pub kms_device: Option<String>,
    pub no_gui: bool,
}

impl<S, R, FnUInput> WeylusClientHandler<S, R, FnUInput> {
    pub fn new(
        sender: S,
        receiver: R,
        on_uinput_inaccessible: FnUInput,
        config: WeylusClientConfig,
    ) -> Self
    where
        R: WeylusReceiver,
        S: WeylusSender + Clone + Send + Sync + 'static,
    {
        let (video_sender, video_receiver) = mpsc::channel::<VideoCommands>();
        let video_thread = {
            let sender = sender.clone();
            // offload creating the videostream to another thread to avoid blocking the thread that
            // is receiving messages from the websocket
            spawn(move || handle_video(video_receiver, sender, config.encoder_options))
        };

        Self {
            sender,
            receiver: Some(receiver),
            video_sender,
            input_device: None,
            capturables: vec![],
            on_uinput_inaccessible,
            config,
            #[cfg(target_os = "linux")]
            capture_cursor: false,
            client_name: None,
            video_thread,
        }
    }

    pub fn run(mut self)
    where
        R: WeylusReceiver,
        S: WeylusSender + Clone + Send + Sync + 'static,
        FnUInput: Fn(),
    {
        for message in self.receiver.take().unwrap() {
            match message {
                Ok(message) => {
                    trace!("Received message: {message:?}");
                    match message {
                        MessageInbound::PointerEvent(event) => self.process_pointer_event(&event),
                        MessageInbound::WheelEvent(event) => self.process_wheel_event(&event),
                        MessageInbound::KeyboardEvent(event) => self.process_keyboard_event(&event),
                        MessageInbound::GetCapturableList => self.send_capturable_list(),
                        MessageInbound::Config(config) => self.update_config(config),
                        MessageInbound::PauseVideo => {
                            self.video_sender.send(VideoCommands::Pause).unwrap()
                        }
                        MessageInbound::ResumeVideo => {
                            self.video_sender.send(VideoCommands::Resume).unwrap()
                        }
                        MessageInbound::RestartVideo => {
                            self.video_sender.send(VideoCommands::Restart).unwrap()
                        }
                        MessageInbound::ChooseCustomInputAreas => {
                            let (sender, receiver) = std::sync::mpsc::channel();
                            crate::gui::get_input_area(self.config.no_gui, sender);
                            let mut sender = self.sender.clone();
                            spawn(move || {
                                while let Ok(areas) = receiver.recv() {
                                    send_message(
                                        &mut sender,
                                        MessageOutbound::CustomInputAreas(areas),
                                    );
                                }
                            });
                        }
                    }
                }
                Err(err) => {
                    warn!("Failed to read message {err}!");
                    self.send_message(MessageOutbound::Error(
                        "Failed to read message!".to_string(),
                    ));
                }
            }
        }

        drop(self.video_sender);
        if let Err(err) = self.video_thread.join() {
            warn!("Failed to join video thread: {err:?}");
        }
    }

    fn send_message(&mut self, message: MessageOutbound)
    where
        S: WeylusSender,
    {
        send_message(&mut self.sender, message)
    }

    fn process_wheel_event(&mut self, event: &WheelEvent) {
        match &mut self.input_device {
            Some(i) => i.send_wheel_event(event),
            None => warn!("Input device is not initalized, can not process WheelEvent!"),
        }
    }

    fn process_pointer_event(&mut self, event: &PointerEvent)
    where
        S: WeylusSender,
    {
        if self.input_device.is_some() {
            self.input_device
                .as_mut()
                .unwrap()
                .send_pointer_event(event);
            if !matches!(event.event_type, crate::protocol::PointerEventType::MOVE) {
                send_pointer_status(
                    &mut self.sender,
                    format!(
                        "agent recv {:?} type={:?} button={:?} buttons={:?} x={:.4} y={:.4}",
                        event.event_type,
                        event.pointer_type,
                        event.button,
                        event.buttons,
                        event.x,
                        event.y
                    ),
                );
            }
        } else {
            warn!("Input device is not initalized, can not process PointerEvent!");
        }
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent)
    where
        S: WeylusSender,
    {
        if self.input_device.is_some() {
            let keyboard_backend = format!(
                "agent recv {} code={} key={}",
                match event.event_type {
                    crate::protocol::KeyboardEventType::DOWN => "down",
                    crate::protocol::KeyboardEventType::UP => "up",
                    crate::protocol::KeyboardEventType::REPEAT => "repeat",
                },
                event.code,
                event.key
            );
            let mut statuses = self.input_device.as_mut().unwrap().drain_keyboard_status();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.input_device
                    .as_mut()
                    .unwrap()
                    .send_keyboard_event(event)
            }));
            if result.is_err() {
                warn!(
                    "Keyboard input handler panicked; event skipped: code={} key={}",
                    event.code, event.key
                );
            }
            statuses.insert(0, keyboard_backend);
            statuses.append(&mut self.input_device.as_mut().unwrap().drain_keyboard_status());
            for status in statuses {
                self.send_message(MessageOutbound::RuntimeStatus(RuntimeStatus {
                    capture_backend: None,
                    encoder_backend: None,
                    input_backend: None,
                    pointer_backend: None,
                    keyboard_backend: Some(status),
                }));
            }
        } else {
            warn!("Input device is not initalized, can not process KeyboardEvent!");
        }
    }

    fn send_capturable_list(&mut self)
    where
        S: WeylusSender,
    {
        let mut windows = Vec::<String>::new();
        self.capturables = get_capturables(
            #[cfg(target_os = "linux")]
            self.config.wayland_support,
            #[cfg(target_os = "linux")]
            self.capture_cursor,
            #[cfg(target_os = "linux")]
            self.config.kms_support,
            #[cfg(target_os = "linux")]
            self.config.kms_device.as_deref(),
        );
        self.capturables.iter().for_each(|c| {
            windows.push(c.name());
        });
        if windows.is_empty() {
            warn!(
                "No capturables found. DISPLAY={:?}",
                std::env::var("DISPLAY")
            );
        }
        self.send_message(MessageOutbound::CapturableList(windows));
    }

    fn update_config(&mut self, config: ClientConfiguration)
    where
        S: WeylusSender,
        FnUInput: Fn(),
    {
        let client_name_changed = if self.client_name != config.client_name {
            self.client_name = config.client_name;
            true
        } else {
            false
        };
        let mut capturable_id = config.capturable_id;
        if capturable_id >= self.capturables.len() {
            warn!(
                "Got invalid id for capturable: {}, current list has {} entries; refreshing list and falling back to 0.",
                capturable_id,
                self.capturables.len()
            );
            self.send_capturable_list();
            if self.capturables.is_empty() {
                self.send_message(MessageOutbound::ConfigError(format!(
                    "No capturable display is available. DISPLAY={:?}",
                    std::env::var("DISPLAY")
                )));
                return;
            }
            capturable_id = 0;
        }

        if capturable_id < self.capturables.len() {
            let capturable = self.capturables[capturable_id].clone();
            debug!(
                "Selected capturable[{}]: {}",
                capturable_id,
                capturable.name()
            );

            #[cfg(target_os = "linux")]
            {
                self.capture_cursor = config.capture_cursor;
            }

            #[cfg(target_os = "linux")]
            {
                let mut portal_selected = false;
                #[cfg(feature = "pipewire")]
                if crate::input::wayland_portal_device::WaylandPortalDevice::supports_capturable(
                    capturable.as_ref(),
                ) {
                    if self.input_device.as_ref().map_or(true, |d| {
                        client_name_changed
                            || d.device_type() != InputDeviceType::WaylandPortalDevice
                    }) {
                        match try_wayland_portal_input_device(capturable.clone()) {
                            Ok(device) => {
                                debug!("Using Wayland portal RemoteDesktop device for input");
                                self.input_device = Some(device);
                                portal_selected = true;
                            }
                            Err(err) => {
                                warn!(
                                    "Failed to create Wayland portal input device, falling back to legacy backends: {}",
                                    err
                                );
                            }
                        }
                    } else if let Some(d) = self.input_device.as_mut() {
                        d.set_capturable(capturable.clone());
                        portal_selected = true;
                    }
                }

                if !portal_selected && config.uinput_support {
                    if self.input_device.as_ref().map_or(true, |d| {
                        client_name_changed || d.device_type() != InputDeviceType::UInputDevice
                    }) {
                        let device = crate::input::uinput_device::UInputDevice::new(
                            capturable.clone(),
                            &self.client_name,
                        );
                        match device {
                            Ok(d) => self.input_device = Some(Box::new(d)),
                            Err(e) => {
                                error!("Failed to create uinput device: {}", e);
                                if let CErrorCode::UInputNotAccessible = e.to_enum() {
                                    (self.on_uinput_inaccessible)();
                                }
                                // Try to fall back to XTest
                                debug!("Attempting to use XTest as fallback");
                                match crate::input::xtest_device::XTestDevice::new(
                                    capturable.clone(),
                                ) {
                                    Ok(xtest_device) => {
                                        debug!("Successfully created XTest device as fallback");
                                        self.input_device = Some(Box::new(xtest_device));
                                    }
                                    Err(xtest_err) => {
                                        error!("Failed to create XTest device: {}", xtest_err);
                                        self.input_device = None;
                                        self.send_message(MessageOutbound::Error(format!(
                                            "Input disabled: failed to create input device (uinput: {}, xtest: {})",
                                            e, xtest_err
                                        )));
                                    }
                                }
                            }
                        }
                    } else if let Some(d) = self.input_device.as_mut() {
                        d.set_capturable(capturable.clone());
                    }
                } else if !portal_selected {
                    // When uinput_support is false, try XTest first, then fall back to AutoPilot
                    if self.input_device.as_ref().map_or(true, |d| {
                        client_name_changed
                            || (d.device_type() != InputDeviceType::XTestDevice
                                && d.device_type() != InputDeviceType::AutoPilotDevice)
                    }) {
                        // Try XTest first
                        match crate::input::xtest_device::XTestDevice::new(capturable.clone()) {
                            Ok(xtest_device) => {
                                debug!("Using XTest device for input");
                                self.input_device = Some(Box::new(xtest_device));
                            }
                            Err(e) => {
                                if has_x_display() {
                                    debug!(
                                        "XTest not available ({}), falling back to AutoPilot",
                                        e
                                    );
                                    self.input_device = Some(Box::new(
                                        crate::input::autopilot_device::AutoPilotDevice::new(
                                            capturable.clone(),
                                        ),
                                    ));
                                } else {
                                    warn!(
                                        "XTest not available ({}), DISPLAY is unset; continuing without input backend",
                                        e
                                    );
                                    self.input_device = None;
                                    self.send_message(MessageOutbound::Error(
                                        "Input disabled: no usable Linux input backend is available."
                                            .to_string(),
                                    ));
                                }
                            }
                        }
                    } else if let Some(d) = self.input_device.as_mut() {
                        d.set_capturable(capturable.clone());
                    }
                }
            }

            #[cfg(target_os = "macos")]
            if self.input_device.is_none() {
                self.input_device = Some(Box::new(
                    crate::input::autopilot_device::AutoPilotDevice::new(capturable.clone()),
                ));
            } else {
                self.input_device
                    .as_mut()
                    .map(|d| d.set_capturable(capturable.clone()));
            }
            #[cfg(target_os = "windows")]
            if self.input_device.is_none() {
                self.input_device = Some(Box::new(
                    crate::input::autopilot_device_win::WindowsInput::new(capturable.clone()),
                ));
            } else {
                self.input_device
                    .as_mut()
                    .map(|d| d.set_capturable(capturable.clone()));
            }

            let input_backend = self
                .input_device
                .as_ref()
                .map(|device| {
                    #[cfg(target_os = "windows")]
                    if device.device_type() == InputDeviceType::WindowsInput {
                        return crate::input::autopilot_device_win::input_backend_label();
                    }
                    device.device_type().label().to_string()
                })
                .unwrap_or_else(|| "不可用".to_string());
            send_input_status(&mut self.sender, &input_backend);

            self.video_sender
                .send(VideoCommands::Start(VideoConfig {
                    capturable,
                    capture_cursor: config.capture_cursor,
                    max_width: config.max_width,
                    max_height: config.max_height,
                    frame_rate: config.frame_rate,
                }))
                .unwrap();
        }
    }
}

fn handle_video<S: WeylusSender + Clone + 'static>(
    receiver: mpsc::Receiver<VideoCommands>,
    mut sender: S,
    encoder_options: EncoderOptions,
) {
    const EFFECTIVE_INIFINITY: Duration = Duration::from_secs(3600 * 24 * 365 * 200);

    let mut recorder: Option<Box<dyn Recorder>> = None;
    let mut video_encoder: Option<Box<VideoEncoder>> = None;
    let mut capture_backend = "未启动".to_string();
    let mut encoder_backend = "未启动".to_string();

    let mut max_width = 1920;
    let mut max_height = 1080;
    let mut frame_duration = EFFECTIVE_INIFINITY;
    let mut last_frame = Instant::now();
    let mut paused = false;

    loop {
        let now = Instant::now();
        let elapsed = now - last_frame;
        let frames_passed = (elapsed.as_secs_f64() / frame_duration.as_secs_f64()) as u32;
        let next_frame = last_frame + (frames_passed + 1) * frame_duration;
        let timeout = next_frame - now;
        last_frame = next_frame;

        if frames_passed > 0 {
            trace!("Dropped {frames_passed} frame(s)!");
        }

        match receiver.recv_timeout(if paused { EFFECTIVE_INIFINITY } else { timeout }) {
            Ok(VideoCommands::Start(config)) => {
                #[allow(unused_assignments)]
                {
                    // gstpipewire can not handle setting a pipeline's state to Null after another
                    // pipeline has been created and its state has been set to Play.
                    // This line makes sure that there always is only a single recorder and thus
                    // single pipeline in this thread by forcing rust to call the destructor of the
                    // current pipeline here, right before creating a new pipeline.
                    // See: https://gitlab.freedesktop.org/pipewire/pipewire/-/issues/986
                    //
                    // This shouldn't affect other Recorder trait objects.
                    recorder = None;
                }
                const MAX_RETRIES: u32 = 5;
                const RETRY_DELAY: Duration = Duration::from_millis(500);
                let mut result = config.capturable.recorder(config.capture_cursor);
                for attempt in 1..MAX_RETRIES {
                    if result.is_ok() {
                        break;
                    }
                    warn!(
                        "Failed to init screen cast (attempt {}/{}), retrying...",
                        attempt, MAX_RETRIES
                    );
                    std::thread::sleep(RETRY_DELAY);
                    result = config.capturable.recorder(config.capture_cursor);
                }
                match result {
                    Ok(r) => {
                        capture_backend = r.backend_name().to_string();
                        encoder_backend = "等待视频流".to_string();
                        recorder = Some(r);
                        video_encoder = None;
                        max_width = config.max_width;
                        max_height = config.max_height;
                        send_message(&mut sender, MessageOutbound::ConfigOk);
                        send_runtime_status(&mut sender, &capture_backend, &encoder_backend);
                    }
                    Err(err) => {
                        capture_backend = "初始化失败".to_string();
                        encoder_backend = "未启动".to_string();
                        warn!("Failed to init screen cast: {}!", err);
                        send_message(
                            &mut sender,
                            MessageOutbound::Error(format!("Failed to init screen cast: {err}!")),
                        );
                        send_runtime_status(&mut sender, &capture_backend, &encoder_backend);
                    }
                }
                last_frame = Instant::now();

                // The Duration type can not handle infinity, if the frame rate is set to 0 we just
                // set the duration between two frames to a very long one, which is effectively
                // infinity.
                let d = 1.0 / config.frame_rate;
                frame_duration = if d.is_finite() {
                    Duration::from_secs_f64(d)
                } else {
                    EFFECTIVE_INIFINITY
                };
                frame_duration = frame_duration.min(EFFECTIVE_INIFINITY);
            }
            Ok(VideoCommands::Pause) => {
                paused = true;
            }
            Ok(VideoCommands::Resume) => {
                paused = false;
            }
            Ok(VideoCommands::Restart) => {
                video_encoder = None;
                encoder_backend = "重启中".to_string();
                send_runtime_status(&mut sender, &capture_backend, &encoder_backend);
            }
            Err(RecvTimeoutError::Timeout) => {
                if recorder.is_none() {
                    warn!("Screen capture not initalized, can not send video frame!");
                    continue;
                }
                let pixel_data = recorder.as_mut().unwrap().capture();
                if let Err(err) = pixel_data {
                    warn!("Error capturing screen: {}", err);
                    continue;
                }
                let pixel_data = pixel_data.unwrap();
                let (width_in, height_in) = pixel_data.size();
                let scale =
                    (max_width as f64 / width_in as f64).min(max_height as f64 / height_in as f64);
                // limit video to 4K
                let scale_max = (3840.0 / width_in as f64).min(2160.0 / height_in as f64);
                let scale = scale.min(scale_max);
                let mut width_out = width_in;
                let mut height_out = height_in;
                if scale < 1.0 {
                    width_out = (width_out as f64 * scale) as usize;
                    height_out = (height_out as f64 * scale) as usize;
                }
                // video encoder is not setup or setup for encoding the wrong size: restart it
                if video_encoder.is_none()
                    || !video_encoder
                        .as_ref()
                        .unwrap()
                        .check_size(width_in, height_in, width_out, height_out)
                {
                    send_message(&mut sender, MessageOutbound::NewVideo);
                    let mut video_sender = sender.clone();
                    let res = VideoEncoder::new(
                        width_in,
                        height_in,
                        width_out,
                        height_out,
                        move |data| {
                            if let Err(err) = video_sender.send_video(data) {
                                warn!("Failed to send video frame: {err}!");
                            }
                        },
                        encoder_options,
                    );
                    match res {
                        Ok(r) => {
                            encoder_backend = r.codec_name();
                            video_encoder = Some(r);
                            send_runtime_status(&mut sender, &capture_backend, &encoder_backend);
                        }
                        Err(e) => {
                            encoder_backend = "初始化失败".to_string();
                            send_runtime_status(&mut sender, &capture_backend, &encoder_backend);
                            warn!("{}", e);
                            continue;
                        }
                    };
                }
                let video_encoder = video_encoder.as_mut().unwrap();
                video_encoder.encode(pixel_data);
            }
            // stop thread once the channel is closed
            Err(RecvTimeoutError::Disconnected) => return,
        };
    }
}

pub struct WsWeylusReceiver {
    recv: tokio::sync::mpsc::Receiver<MessageInbound>,
}

impl Iterator for WsWeylusReceiver {
    type Item = Result<MessageInbound, Infallible>;

    fn next(&mut self) -> Option<Self::Item> {
        self.recv.blocking_recv().map(Ok)
    }
}

impl WeylusReceiver for WsWeylusReceiver {
    type Error = Infallible;
}

pub enum WsMessage {
    Frame(Frame<'static>),
    Video(Vec<u8>),
    MessageOutbound(MessageOutbound),
}

unsafe impl Send for WsMessage {}

#[derive(Clone)]
pub struct WsWeylusSender {
    sender: tokio::sync::mpsc::Sender<WsMessage>,
}

impl WeylusSender for WsWeylusSender {
    type Error = tokio::sync::mpsc::error::SendError<WsMessage>;

    fn send_message(&mut self, message: MessageOutbound) -> Result<(), Self::Error> {
        self.sender
            .blocking_send(WsMessage::MessageOutbound(message))
    }

    fn send_video(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.sender.blocking_send(WsMessage::Video(bytes.to_vec()))
    }
}

pub fn weylus_websocket_channel(
    websocket: WebSocket<TokioIo<Upgraded>>,
    semaphore_shutdown: Arc<tokio::sync::Semaphore>,
) -> (WsWeylusSender, WsWeylusReceiver) {
    let (rx, mut tx) = websocket.split(|ws| tokio::io::split(ws));

    let mut rx = FragmentCollectorRead::new(rx);

    let (sender_inbound, receiver_inbound) = channel::<MessageInbound>(32);
    let (sender_outbound, mut receiver_outbound) = channel::<WsMessage>(32);

    {
        let sender_outbound = sender_outbound.clone();
        tokio::spawn(async move {
            let mut send_fn = |frame| async {
                if let Err(err) = sender_outbound.send(WsMessage::Frame(frame)).await {
                    warn!("Failed to send websocket frame while receiving fragmented frame: {err}.")
                };
                Ok(())
            };

            loop {
                let fut = rx.read_frame::<_, WebSocketError>(&mut send_fn);

                let frame = tokio::select! {
                    _ = semaphore_shutdown.acquire() => break,
                    frame = fut => match frame {
                        Ok(frame) => frame,
                        Err(err) => {
                            warn!("Invalid websocket frame: {err}.");
                            break;
                        },
                    },
                };
                match frame.opcode {
                    OpCode::Close => break,
                    OpCode::Text => match serde_json::from_slice(&frame.payload) {
                        Ok(msg) => {
                            if let Err(err) = sender_inbound.send(msg).await {
                                warn!("Failed to forward inbound message to WeylusClientHandler: {err}.");
                            }
                        }
                        Err(err) => warn!("Failed to parse message: {err}"),
                    },
                    _ => {}
                }
            }
        });
    }

    tokio::spawn(async move {
        loop {
            let msg = if let Some(msg) = receiver_outbound.recv().await {
                msg
            } else {
                break;
            };

            match msg {
                WsMessage::Frame(frame) => {
                    if let Err(err) = tx.write_frame(frame).await {
                        if let WebSocketError::ConnectionClosed = err {
                            break;
                        }
                        warn!("Failed to send frame: {err}");
                    }
                }
                WsMessage::Video(data) => {
                    if let Err(err) = tx.write_frame(Frame::binary(data.into())).await {
                        if let WebSocketError::ConnectionClosed = err {
                            break;
                        }
                        warn!("Failed to send video frame: {err}");
                    }
                }
                WsMessage::MessageOutbound(msg) => {
                    let json_string = serde_json::to_string(&msg).unwrap();
                    let data = json_string.as_bytes();
                    if let Err(err) = tx.write_frame(Frame::text(data.into())).await {
                        if let WebSocketError::ConnectionClosed = err {
                            break;
                        }
                        warn!("Failed to send outbound message: {err}");
                    }
                }
            }
        }
    });

    (
        WsWeylusSender {
            sender: sender_outbound,
        },
        WsWeylusReceiver {
            recv: receiver_inbound,
        },
    )
}
