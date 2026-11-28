use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use pumpkin_util::text::TextComponent;
use tokio::sync::oneshot;

type Verdict = Result<(), TextComponent>;
const LOGIN_ADMISSION_TIMEOUT: Duration = Duration::from_secs(5);

/// A single-use verdict sender for a pending login.
pub struct PendingLoginAdmission(oneshot::Sender<Verdict>);

impl PendingLoginAdmission {
    /// Whether the connection still awaits this verdict.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        !self.0.is_closed()
    }

    /// Resolves this login attempt. Returns false if the connection stopped waiting.
    #[must_use]
    pub fn resolve(self, verdict: Verdict) -> bool {
        self.0.send(verdict).is_ok()
    }
}

#[derive(Default)]
enum State {
    #[default]
    Closed,
    Open,
    Deferred(oneshot::Receiver<Verdict>),
}

#[derive(Default)]
pub(crate) struct LoginAdmission(Mutex<State>);

impl LoginAdmission {
    pub(crate) fn open(&self) -> LoginAdmissionScope<'_> {
        *self.state() = State::Open;
        LoginAdmissionScope(self)
    }

    pub(crate) fn defer(&self) -> Option<PendingLoginAdmission> {
        let mut state = self.state();
        if !matches!(*state, State::Open) {
            return None;
        }
        let (sender, receiver) = oneshot::channel();
        *state = State::Deferred(receiver);
        Some(PendingLoginAdmission(sender))
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub(crate) struct LoginAdmissionScope<'a>(&'a LoginAdmission);

impl LoginAdmissionScope<'_> {
    pub(crate) fn finish(self) -> Option<LoginAdmissionWait> {
        match std::mem::take(&mut *self.0.state()) {
            State::Deferred(receiver) => Some(LoginAdmissionWait(receiver)),
            _ => None,
        }
    }
}

impl Drop for LoginAdmissionScope<'_> {
    fn drop(&mut self) {
        *self.0.state() = State::Closed;
    }
}

pub(crate) struct LoginAdmissionWait(oneshot::Receiver<Verdict>);

impl LoginAdmissionWait {
    pub(crate) async fn verdict(self) -> Verdict {
        match tokio::time::timeout(LOGIN_ADMISSION_TIMEOUT, self.0).await {
            Ok(Ok(verdict)) => verdict,
            Ok(Err(_)) | Err(_) => Err(TextComponent::text(
                "Unable to verify your login status. Please try again.",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pending_admission_is_independent_of_other_connections() {
        let first = LoginAdmission::default();
        let first_scope = first.open();
        let first_reply = first.defer().unwrap();
        assert!(first.defer().is_none());
        let mut first_verdict = std::pin::pin!(first_scope.finish().unwrap().verdict());
        assert!(futures::poll!(&mut first_verdict).is_pending());
        assert!(first.defer().is_none());

        let second = LoginAdmission::default();
        let second_scope = second.open();
        let second_reply = second.defer().unwrap();
        assert!(second_reply.resolve(Ok(())));
        assert!(second_scope.finish().unwrap().verdict().await.is_ok());

        assert!(first_reply.resolve(Err(TextComponent::text("denied"))));
        assert_eq!(first_verdict.await.unwrap_err().get_text(), "denied");
    }

    #[tokio::test(start_paused = true)]
    async fn unresolved_admission_times_out_and_rejects_late_verdicts() {
        let admission = LoginAdmission::default();
        let scope = admission.open();
        let reply = admission.defer().unwrap();
        assert!(scope.finish().unwrap().verdict().await.is_err());
        assert!(!reply.resolve(Ok(())));
    }

    #[tokio::test]
    async fn cancelled_or_abandoned_admission_cannot_allow_a_connection() {
        let admission = LoginAdmission::default();
        let scope = admission.open();
        let reply = admission.defer().unwrap();
        drop(scope);
        assert!(!reply.is_pending());
        assert!(!reply.resolve(Ok(())));

        let scope = admission.open();
        let reply = admission.defer().unwrap();
        let wait = scope.finish().unwrap();
        drop(reply);
        assert!(wait.verdict().await.is_err());
    }
}
