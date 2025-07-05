use anyhow::{Result, anyhow};
use dashmap::DashSet;
use serde::Serialize;
use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};
use tokio::sync::{Mutex, mpsc};

use super::connection::{Connection, Event, Response};
use super::domains::browser::BrowserContextID;
use super::domains::dom::*;
use super::domains::fetch::*;
use super::domains::network::*;
use super::domains::page::*;
use super::domains::runtime::*;
use super::domains::target::*;
use super::emulation_manager::EmulationManager;
use super::js_manager::JsManager;
use super::network_manager::NetworkManager;
use super::target_manager::TargetManager;

#[derive(Debug, Clone)]
pub struct Target {
    connection: Weak<Connection>,
    target_id: Arc<TargetId>,
    parent_id: Option<Arc<TargetId>>,
    target_type: String,
    browser_context_id: BrowserContextID,
    session_id: Arc<SessionId>,
    runtime_ref_count: Arc<AtomicI32>,
    network_manager: Option<Arc<NetworkManager>>,
    emulation_manager: Option<Arc<EmulationManager>>,
    js_manager: Option<Arc<JsManager>>,
    dom_lock: Arc<Mutex<()>>,
}

impl Target {
    pub fn new(
        connection: Weak<Connection>,
        target_id: Arc<TargetId>,
        parent_id: Option<Arc<TargetId>>,
        target_type: String,
        browser_context_id: BrowserContextID,
        session_id: Arc<SessionId>,
    ) -> Self {
        Self {
            connection,
            target_id,
            parent_id,
            target_type,
            browser_context_id,
            session_id,
            runtime_ref_count: Arc::new(AtomicI32::new(0)),
            network_manager: None,
            emulation_manager: None,
            js_manager: None,
            dom_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn connection(&self) -> Option<Arc<Connection>> {
        self.connection.upgrade()
    }

    pub fn target_manager(&self) -> Arc<TargetManager> {
        let connection = match self.connection() {
            Some(connection) => connection,
            None => panic!("Connection is not available"),
        };
        let target_manager = match connection.target_manager() {
            Some(target_manager) => target_manager.clone(),
            None => panic!("Target manager is not available"),
        };
        target_manager
    }

    pub fn session_id(&self) -> Arc<SessionId> {
        self.session_id.clone()
    }

    pub fn target_id(&self) -> Arc<TargetId> {
        self.target_id.clone()
    }

    pub fn parent_id(&self) -> &Option<Arc<TargetId>> {
        &self.parent_id
    }

    pub fn target_type(&self) -> &str {
        &self.target_type
    }

    pub fn browser_context_id(&self) -> &BrowserContextID {
        &self.browser_context_id
    }

    pub async fn send<P: Serialize>(&self, method: &str, params: &P) -> Result<Response> {
        let Some(conn) = self.connection() else {
            return Err(anyhow!("Connection is not available"));
        };

        match conn.send(method, params, Some(&self.session_id())).await {
            Ok(response) => Ok(response),
            Err(e) => Err(e),
        }
    }

    pub async fn subscribe(
        &self,
        methods: DashSet<String>,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Event>>> {
        let Some(conn) = self.connection() else {
            return Err(anyhow!("Connection is not available"));
        };

        let session_ids = DashSet::with_capacity(1);
        session_ids.insert(self.session_id());

        let (_, rx) = conn.subscribe(methods, session_ids).await;
        Ok(rx)
    }

    //Weaked parent to prevent some bugs with target (UB of CDP);
    pub async fn init(&mut self, parent_target: Option<Weak<Target>>) -> Result<()> {
        // match self
        //     .send(
        //         "Page.setLifecycleEventsEnabled",
        //         &SetLifecycleEventsEnabled::default(),
        //     )
        //     .await
        // {
        //     Ok(response) => response,
        //     Err(e) => {
        //         println!("Error on set lifecycle events enabled: {:?}", e);
        //         return Err(e);
        //     }
        // };
        // match self.send("Network.enable", &NetworkEnable::default()).await {
        //     Ok(response) => response,
        //     Err(e) => {
        //         println!("Error on enable network: {:?}", e);
        //         return Err(e);
        //     }
        // };
        // match self.send("DOM.enable", &DomEnable::default()).await {
        //     Ok(response) => response,
        //     Err(e) => {
        //         println!("Error on enable DOM: {:?}", e);
        //         return Err(e);
        //     }
        // };
        // match self.send("Fetch.enable", &FetchEnable::default()).await {
        //     Ok(response) => response,
        //     Err(e) => {
        //         println!("Error on enable Fetch: {:?}", e);
        //         return Err(e);
        //     }
        // };

        let set_lifecycle_events_enabled = SetLifecycleEventsEnabled::default();
        let network_enable = NetworkEnable::default();
        let dom_enable = DomEnable::default();
        let fetch_enable = FetchEnable::default();

        let (page_lifecycle_events_enabled, network_enabled, dom_enabled, fetch_enabled) = tokio::join!(
            self.send(
                "Page.setLifecycleEventsEnabled",
                &set_lifecycle_events_enabled
            ),
            self.send("Network.enable", &network_enable),
            self.send("DOM.enable", &dom_enable),
            self.send("Fetch.enable", &fetch_enable),
        );

        match page_lifecycle_events_enabled {
            Ok(response) => response,
            Err(e) => {
                println!("Error on set lifecycle events enabled: {:?}", e);
                return Err(e);
            }
        };

        match network_enabled {
            Ok(response) => response,
            Err(e) => {
                println!("Error on enable network: {:?}", e);
                return Err(e);
            }
        };

        match dom_enabled {
            Ok(response) => response,
            Err(e) => {
                println!("Error on enable DOM: {:?}", e);
                return Err(e);
            }
        };

        match fetch_enabled {
            Ok(response) => response,
            Err(e) => {
                println!("Error on enable Fetch: {:?}", e);
                return Err(e);
            }
        };

        if let Some(parent) = parent_target {
            let parent_target = match parent.upgrade() {
                Some(parent_target) => parent_target,
                None => return Err(anyhow!("Parent target is not available")),
            };

            self.network_manager = parent_target.network_manager();
            self.emulation_manager = parent_target.emulation_manager();
            self.js_manager = parent_target.js_manager();

            let (network_result, emulation_result, js_result) = tokio::join!(
                async {
                    if let Some(network_manager) = self.network_manager.as_ref() {
                        network_manager.add_session(self.session_id()).await
                    } else {
                        Err(anyhow!(
                            "Network manager is not available, probably parent target is not initialized or destroyed"
                        ))
                    }
                },
                async {
                    if let Some(emulation_manager) = self.emulation_manager.as_ref() {
                        emulation_manager.add_session(self.session_id()).await
                    } else {
                        Err(anyhow!(
                            "Emulation manager is not available, probably parent target is not initialized or destroyed"
                        ))
                    }
                },
                async {
                    if let Some(js_manager) = self.js_manager.as_ref() {
                        js_manager.add_session(self.session_id()).await
                    } else {
                        Err(anyhow!(
                            "Js manager is not available, probably parent target is not initialized or destroyed"
                        ))
                    }
                }
            );

            network_result?;
            emulation_result?;
            js_result?;
        } else {
            let connection = match self.connection() {
                Some(connection) => connection,
                None => return Err(anyhow!("Connection is not available")),
            };
            let network_manager = NetworkManager::new(Arc::downgrade(&connection));
            let emulation_manager = EmulationManager::new(Arc::downgrade(&connection));
            let js_manager = JsManager::new(Arc::downgrade(&connection));

            let (network_manager_init, js_manager_init) =
                tokio::join!(network_manager.clone().init(), js_manager.clone().init(),);

            network_manager_init?;
            js_manager_init?;

            // network_manager.clone().init().await?;
            // network_manager.add_session(&self.session_id()).await?;
            // emulation_manager.add_session(&self.session_id()).await?;
            // js_manager.clone().init().await?;
            // js_manager.add_session(&self.session_id()).await?;

            let (
                network_manager_add_session,
                emulation_manager_add_session,
                js_manager_add_session,
            ) = tokio::join!(
                network_manager.add_session(self.session_id()),
                emulation_manager.add_session(self.session_id()),
                js_manager.add_session(self.session_id()),
            );

            network_manager_add_session?;
            emulation_manager_add_session?;
            js_manager_add_session?;

            self.network_manager = Some(network_manager);
            self.emulation_manager = Some(emulation_manager);
            self.js_manager = Some(js_manager);
        }

        Ok(())
    }

    pub fn network_manager(&self) -> Option<Arc<NetworkManager>> {
        self.network_manager.clone()
    }

    pub fn emulation_manager(&self) -> Option<Arc<EmulationManager>> {
        self.emulation_manager.clone()
    }

    pub fn js_manager(&self) -> Option<Arc<JsManager>> {
        self.js_manager.clone()
    }

    pub fn dom_lock(&self) -> Arc<Mutex<()>> {
        self.dom_lock.clone()
    }

    pub async fn enable_runtime(&self) {
        let old_count = self.runtime_ref_count.fetch_add(1, Ordering::SeqCst);

        if old_count == 0 {
            let _ = self.send("Runtime.enable", &RuntimeEnable::default()).await;
        }
    }

    pub async fn disable_runtime(&self) {
        let old_count = self.runtime_ref_count.load(Ordering::SeqCst);

        if old_count == 0 {
            return;
        }

        let old_count = self.runtime_ref_count.fetch_sub(1, Ordering::SeqCst);

        if old_count > 0 && old_count == 1 {
            let _ = self
                .send("Runtime.disable", &RuntimeDisable::default())
                .await;
        }
    }

    pub async fn shutdown(self) -> Result<()> {
        let (network_manager, emulation_manager, js_manager) = (
            self.network_manager(),
            self.emulation_manager(),
            self.js_manager(),
        );

        let target_type = self.target_type();

        tokio::join!(
            async {
                if let Some(network_manager) = network_manager {
                    let _ = network_manager.remove_session(&self.session_id()).await;
                    if target_type != "iframe" {
                        network_manager.shutdown().await;
                    }
                }
            },
            async {
                if let Some(emulation_manager) = emulation_manager {
                    let _ = emulation_manager.remove_session(&self.session_id()).await;
                }
            },
            async {
                if let Some(js_manager) = js_manager {
                    let _ = js_manager.remove_session(&self.session_id()).await;
                    if target_type != "iframe" {
                        js_manager.shutdown().await;
                    }
                }
            }
        );

        // if let Some(network_manager) = network_manager {
        //     let _ = network_manager.remove_session(&self.session_id()).await;
        //     if target_type != "iframe" {
        //         network_manager.shutdown().await;
        //     }
        // }
        // if let Some(emulation_manager) = emulation_manager {
        //     let _ = emulation_manager.remove_session(&self.session_id()).await;
        // }
        // if let Some(js_manager) = js_manager {
        //     let _ = js_manager.remove_session(&self.session_id()).await;
        //     if target_type != "iframe" {
        //         js_manager.shutdown().await;
        //     }
        // }

        Ok(())
    }
}
