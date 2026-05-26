use crate::capturable::Capturable;
use crate::protocol::{KeyboardEvent, PointerEvent, WheelEvent};

#[derive(PartialEq, Eq)]
pub enum InputDeviceType {
    AutoPilotDevice,
    UInputDevice,
    #[cfg(target_os = "linux")]
    WaylandPortalDevice,
    #[cfg(target_os = "linux")]
    XTestDevice,
    #[cfg(target_os = "windows")]
    WindowsInput,
}

impl InputDeviceType {
    pub fn label(&self) -> &'static str {
        match self {
            InputDeviceType::AutoPilotDevice => "AutoPilot",
            InputDeviceType::UInputDevice => "uinput",
            #[cfg(target_os = "linux")]
            InputDeviceType::WaylandPortalDevice => "Wayland Portal",
            #[cfg(target_os = "linux")]
            InputDeviceType::XTestDevice => "XTest",
            #[cfg(target_os = "windows")]
            InputDeviceType::WindowsInput => "Windows SendInput",
        }
    }
}

pub trait InputDevice {
    fn send_wheel_event(&mut self, event: &WheelEvent);
    fn send_pointer_event(&mut self, event: &PointerEvent);
    fn send_keyboard_event(&mut self, event: &KeyboardEvent);
    fn drain_keyboard_status(&mut self) -> Vec<String> {
        Vec::new()
    }
    fn set_capturable(&mut self, capturable: Box<dyn Capturable>);
    fn device_type(&self) -> InputDeviceType;
}
