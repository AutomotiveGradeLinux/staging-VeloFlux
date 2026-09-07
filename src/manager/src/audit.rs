// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub(crate) struct ResourceMutationLog {
    kind: &'static str,
    action: &'static str,
    name: String,
    revision: Option<u64>,
}

impl ResourceMutationLog {
    pub(crate) fn new(
        kind: &'static str,
        action: &'static str,
        name: impl Into<String>,
        revision: Option<u64>,
    ) -> Self {
        Self {
            kind,
            action,
            name: name.into(),
            revision,
        }
    }

    pub(crate) fn set_revision(&mut self, revision: Option<u64>) {
        self.revision = revision;
    }

    pub(crate) fn log_success(&self) {
        let Some(revision) = self.revision else {
            return;
        };
        tracing::info!(
            kind = self.kind,
            name = %self.name,
            action = self.action,
            revision,
            "rest api audit"
        );
    }

    pub(crate) fn log_failure(&self, _error: &(impl std::fmt::Display + ?Sized)) {
        let Some(revision) = self.revision else {
            return;
        };
        tracing::error!(
            kind = self.kind,
            name = %self.name,
            action = self.action,
            revision,
            "rest api audit failed"
        );
    }
}
