use super::page::FrameId;
use super::runtime::{ExecutionContextId, RemoteObject, RemoteObjectId};
use serde::{Deserialize, Serialize};
pub type BackendNodeId = u64;
pub type NodeId = u64;
pub type Quad = [f32; 8];
pub type ShadowRootType = String;
pub type PseudoType = String;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

//This struct wins for not alloc name and value, if we don't need them. To be honest, only case we need them is text finder.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MinimalNode {
    ///Actually for some reason NodeId is not existing in the response for every node, especially for the shadow roots. Guess it's a conflict of General root and Shadow root with nodes.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub node_id: Option<NodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<NodeId>,
    pub backend_node_id: BackendNodeId,
    pub node_type: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MinimalNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<FrameId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow_roots: Option<Vec<MinimalNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pseudo_elements: Option<Vec<MinimalNode>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FullNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<NodeId>,
    pub backend_node_id: BackendNodeId,
    pub node_type: i32,
    pub local_name: String,
    pub node_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FullNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<FrameId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow_roots: Option<Vec<FullNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pseudo_elements: Option<Vec<FullNode>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoxModel {
    pub content: Quad,
    pub padding: Quad,
    pub border: Quad,
    pub margin: Quad,
    pub width: i32,
    pub height: i32,
}

///Events
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
//For our cases, we don't need anything here. We will use it just for lazy quering of elements.
pub struct AttributeModified {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttributeRemoved {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterDataModified {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChildNodeInserted {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChildNodeRemoved {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetChildNodes {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DistributedNodesUpdated {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InlineStyleInvalidated {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PseudoElementAdded {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PseudoElementRemoved {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShadowRootPushed {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShadowRootPopped {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentUpdated {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopLayerElementsUpdated {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScrollableFlagUpdated {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChildNodeCountUpdated {}

///Deserialized responses
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DescribeNodeResponse {
    pub node: MinimalNode,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DescribeNodeResponseFull {
    pub node: FullNode,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetAttributesResponse {
    pub attributes: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetBoxModelResponse {
    pub model: BoxModel,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetDocumentResponse {
    pub root: MinimalNode,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetDocumentResponseFull {
    pub root: FullNode,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetFrameOwnerResponse {
    pub backend_node_id: BackendNodeId,
    pub node_id: Option<NodeId>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuerySelectorResponse {
    pub node_id: NodeId,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuerySelectorAllResponse {
    pub node_ids: Vec<NodeId>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeForLocationResponse {
    pub backend_node_id: BackendNodeId,
    pub frame_id: FrameId,
    pub node_id: Option<NodeId>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolveNodeResponse {
    pub object: RemoteObject,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetOuterHTMLResponse {
    #[serde(rename = "outerHTML")]
    pub outer_html: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PushNodesByBackendIdsToFrontendResponse {
    #[serde(rename = "nodeIds")]
    pub node_ids: Vec<NodeId>,
}

///Serialized requests
#[derive(Serialize)]
pub struct DomEnable {}

impl DomEnable {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct DomDisable {}

impl DomDisable {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct DescribeNode<'a> {
    #[serde(rename = "nodeId", skip_serializing_if = "Option::is_none")]
    pub node_id: Option<&'a NodeId>,
    #[serde(rename = "backendNodeId", skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<&'a BackendNodeId>,
    #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<&'a RemoteObjectId>,
    #[serde(rename = "depth", skip_serializing_if = "Option::is_none")]
    pub depth: Option<i32>,
    #[serde(rename = "pierce", skip_serializing_if = "Option::is_none")]
    pub pierce: Option<bool>,
}

impl<'a> DescribeNode<'a> {
    pub fn default(backend_node_id: &'a BackendNodeId) -> Self {
        Self {
            node_id: None,
            backend_node_id: Some(backend_node_id),
            object_id: None,
            depth: Some(-1),
            pierce: Some(true),
        }
    }

    pub fn new() -> Self {
        Self {
            node_id: None,
            backend_node_id: None,
            object_id: None,
            depth: Some(-1),
            pierce: Some(true),
        }
    }

    pub fn node_id(mut self, node_id: &'a NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn backend_node_id(mut self, backend_node_id: &'a BackendNodeId) -> Self {
        self.backend_node_id = Some(backend_node_id);
        self
    }

    pub fn object_id(mut self, object_id: &'a RemoteObjectId) -> Self {
        self.object_id = Some(object_id);
        self
    }

    pub fn depth(mut self, depth: i32) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn pierce(mut self, pierce: bool) -> Self {
        self.pierce = Some(pierce);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct Focus<'a> {
    #[serde(rename = "nodeId", skip_serializing_if = "Option::is_none")]
    pub node_id: Option<&'a NodeId>,
    #[serde(rename = "backendNodeId", skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<&'a BackendNodeId>,
    #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<&'a RemoteObjectId>,
}

impl<'a> Focus<'a> {
    pub fn default(backend_node_id: &'a BackendNodeId) -> Self {
        Self {
            node_id: None,
            backend_node_id: Some(backend_node_id),
            object_id: None,
        }
    }

    pub fn new() -> Self {
        Self {
            node_id: None,
            backend_node_id: None,
            object_id: None,
        }
    }

    pub fn node_id(mut self, node_id: &'a NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn backend_node_id(mut self, backend_node_id: &'a BackendNodeId) -> Self {
        self.backend_node_id = Some(backend_node_id);
        self
    }

    pub fn object_id(mut self, object_id: &'a RemoteObjectId) -> Self {
        self.object_id = Some(object_id);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct GetAttributes<'a> {
    #[serde(rename = "nodeId")]
    pub node_id: &'a NodeId,
}

impl<'a> GetAttributes<'a> {
    pub fn default(node_id: &'a NodeId) -> Self {
        Self { node_id }
    }
}

#[derive(Serialize)]
pub struct GetBoxModel<'a> {
    #[serde(rename = "nodeId", skip_serializing_if = "Option::is_none")]
    pub node_id: Option<&'a NodeId>,
    #[serde(rename = "backendNodeId", skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<&'a BackendNodeId>,
    #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<&'a RemoteObjectId>,
}

impl<'a> GetBoxModel<'a> {
    pub fn default(backend_node_id: &'a BackendNodeId) -> Self {
        Self {
            node_id: None,
            backend_node_id: Some(backend_node_id),
            object_id: None,
        }
    }

    pub fn new() -> Self {
        Self {
            node_id: None,
            backend_node_id: None,
            object_id: None,
        }
    }

    pub fn node_id(mut self, node_id: &'a NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn backend_node_id(mut self, backend_node_id: &'a BackendNodeId) -> Self {
        self.backend_node_id = Some(backend_node_id);
        self
    }

    pub fn object_id(mut self, object_id: &'a RemoteObjectId) -> Self {
        self.object_id = Some(object_id);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct GetDocument {
    #[serde(rename = "depth", skip_serializing_if = "Option::is_none")]
    pub depth: Option<i32>,
    #[serde(rename = "pierce", skip_serializing_if = "Option::is_none")]
    pub pierce: Option<bool>,
}

impl GetDocument {
    pub fn default() -> Self {
        Self {
            depth: Some(-1),
            pierce: Some(true),
        }
    }

    pub fn new() -> Self {
        Self {
            depth: None,
            pierce: Some(true),
        }
    }

    pub fn depth(mut self, depth: i32) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn pierce(mut self, pierce: bool) -> Self {
        self.pierce = Some(pierce);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct GetNodeForLocation {
    #[serde(rename = "x", skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(rename = "y", skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(
        rename = "includeUserAgentShadowDOM",
        skip_serializing_if = "Option::is_none"
    )]
    pub include_user_agent_shadow_dom: Option<bool>,
    #[serde(
        rename = "ignorePointerEventsNone",
        skip_serializing_if = "Option::is_none"
    )]
    pub ignore_pointer_events_none: Option<bool>,
}

impl<'a> GetNodeForLocation {
    pub fn default(x: i32, y: i32) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
            include_user_agent_shadow_dom: Some(true),
            ignore_pointer_events_none: Some(true),
        }
    }

    pub fn new() -> Self {
        Self {
            x: None,
            y: None,
            include_user_agent_shadow_dom: None,
            ignore_pointer_events_none: None,
        }
    }

    pub fn x(mut self, x: i32) -> Self {
        self.x = Some(x);
        self
    }

    pub fn y(mut self, y: i32) -> Self {
        self.y = Some(y);
        self
    }

    pub fn include_user_agent_shadow_dom(mut self, include_user_agent_shadow_dom: bool) -> Self {
        self.include_user_agent_shadow_dom = Some(include_user_agent_shadow_dom);
        self
    }

    pub fn ignore_pointer_events_none(mut self, ignore_pointer_events_none: bool) -> Self {
        self.ignore_pointer_events_none = Some(ignore_pointer_events_none);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct GetOuterHTML<'a> {
    #[serde(rename = "nodeId", skip_serializing_if = "Option::is_none")]
    pub node_id: Option<&'a NodeId>,
    #[serde(rename = "backendNodeId", skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<&'a BackendNodeId>,
    #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<&'a RemoteObjectId>,
}

impl<'a> GetOuterHTML<'a> {
    pub fn default(backend_node_id: &'a BackendNodeId) -> Self {
        Self {
            node_id: None,
            backend_node_id: Some(backend_node_id),
            object_id: None,
        }
    }

    pub fn new() -> Self {
        Self {
            node_id: None,
            backend_node_id: None,
            object_id: None,
        }
    }

    pub fn node_id(mut self, node_id: &'a NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn backend_node_id(mut self, backend_node_id: &'a BackendNodeId) -> Self {
        self.backend_node_id = Some(backend_node_id);
        self
    }

    pub fn object_id(mut self, object_id: &'a RemoteObjectId) -> Self {
        self.object_id = Some(object_id);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct QuerySelector<'a> {
    #[serde(rename = "selector")]
    pub selector: &'a str,
    #[serde(rename = "nodeId")]
    pub node_id: &'a NodeId,
}

impl<'a> QuerySelector<'a> {
    pub fn default(selector: &'a str, node_id: &'a NodeId) -> Self {
        Self { selector, node_id }
    }
}

#[derive(Serialize)]
pub struct QuerySelectorAll<'a> {
    #[serde(rename = "selector")]
    pub selector: &'a str,
    #[serde(rename = "nodeId")]
    pub node_id: &'a NodeId,
}

impl<'a> QuerySelectorAll<'a> {
    pub fn default(selector: &'a str, node_id: &'a NodeId) -> Self {
        Self { selector, node_id }
    }
}

#[derive(Serialize)]
pub struct RequestNode<'a> {
    #[serde(rename = "objectId")]
    pub object_id: &'a RemoteObjectId,
}

impl<'a> RequestNode<'a> {
    pub fn default(object_id: &'a RemoteObjectId) -> Self {
        Self { object_id }
    }
}

#[derive(Serialize)]
pub struct ResolveNode<'a> {
    #[serde(rename = "nodeId", skip_serializing_if = "Option::is_none")]
    pub node_id: Option<&'a NodeId>,
    #[serde(rename = "backendNodeId", skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<&'a BackendNodeId>,
    #[serde(rename = "objectGroup", skip_serializing_if = "Option::is_none")]
    pub object_group: Option<&'a str>,
    #[serde(rename = "executionContextId", skip_serializing_if = "Option::is_none")]
    pub execution_context_id: Option<&'a ExecutionContextId>,
}

impl<'a> ResolveNode<'a> {
    pub fn default(backend_node_id: &'a BackendNodeId) -> Self {
        Self {
            node_id: None,
            backend_node_id: Some(backend_node_id),
            object_group: None,
            execution_context_id: None,
        }
    }

    pub fn new() -> Self {
        Self {
            node_id: None,
            backend_node_id: None,
            object_group: None,
            execution_context_id: None,
        }
    }

    pub fn node_id(mut self, node_id: &'a NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn backend_node_id(mut self, backend_node_id: &'a BackendNodeId) -> Self {
        self.backend_node_id = Some(backend_node_id);
        self
    }

    pub fn object_group(mut self, object_group: &'a str) -> Self {
        self.object_group = Some(object_group);
        self
    }

    pub fn execution_context_id(mut self, execution_context_id: &'a ExecutionContextId) -> Self {
        self.execution_context_id = Some(execution_context_id);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct ScrollIntoViewIfNeeded<'a> {
    #[serde(rename = "nodeId", skip_serializing_if = "Option::is_none")]
    pub node_id: Option<&'a NodeId>,
    #[serde(rename = "backendNodeId", skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<&'a BackendNodeId>,
    #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<&'a RemoteObjectId>,
    #[serde(rename = "rect", skip_serializing_if = "Option::is_none")]
    pub rect: Option<&'a Rect>,
}

impl<'a> ScrollIntoViewIfNeeded<'a> {
    pub fn default(backend_node_id: &'a BackendNodeId) -> Self {
        Self {
            node_id: None,
            backend_node_id: Some(backend_node_id),
            object_id: None,
            rect: None,
        }
    }

    pub fn new() -> Self {
        Self {
            node_id: None,
            backend_node_id: None,
            object_id: None,
            rect: None,
        }
    }

    pub fn node_id(mut self, node_id: &'a NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn backend_node_id(mut self, backend_node_id: &'a BackendNodeId) -> Self {
        self.backend_node_id = Some(backend_node_id);
        self
    }

    pub fn object_id(mut self, object_id: &'a RemoteObjectId) -> Self {
        self.object_id = Some(object_id);
        self
    }

    pub fn rect(mut self, rect: &'a Rect) -> Self {
        self.rect = Some(rect);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct SetAttributeValue<'a> {
    #[serde(rename = "nodeId")]
    pub node_id: &'a NodeId,
    #[serde(rename = "name")]
    pub name: &'a str,
    #[serde(rename = "value")]
    pub value: &'a str,
}

impl<'a> SetAttributeValue<'a> {
    pub fn default(node_id: &'a NodeId, name: &'a str, value: &'a str) -> Self {
        Self {
            node_id,
            name,
            value,
        }
    }
}

#[derive(Serialize)]
pub struct SetFileInputFiles {
    #[serde(rename = "files")]
    pub files: Option<Vec<String>>,
    #[serde(rename = "nodeId", skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(rename = "backendNodeId", skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<BackendNodeId>,
    #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<RemoteObjectId>,
}

impl SetFileInputFiles {
    pub fn default(files: Vec<String>, backend_node_id: BackendNodeId) -> Self {
        Self {
            files: Some(files),
            node_id: None,
            backend_node_id: Some(backend_node_id),
            object_id: None,
        }
    }

    pub fn new() -> Self {
        Self {
            files: None,
            node_id: None,
            backend_node_id: None,
            object_id: None,
        }
    }

    pub fn files(mut self, files: Vec<String>) -> Self {
        self.files = Some(files);
        self
    }

    pub fn node_id(mut self, node_id: NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn backend_node_id(mut self, backend_node_id: BackendNodeId) -> Self {
        self.backend_node_id = Some(backend_node_id);
        self
    }

    pub fn object_id(mut self, object_id: RemoteObjectId) -> Self {
        self.object_id = Some(object_id);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct PushNodesByBackendIdsToFrontend {
    #[serde(rename = "backendNodeIds")]
    pub backend_node_ids: Vec<BackendNodeId>,
}

impl PushNodesByBackendIdsToFrontend {
    pub fn default(backend_node_ids: Vec<BackendNodeId>) -> Self {
        Self { backend_node_ids }
    }
}

#[derive(Serialize)]
pub struct GetFrameOwner<'a> {
    #[serde(rename = "frameId")]
    pub frame_id: &'a FrameId,
}

impl<'a> GetFrameOwner<'a> {
    pub fn default(frame_id: &'a FrameId) -> Self {
        Self { frame_id }
    }
}

#[derive(Serialize)]
pub struct PerformSearch<'a> {
    #[serde(rename = "query")]
    pub query: &'a str,
    #[serde(
        rename = "includeFocusOutline",
        skip_serializing_if = "Option::is_none"
    )]
    pub include_focus_outline: Option<bool>,
}

impl<'a> PerformSearch<'a> {
    pub fn default(query: &'a str) -> Self {
        Self {
            query,
            include_focus_outline: Some(true),
        }
    }
}

#[derive(Deserialize)]
pub struct PerformSearchResponse {
    #[serde(rename = "searchId")]
    pub search_id: String,
    #[serde(rename = "resultCount")]
    pub result_count: i32,
}

#[derive(Serialize)]
pub struct GetSearchResults<'a> {
    #[serde(rename = "searchId")]
    pub search_id: &'a str,
    #[serde(rename = "fromIndex")]
    pub from_index: i32,
    #[serde(rename = "toIndex")]
    pub to_index: i32,
}

impl<'a> GetSearchResults<'a> {
    pub fn default(search_id: &'a str, to_index: i32) -> Self {
        Self {
            search_id,
            from_index: 0,
            to_index,
        }
    }
}

#[derive(Deserialize)]
pub struct GetSearchResultsResponse {
    #[serde(rename = "nodeIds")]
    pub node_ids: Vec<NodeId>,
}
