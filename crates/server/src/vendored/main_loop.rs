/*
This file is part of auto-lsp.
Copyright (C) 2025 CLAUZEL Adrien

auto-lsp is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>
*/

use std::{
    panic::RefUnwindSafe,
    sync::atomic::{AtomicI32, Ordering},
};

use crate::vendored::intent::ThreadIntent;

use super::{
    notification_registry::NotificationRegistry, request_registry::RequestRegistry,
    session::Session,
};

use anyhow::Error;
use crossbeam_channel::select;
use db::Url;
use lsp_server::Message;
use lsp_types::{Diagnostic, PublishDiagnosticsParams, notification::PublishDiagnostics};

#[derive(Debug)]
pub enum Task {
    Response(lsp_server::Response),
    Notification(lsp_server::Notification),
    NotificationError(Error),
}

impl<Db: salsa::Database + Clone + Send + RefUnwindSafe> Session<Db> {
    /// Main loop of the LSP server, backed by [`lsp-server`] and [`crossbeam-channel`] crates.
    pub fn main_loop(
        mut self,
        req_registry: &RequestRegistry<Db>,
        not_registry: &NotificationRegistry<Db>,
    ) -> anyhow::Result<()> {
        loop {
            select! {
                recv(self.connection.receiver) -> msg => {
                    match msg? {
                        Message::Request(req) => {
                            if self.connection.handle_shutdown(&req)? {
                                return Ok(());
                            }

                            self.req_queue.incoming.register(req.id.clone(), req.method.clone());

                            if let Some(method) = req_registry.get(&req) {
                                RequestRegistry::exec(&self, method, req);
                            } else if let Some(method) = req_registry.get_sync_mut(&req) {
                                RequestRegistry::exec_sync_mut(&mut self, method, req)?;
                            } else {
                                RequestRegistry::complete(&mut self,
                                    RequestRegistry::<Db>::request_mismatch(req.id.clone(), anyhow::format_err!("Unknown request: {}", req.method))
                                )?;
                            }
                        }
                        Message::Notification(not) => {
                            if let Some(method) = not_registry.get(&not) {
                                NotificationRegistry::exec(&self, method, not);
                            } else if let Some(method) = not_registry.get_sync_mut(&not) {
                                NotificationRegistry::exec_sync_mut(&mut self, method, not)?;
                            } else {
                                NotificationRegistry::handle_error(&self, anyhow::format_err!("Unknown notification: {}", not.method))?;
                            }
                        }
                        Message::Response(_) => {}
                    }
                },
                recv(self.task_receiver) -> task => {
                    match task? {
                        Task::Response(resp) => RequestRegistry::complete(&mut self, resp)?,
                        Task::Notification(not) => self.connection.sender.send(not.into())?,
                        Task::NotificationError(err) => NotificationRegistry::handle_error(&self, err)?,
                    }
                }
            }
        }
    }

    /// Send a notification to the client.
    pub fn send_notification<N: lsp_types::notification::Notification>(
        &self,
        params: &N::Params,
    ) -> anyhow::Result<()> {
        let params = serde_json::to_value(params)?;
        let n = lsp_server::Notification {
            method: N::METHOD.into(),
            params,
        };
        self.connection.sender.send(Message::Notification(n))?;
        Ok(())
    }

    pub fn schedule_diagnostics(
        &self,
        uri: Url,
        callback: fn(&Db, &Url) -> anyhow::Result<Vec<Diagnostic>>,
    ) {
        let db = self.db.clone();
        let sender = self.task_sender.clone();
        self.task_pool.spawn(
            ThreadIntent::Worker,
            std::panic::AssertUnwindSafe(move || {
                match salsa::Cancelled::catch(|| callback(&db, &uri)) {
                    Ok(Ok(diagnostics)) => sender
                        .send(Task::Notification(diagnostics_to_notification(
                            uri,
                            diagnostics,
                        )))
                        .unwrap(),
                    Ok(Err(e)) => {
                        sender.send(Task::NotificationError(e)).unwrap();
                    }
                    Err(e) => {
                        log::warn!("Cancelled diagnostics request: {e}");
                    }
                }
            }),
        );
    }

    pub fn clear_diagnostics(&self, uri: Url) {
        self.task_sender
            .send(Task::Notification(diagnostics_to_notification(
                uri,
                Vec::new(),
            )));
    }
}

fn diagnostics_to_notification(uri: Url, diagnostics: Vec<Diagnostic>) -> lsp_server::Notification {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };

    lsp_server::Notification::new(
        <PublishDiagnostics as lsp_types::notification::Notification>::METHOD.to_string(),
        params,
    )
}
