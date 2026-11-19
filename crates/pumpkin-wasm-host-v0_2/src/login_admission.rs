use pumpkin_core::server::login_admission::PendingLoginAdmission;
use pumpkin_wasm_host_common::state::{FromResource, PluginHostState};
use wasmtime::component::Resource;

use crate::pumpkin::plugin::{
    login_admission::{Host, HostPendingAdmission, PendingAdmission},
    player::Player,
    text::TextComponent,
};

impl FromResource for PendingAdmission {
    type Internal = PendingLoginAdmission;
}

impl Host for PluginHostState {
    fn defer(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<Resource<PendingAdmission>>> {
        self.get(&player)?
            .defer_login_admission()
            .map(|admission| self.add(admission))
            .transpose()
    }

    fn resolve(
        &mut self,
        admission: Resource<PendingAdmission>,
        verdict: Result<(), Resource<TextComponent>>,
    ) -> wasmtime::Result<bool> {
        let admission = self.take(admission)?;
        let verdict = match verdict {
            Ok(()) => Ok(()),
            Err(message) => Err(self.take(message)?),
        };
        Ok(admission.resolve(verdict))
    }
}

impl HostPendingAdmission for PluginHostState {
    fn is_pending(&mut self, admission: Resource<PendingAdmission>) -> wasmtime::Result<bool> {
        Ok(self.get(&admission)?.is_pending())
    }

    fn drop(&mut self, admission: Resource<PendingAdmission>) -> wasmtime::Result<()> {
        self.drop(admission)
    }
}
