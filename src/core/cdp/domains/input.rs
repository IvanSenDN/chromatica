use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub enum KeyEventType {
    #[serde(rename = "keyDown")]
    KeyDown,
    #[serde(rename = "keyUp")]
    KeyUp,
    #[serde(rename = "rawKeyDown")]
    RawKeyDown,
    #[serde(rename = "char")]
    Char,
}

#[derive(Serialize)]
pub enum MouseEventType {
    #[serde(rename = "mousePressed")]
    MousePressed,
    #[serde(rename = "mouseReleased")]
    MouseReleased,
    #[serde(rename = "mouseMoved")]
    MouseMoved,
    #[serde(rename = "mouseWheel")]
    MouseWheel,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TouchEventType {
    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Modifier {
    Alt,
    Control,
    Meta,
    Shift,
}

impl Modifier {
    pub fn to_i32(&self) -> i32 {
        match self {
            Modifier::Alt => 1,
            Modifier::Control => 2,
            Modifier::Meta => 4,
            Modifier::Shift => 8,
        }
    }
}

#[derive(Serialize)]
pub enum MouseButton {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "middle")]
    Middle,
    #[serde(rename = "back")]
    Back,
    #[serde(rename = "forward")]
    Forward,
}

pub type TimeSinceEpoch = f64;

#[derive(Serialize)]
pub enum GestureSourceType {
    #[serde(rename = "touch")]
    Touch,
    #[serde(rename = "mouse")]
    Mouse,
    #[serde(rename = "default")]
    Default,
}

///https://chromedevtools.github.io/devtools-protocol/tot/Input/#type-TouchPoint
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TouchPoint {
    pub x: f64,
    pub y: f64,
    pub radius_x: Option<f64>,
    pub radius_y: Option<f64>,
    pub rotation_angle: Option<f64>,
    pub force: Option<f64>,
    pub tangential_pressure: Option<f64>,
    pub tilt_x: Option<f64>,
    pub tilt_y: Option<f64>,
    pub twist: Option<i32>,
    pub id: Option<u32>,
}

impl TouchPoint {
    pub fn default(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            radius_x: None,
            radius_y: None,
            rotation_angle: None,
            force: None,
            tangential_pressure: None,
            tilt_x: None,
            tilt_y: None,
            twist: None,
            id: None,
        }
    }

    pub fn radius_x(mut self, radius_x: f64) -> Self {
        self.radius_x = Some(radius_x);
        self
    }

    pub fn radius_y(mut self, radius_y: f64) -> Self {
        self.radius_y = Some(radius_y);
        self
    }

    pub fn rotation_angle(mut self, rotation_angle: f64) -> Self {
        self.rotation_angle = Some(rotation_angle);
        self
    }

    pub fn force(mut self, force: f64) -> Self {
        self.force = Some(force);
        self
    }

    pub fn tangential_pressure(mut self, tangential_pressure: f64) -> Self {
        self.tangential_pressure = Some(tangential_pressure);
        self
    }

    pub fn tilt_x(mut self, tilt_x: f64) -> Self {
        self.tilt_x = Some(tilt_x);
        self
    }

    pub fn tilt_y(mut self, tilt_y: f64) -> Self {
        self.tilt_y = Some(tilt_y);
        self
    }

    pub fn twist(mut self, twist: i32) -> Self {
        self.twist = Some(twist);
        self
    }

    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}
///https://chromedevtools.github.io/devtools-protocol/tot/Input/#type-DragDataItem
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DragDataItem {
    pub mime_type: String,
    pub data: String,
    pub title: Option<String>,
    pub base_url: Option<String>,
}

///https://chromedevtools.github.io/devtools-protocol/tot/Input/#type-DragData
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DragData {
    pub items: Vec<DragDataItem>,
    pub files: Vec<String>,
    pub drag_operations_mask: Option<i32>, //Copy = 1, Link = 2, Move = 16
}

///Deserializable events
#[derive(Debug, Deserialize, Clone)]
pub struct DragIntercepted {
    pub data: DragData,
}

///Serializable requests
#[derive(Serialize)]
pub struct DispatchKeyEvent<'a> {
    #[serde(rename = "type")]
    pub key_type: KeyEventType,
    #[serde(rename = "modifiers", skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<i32>,
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<TimeSinceEpoch>,
    #[serde(rename = "text", skip_serializing_if = "Option::is_none")]
    pub text: Option<&'a str>,
    #[serde(rename = "unmodifiedText", skip_serializing_if = "Option::is_none")]
    pub unmodified_text: Option<&'a str>,
    #[serde(rename = "keyIdentifier", skip_serializing_if = "Option::is_none")]
    pub key_identifier: Option<&'a str>,
    #[serde(rename = "code", skip_serializing_if = "Option::is_none")]
    pub code: Option<&'a str>,
    #[serde(rename = "key", skip_serializing_if = "Option::is_none")]
    pub key: Option<&'a str>,
    #[serde(
        rename = "windowsVirtualKeyCode",
        skip_serializing_if = "Option::is_none"
    )]
    pub windows_virtual_key_code: Option<i32>,
    #[serde(
        rename = "nativeVirtualKeyCode",
        skip_serializing_if = "Option::is_none"
    )]
    pub native_virtual_key_code: Option<i32>,
    #[serde(rename = "autoRepeat", skip_serializing_if = "Option::is_none")]
    pub auto_repeat: Option<bool>,
    #[serde(rename = "isKeypad", skip_serializing_if = "Option::is_none")]
    pub is_keypad: Option<bool>,
    #[serde(rename = "isSystemKey", skip_serializing_if = "Option::is_none")]
    pub is_system_key: Option<bool>,
    #[serde(rename = "location", skip_serializing_if = "Option::is_none")]
    pub location: Option<i32>,
    #[serde(rename = "commands", skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<&'a str>>,
}

impl<'a> DispatchKeyEvent<'a> {
    // pub fn default(key_type: KeyEventType) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         key_type,
    //         modifiers: None,
    //         timestamp: None,
    //         text: None,
    //         unmodified_text: None,
    //         key_identifier: None,
    //         code: None,
    //         key: None,
    //         windows_virtual_key_code: None,
    //         native_virtual_key_code: None,
    //         auto_repeat: None,
    //         is_keypad: None,
    //         is_system_key: None,
    //         location: None,
    //         commands: None,
    //     })
    //     .unwrap()
    // }

    pub fn default(key_type: KeyEventType, text: &'a str) -> Self {
        Self {
            key_type,
            modifiers: None,
            timestamp: None,
            text: Some(text),
            unmodified_text: None,
            key_identifier: None,
            code: None,
            key: None,
            windows_virtual_key_code: None,
            native_virtual_key_code: None,
            auto_repeat: None,
            is_keypad: None,
            is_system_key: None,
            location: None,
            commands: None,
        }
    }

    pub fn new(key_type: KeyEventType, text: &'a str) -> Self {
        Self {
            key_type,
            modifiers: None,
            timestamp: None,
            text: Some(text),
            unmodified_text: None,
            key_identifier: None,
            code: None,
            key: None,
            windows_virtual_key_code: None,
            native_virtual_key_code: None,
            auto_repeat: None,
            is_keypad: None,
            is_system_key: None,
            location: None,
            commands: None,
        }
    }

    pub fn modifiers(mut self, modifiers: Modifier) -> Self {
        self.modifiers = Some(modifiers.to_i32());
        self
    }

    pub fn timestamp(mut self, timestamp: TimeSinceEpoch) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn text(mut self, text: &'a str) -> Self {
        self.text = Some(text);
        self
    }

    pub fn unmodified_text(mut self, unmodified_text: &'a str) -> Self {
        self.unmodified_text = Some(unmodified_text);
        self
    }

    pub fn key_identifier(mut self, key_identifier: &'a str) -> Self {
        self.key_identifier = Some(key_identifier);
        self
    }

    pub fn code(mut self, code: &'a str) -> Self {
        self.code = Some(code);
        self
    }

    pub fn key(mut self, key: &'a str) -> Self {
        self.key = Some(key);
        self
    }

    pub fn windows_virtual_key_code(mut self, code: i32) -> Self {
        self.windows_virtual_key_code = Some(code);
        self
    }

    pub fn native_virtual_key_code(mut self, code: i32) -> Self {
        self.native_virtual_key_code = Some(code);
        self
    }

    pub fn auto_repeat(mut self, auto_repeat: bool) -> Self {
        self.auto_repeat = Some(auto_repeat);
        self
    }

    pub fn is_keypad(mut self, is_keypad: bool) -> Self {
        self.is_keypad = Some(is_keypad);
        self
    }

    pub fn is_system_key(mut self, is_system_key: bool) -> Self {
        self.is_system_key = Some(is_system_key);
        self
    }

    pub fn location(mut self, location: i32) -> Self {
        self.location = Some(location);
        self
    }

    pub fn commands(mut self, commands: Vec<&'a str>) -> Self {
        self.commands = Some(commands);
        self
    }

    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchMouseEvent<'a> {
    #[serde(rename = "type")]
    pub event_type: MouseEventType,
    pub x: f32,
    pub y: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<TimeSinceEpoch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button: Option<MouseButton>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tangential_pressure: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilt_x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilt_y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twist: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer_type: Option<&'a str>,
}

impl<'a> DispatchMouseEvent<'a> {
    // pub fn default(event_type: MouseEventType, x: f64, y: f64) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         event_type,
    //         x,
    //         y,
    //         modifiers: None,
    //         timestamp: None,
    //         button: None,
    //         buttons: None,
    //         click_count: None,
    //         force: None,
    //         tangential_pressure: None,
    //         tilt_x: None,
    //         tilt_y: None,
    //         twist: None,
    //         delta_x: None,
    //         delta_y: None,
    //         pointer_type: None,
    //     })
    //     .unwrap()
    // }

    pub fn default(event_type: MouseEventType, x: f32, y: f32) -> Self {
        Self {
            event_type,
            x,
            y,
            modifiers: None,
            timestamp: None,
            button: None,
            buttons: None,
            click_count: None,
            force: None,
            tangential_pressure: None,
            tilt_x: None,
            tilt_y: None,
            twist: None,
            delta_x: None,
            delta_y: None,
            pointer_type: None,
        }
    }

    pub fn new(event_type: MouseEventType, x: f32, y: f32) -> Self {
        Self {
            event_type,
            x,
            y,
            modifiers: None,
            timestamp: None,
            button: None,
            buttons: None,
            click_count: None,
            force: None,
            tangential_pressure: None,
            tilt_x: None,
            tilt_y: None,
            twist: None,
            delta_x: None,
            delta_y: None,
            pointer_type: None,
        }
    }

    pub fn modifiers(mut self, modifiers: Modifier) -> Self {
        self.modifiers = Some(modifiers.to_i32());
        self
    }

    pub fn timestamp(mut self, timestamp: TimeSinceEpoch) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn button(mut self, button: MouseButton) -> Self {
        self.button = Some(button);
        self
    }

    pub fn buttons(mut self, buttons: i32) -> Self {
        self.buttons = Some(buttons);
        self
    }

    pub fn click_count(mut self, click_count: i32) -> Self {
        self.click_count = Some(click_count);
        self
    }

    pub fn force(mut self, force: f64) -> Self {
        self.force = Some(force);
        self
    }

    pub fn tangential_pressure(mut self, tangential_pressure: f64) -> Self {
        self.tangential_pressure = Some(tangential_pressure);
        self
    }

    pub fn tilt_x(mut self, tilt_x: f64) -> Self {
        self.tilt_x = Some(tilt_x);
        self
    }

    pub fn tilt_y(mut self, tilt_y: f64) -> Self {
        self.tilt_y = Some(tilt_y);
        self
    }

    pub fn twist(mut self, twist: i32) -> Self {
        self.twist = Some(twist);
        self
    }

    pub fn delta_x(mut self, delta_x: f64) -> Self {
        self.delta_x = Some(delta_x);
        self
    }

    pub fn delta_y(mut self, delta_y: f64) -> Self {
        self.delta_y = Some(delta_y);
        self
    }

    pub fn pointer_type(mut self, pointer_type: &'a str) -> Self {
        self.pointer_type = Some(pointer_type);
        self
    }

    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct DispatchTouchEvent {
    #[serde(rename = "type")]
    pub event_type: TouchEventType,
    #[serde(rename = "touchPoints")]
    pub touch_points: Vec<TouchPoint>,
    #[serde(rename = "modifiers", skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<i32>,
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<TimeSinceEpoch>,
}

impl DispatchTouchEvent {
    // pub fn default(event_type: TouchEventType, touch_points: Vec<TouchPoint>) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         event_type,
    //         touch_points,
    //         modifiers: None,
    //         timestamp: None,
    //     })
    //     .unwrap()
    // }

    pub fn default(event_type: TouchEventType, touch_points: Vec<TouchPoint>) -> Self {
        Self {
            event_type,
            touch_points,
            modifiers: None,
            timestamp: None,
        }
    }

    pub fn new(event_type: TouchEventType, touch_points: Vec<TouchPoint>) -> Self {
        Self {
            event_type,
            touch_points,
            modifiers: None,
            timestamp: None,
        }
    }

    pub fn modifiers(mut self, modifiers: Modifier) -> Self {
        self.modifiers = Some(modifiers.to_i32());
        self
    }

    pub fn timestamp(mut self, timestamp: TimeSinceEpoch) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}
