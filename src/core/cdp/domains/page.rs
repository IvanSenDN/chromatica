use super::dom::BackendNodeId;
use super::network::LoaderId;
use super::runtime::ExecutionContextId;
use serde::{Deserialize, Serialize};
pub type FrameId = String;
pub type DialogType = String; //alert, confirm, prompt, beforeunload
pub type TransitionType = String;
pub type ScriptIdentifier = String;
pub type ReferrerPolicy = String;

#[derive(Deserialize)]
pub struct NavigationEntry {
    #[serde(rename = "id")]
    pub id: u64,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "userTypedURL")]
    pub user_typed_url: String,
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "transitionType")]
    pub transition_type: TransitionType,
}

///Deserializable structs
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub id: FrameId,
    pub parent_id: Option<FrameId>,
    // pub loader_id: LoaderId,
    // pub name: Option<String>,
    pub url: String,
    // pub url_fragment: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrameTree {
    pub frame: Frame,
    // pub child_frames: Option<Vec<FrameTree>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetFrameTreeResponse {
    pub frame_tree: FrameTree,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CaptureScreenshotResponse {
    pub data: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrintToPDFResponse {
    pub data: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddScriptToEvaluateOnNewDocumentResponse {
    pub identifier: ScriptIdentifier,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateIsolatedWorldResponse {
    pub execution_context_id: ExecutionContextId,
}

///Deserializable events
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileChooserOpened {
    pub frame_id: FrameId,
    pub mode: String, //selectSingle, selectMultiple
    pub backend_node_id: Option<BackendNodeId>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrameAttached {
    pub frame_id: String,
    pub parent_frame_id: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrameDetached {
    pub frame_id: FrameId,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrameNavigated {
    pub frame: Frame,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleEvent {
    pub frame_id: FrameId,
    // pub loader_id: LoaderId,
    pub name: String,
    // pub timestamp: MonotonicTime,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JavascriptDialogOpening {
    pub url: String,
    pub frame_id: Option<FrameId>,
    pub message: String,
    #[serde(rename = "type")]
    pub dialog_type: DialogType,
    pub has_browser_handler: bool,
    pub default_prompt: Option<String>,
}

///Serializable structs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

impl Viewport {
    ///Actually needed for screenshoting elements
    pub fn default(x: f32, y: f32, width: f32, height: f32, scale: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            scale,
        }
    }
}

#[derive(Serialize)]
pub struct AddScriptToEvaluateOnNewDocument<'a> {
    #[serde(rename = "source")]
    pub source: &'a str,
    #[serde(rename = "worldName", skip_serializing_if = "Option::is_none")]
    pub world_name: Option<&'a str>,
    #[serde(rename = "runImmediately", default = "default_true")]
    pub run_immediately: bool,
}

impl<'a> AddScriptToEvaluateOnNewDocument<'a> {
    pub fn default(source: &'a str) -> Self {
        Self {
            source,
            world_name: None,
            run_immediately: true,
        }
    }
    pub fn new(source: &'a str, run_immediately: bool) -> Self {
        Self {
            source,
            world_name: None,
            run_immediately,
        }
    }

    pub fn world_name(mut self, value: &'a str) -> Self {
        self.world_name = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct BringToFront {}

impl BringToFront {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct CaptureScreenshot<'a> {
    #[serde(rename = "format", skip_serializing_if = "Option::is_none")]
    pub format: Option<&'a str>,
    #[serde(rename = "quality", skip_serializing_if = "Option::is_none")]
    pub quality: Option<u64>,
    #[serde(rename = "clip", skip_serializing_if = "Option::is_none")]
    pub clip: Option<Viewport>,
    #[serde(rename = "fromSurface", skip_serializing_if = "Option::is_none")]
    pub from_surface: Option<bool>,
    #[serde(
        rename = "captureBeyondViewport",
        skip_serializing_if = "Option::is_none"
    )]
    pub capture_beyond_viewport: Option<bool>,
}

impl<'a> CaptureScreenshot<'a> {
    pub fn default() -> Self {
        Self {
            format: Some("png"),
            quality: None,
            clip: None,
            from_surface: None,
            capture_beyond_viewport: None,
        }
    }

    pub fn new() -> Self {
        Self {
            format: None,
            quality: None,
            clip: None,
            from_surface: None,
            capture_beyond_viewport: None,
        }
    }

    pub fn format(mut self, value: &'a str) -> Self {
        self.format = Some(value);
        self
    }

    pub fn quality(mut self, value: u64) -> Self {
        self.quality = Some(value);
        self
    }

    pub fn clip(mut self, value: Viewport) -> Self {
        self.clip = Some(value);
        self
    }

    pub fn from_surface(mut self, value: bool) -> Self {
        self.from_surface = Some(value);
        self
    }

    pub fn capture_beyond_viewport(mut self, value: bool) -> Self {
        self.capture_beyond_viewport = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct PageClose {}

impl PageClose {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct CreateIsolatedWorld<'a> {
    #[serde(rename = "frameId", skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<&'a FrameId>,
    #[serde(rename = "worldName", skip_serializing_if = "Option::is_none")]
    pub world_name: Option<&'a str>,
    ///grantUniveralAccess is actually bugged in CDP and actually doesn't work: https://issues.chromium.org/issues/40740901
    #[serde(
        rename = "grantUniveralAccess",
        skip_serializing_if = "Option::is_none"
    )]
    pub grant_universal_access: Option<bool>,
}

impl<'a> CreateIsolatedWorld<'a> {
    pub fn default(frame_id: &'a FrameId) -> Self {
        Self {
            frame_id: Some(frame_id),
            world_name: None,
            grant_universal_access: None,
        }
    }

    pub fn new(frame_id: &'a FrameId) -> Self {
        Self {
            frame_id: Some(frame_id),
            world_name: None,
            grant_universal_access: None,
        }
    }

    pub fn world_name(mut self, value: &'a str) -> Self {
        self.world_name = Some(value);
        self
    }

    pub fn grant_universal_access(mut self, value: bool) -> Self {
        self.grant_universal_access = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct PageEnable {}

impl PageEnable {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct PageDisable {}

impl PageDisable {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct GetFrameTree {}

impl GetFrameTree {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct HandleJavaScriptDialog<'a> {
    #[serde(rename = "accept")]
    pub accept: bool,
    #[serde(rename = "promptText", skip_serializing_if = "Option::is_none")]
    pub prompt_text: Option<&'a str>,
}

impl<'a> HandleJavaScriptDialog<'a> {
    pub fn default() -> Self {
        Self {
            accept: true,
            prompt_text: None,
        }
    }

    pub fn new(accept: bool) -> Self {
        Self {
            accept,
            prompt_text: None,
        }
    }

    pub fn prompt_text(mut self, value: &'a str) -> Self {
        self.prompt_text = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct Navigate<'a> {
    #[serde(rename = "url")]
    pub url: &'a str,
    #[serde(rename = "referrer", skip_serializing_if = "Option::is_none")]
    pub referrer: Option<&'a str>,
    #[serde(rename = "referrerPolicy", skip_serializing_if = "Option::is_none")]
    pub referrer_policy: Option<&'a str>,
    #[serde(rename = "transitionType", skip_serializing_if = "Option::is_none")]
    pub transition_type: Option<&'a str>,
    ///We will use this for each navigation request
    #[serde(rename = "frameId")]
    pub frame_id: &'a FrameId,
}

impl<'a> Navigate<'a> {
    pub fn default(url: &'a str, frame_id: &'a FrameId) -> Self {
        Self {
            url,
            referrer: None,
            referrer_policy: None,
            transition_type: None,
            frame_id,
        }
    }

    pub fn new(url: &'a str, frame_id: &'a FrameId) -> Self {
        Self {
            url,
            referrer: None,
            referrer_policy: None,
            transition_type: None,
            frame_id,
        }
    }

    pub fn referrer(mut self, value: &'a str) -> Self {
        self.referrer = Some(value);
        self
    }

    pub fn referrer_policy(mut self, value: &'a str) -> Self {
        self.referrer_policy = Some(value);
        self
    }

    pub fn transition_type(mut self, value: &'a str) -> Self {
        self.transition_type = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct PrintToPDF<'a> {
    #[serde(rename = "landscape", skip_serializing_if = "Option::is_none")]
    pub landscape: Option<bool>,
    #[serde(
        rename = "displayHeaderFooter",
        skip_serializing_if = "Option::is_none"
    )]
    pub display_header_footer: Option<bool>,
    #[serde(rename = "printBackground", skip_serializing_if = "Option::is_none")]
    pub print_background: Option<bool>,
    #[serde(rename = "scale", skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(rename = "paperWidth", skip_serializing_if = "Option::is_none")]
    pub paper_width: Option<f64>,
    #[serde(rename = "paperHeight", skip_serializing_if = "Option::is_none")]
    pub paper_height: Option<f64>,
    #[serde(rename = "marginTop", skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<f64>,
    #[serde(rename = "marginBottom", skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<f64>,
    #[serde(rename = "marginLeft", skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<f64>,
    #[serde(rename = "marginRight", skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<f64>,
    #[serde(rename = "pageRanges", skip_serializing_if = "Option::is_none")]
    pub page_ranges: Option<&'a str>,
    #[serde(
        rename = "ignoreInvalidPageRanges",
        skip_serializing_if = "Option::is_none"
    )]
    pub ignore_invalid_page_ranges: Option<bool>,
    #[serde(rename = "headerTemplate", skip_serializing_if = "Option::is_none")]
    pub header_template: Option<&'a str>,
    #[serde(rename = "footerTemplate", skip_serializing_if = "Option::is_none")]
    pub footer_template: Option<&'a str>,
    #[serde(rename = "preferCssPageSize", skip_serializing_if = "Option::is_none")]
    pub prefer_css_page_size: Option<bool>,
}

impl<'a> PrintToPDF<'a> {
    pub fn default() -> Self {
        Self {
            landscape: None,
            display_header_footer: None,
            print_background: None,
            scale: None,
            paper_width: None,
            paper_height: None,
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            page_ranges: None,
            ignore_invalid_page_ranges: None,
            header_template: None,
            footer_template: None,
            prefer_css_page_size: None,
        }
    }

    pub fn new() -> Self {
        Self {
            landscape: None,
            display_header_footer: None,
            print_background: None,
            scale: None,
            paper_width: None,
            paper_height: None,
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            page_ranges: None,
            ignore_invalid_page_ranges: None,
            header_template: None,
            footer_template: None,
            prefer_css_page_size: None,
        }
    }

    pub fn landscape(mut self, value: bool) -> Self {
        self.landscape = Some(value);
        self
    }

    pub fn display_header_footer(mut self, value: bool) -> Self {
        self.display_header_footer = Some(value);
        self
    }

    pub fn print_background(mut self, value: bool) -> Self {
        self.print_background = Some(value);
        self
    }

    pub fn scale(mut self, value: f64) -> Self {
        self.scale = Some(value);
        self
    }

    pub fn paper_width(mut self, value: f64) -> Self {
        self.paper_width = Some(value);
        self
    }

    pub fn paper_height(mut self, value: f64) -> Self {
        self.paper_height = Some(value);
        self
    }

    pub fn margin_top(mut self, value: f64) -> Self {
        self.margin_top = Some(value);
        self
    }

    pub fn margin_bottom(mut self, value: f64) -> Self {
        self.margin_bottom = Some(value);
        self
    }

    pub fn margin_left(mut self, value: f64) -> Self {
        self.margin_left = Some(value);
        self
    }

    pub fn margin_right(mut self, value: f64) -> Self {
        self.margin_right = Some(value);
        self
    }

    pub fn page_ranges(mut self, value: &'a str) -> Self {
        self.page_ranges = Some(value);
        self
    }

    pub fn ignore_invalid_page_ranges(mut self, value: bool) -> Self {
        self.ignore_invalid_page_ranges = Some(value);
        self
    }

    pub fn header_template(mut self, value: &'a str) -> Self {
        self.header_template = Some(value);
        self
    }

    pub fn footer_template(mut self, value: &'a str) -> Self {
        self.footer_template = Some(value);
        self
    }

    pub fn prefer_css_page_size(mut self, value: bool) -> Self {
        self.prefer_css_page_size = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct Reload<'a> {
    #[serde(rename = "ignoreCache", skip_serializing_if = "Option::is_none")]
    pub ignore_cache: Option<bool>,
    #[serde(
        rename = "scriptToEvaluateOnLoad",
        skip_serializing_if = "Option::is_none"
    )]
    pub script_to_evaluate_on_load: Option<&'a str>,
    ///There is no frameId parameter in CDP docs, it's strange and should be fixed
    #[serde(rename = "frameId", skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<&'a FrameId>,
    #[serde(rename = "loaderId", skip_serializing_if = "Option::is_none")]
    pub loader_id: Option<&'a LoaderId>,
}

impl<'a> Reload<'a> {
    pub fn default() -> Self {
        Self {
            ignore_cache: None,
            script_to_evaluate_on_load: None,
            frame_id: None,
            loader_id: None,
        }
    }

    pub fn new() -> Self {
        Self {
            ignore_cache: None,
            script_to_evaluate_on_load: None,
            frame_id: None,
            loader_id: None,
        }
    }

    pub fn ignore_cache(mut self, value: bool) -> Self {
        self.ignore_cache = Some(value);
        self
    }

    pub fn script_to_evaluate_on_load(mut self, value: &'a str) -> Self {
        self.script_to_evaluate_on_load = Some(value);
        self
    }

    pub fn frame_id(mut self, value: &'a FrameId) -> Self {
        self.frame_id = Some(value);
        self
    }

    pub fn loader_id(mut self, value: &'a LoaderId) -> Self {
        self.loader_id = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct RemoveScriptToEvaluateOnNewDocument<'a> {
    #[serde(rename = "identifier")]
    pub identifier: &'a str,
}

impl<'a> RemoveScriptToEvaluateOnNewDocument<'a> {
    pub fn default(identifier: &'a str) -> Self {
        Self { identifier }
    }
}

#[derive(Serialize)]
pub struct SetBypassCSP {
    pub enabled: bool,
}

impl SetBypassCSP {
    pub fn default() -> Self {
        Self { enabled: true }
    }

    pub fn build(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[derive(Serialize)]
pub struct SetDocumentContent<'a> {
    pub frame_id: &'a FrameId,
    pub html: &'a str,
}

impl<'a> SetDocumentContent<'a> {
    pub fn default(frame_id: &'a FrameId, html: &'a str) -> Self {
        Self { frame_id, html }
    }
}

#[derive(Serialize)]
pub struct SetInterceptFileChooserDialog {
    pub enabled: bool,
}

impl SetInterceptFileChooserDialog {
    pub fn default(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[derive(Serialize)]
pub struct SetLifecycleEventsEnabled {
    pub enabled: bool,
}

impl SetLifecycleEventsEnabled {
    pub fn default() -> Self {
        Self { enabled: true }
    }

    pub fn build(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[derive(Serialize)]
pub struct StopLoading<'a> {
    #[serde(rename = "frameId", skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<&'a FrameId>,
}

impl<'a> StopLoading<'a> {
    pub fn default() -> Self {
        Self { frame_id: None }
    }

    pub fn build(frame_id: &'a FrameId) -> Self {
        Self {
            frame_id: Some(frame_id),
        }
    }
}

#[derive(Serialize)]
pub struct SetWebLifecycleState<'a> {
    pub state: &'a str,
}

impl<'a> SetWebLifecycleState<'a> {
    pub fn default() -> Self {
        Self { state: "active" }
    }

    pub fn build(state: &'a str) -> Self {
        Self { state }
    }
}

#[derive(Serialize)]
pub struct GetNavigationHistory {}

impl GetNavigationHistory {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Deserialize)]
pub struct GetNavigationHistoryResponse {
    #[serde(rename = "currentIndex")]
    pub current_index: u64,
    #[serde(rename = "entries")]
    pub entries: Vec<NavigationEntry>,
}
