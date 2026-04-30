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

use super::{main_loop::Task, pool::Pool};
use lsp_server::Connection;
use std::{
    num::NonZeroUsize,
    panic::{RefUnwindSafe, UnwindSafe},
};

pub type ReqHandler<Db> = fn(&mut Session<Db>, lsp_server::Response);
type ReqQueue<Db> = lsp_server::ReqQueue<String, ReqHandler<Db>>;

/// Callback for unhandled errors from `with_db` handlers.
///
/// Called before the default error handling (error response for requests, `window/showMessage` for notifications).
pub type ErrorCallback = fn(&anyhow::Error);

/// Main session object that holds both lsp server connection and initialization options.
pub struct Session<Db: salsa::Database> {
    pub(crate) task_receiver: crossbeam_channel::Receiver<Task>,
    pub(crate) task_sender: crossbeam_channel::Sender<Task>,
    pub task_pool: Pool,
    /// Request queue for incoming requests
    pub req_queue: ReqQueue<Db>,
    pub connection: Connection,
    pub db: Db,
}

impl<Db: salsa::Database> Session<Db> {
    pub fn new(connection: Connection, db: Db) -> Self {
        let (task_sender, task_receiver) = crossbeam_channel::unbounded();

        let max_threads = std::thread::available_parallelism()
            .unwrap_or_else(|_| NonZeroUsize::new(1).unwrap())
            .get();

        log::info!("Max threads: {max_threads}");

        Self {
            connection,
            req_queue: ReqQueue::default(),
            db,
            task_receiver,
            task_sender,
            task_pool: Pool::new(max_threads),
        }
    }
}

/// Perform an operation on a snapshot of the database that may be cancelled.
///
/// From: <https://github.com/rust-lang/rust-analyzer/blob/4e4aee41c969e86adefdb8c687e2e91bb101329a/crates/ide/src/lib.rs#L862>
impl<Db: salsa::Database + RefUnwindSafe> Session<Db> {
    pub fn with_db<F, T>(&self, f: F) -> Result<T, salsa::Cancelled>
    where
        F: FnOnce(&Db) -> T + UnwindSafe,
    {
        let db = &self.db;
        salsa::Cancelled::catch(|| f(db))
    }
}
