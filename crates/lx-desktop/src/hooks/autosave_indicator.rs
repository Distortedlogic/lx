use std::time::Duration;

use dioxus::prelude::*;

const SAVING_DELAY: Duration = Duration::from_millis(250);
const SAVED_LINGER: Duration = Duration::from_millis(1600);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutosaveState {
  Idle,
  Saving,
  Saved,
  Error,
}

#[derive(Clone, Copy)]
pub struct AutosaveIndicator {
  pub state: Signal<AutosaveState>,
  save_id: Signal<u64>,
}

impl AutosaveIndicator {
  pub fn state(&self) -> AutosaveState {
    (self.state)()
  }

  pub fn mark_dirty(&mut self) {
    self.save_id.set((self.save_id)() + 1);
  }

  pub fn reset(&mut self) {
    self.state.set(AutosaveState::Idle);
    self.save_id.set(0);
  }

  pub async fn run_save<F, Fut>(&mut self, save: F) -> Result<(), Box<dyn std::error::Error>>
  where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
  {
    self.save_id.set((self.save_id)() + 1);
    let current_id = (self.save_id)();

    let mut state = self.state;
    let save_id = self.save_id;

    spawn(async move {
      tokio::time::sleep(SAVING_DELAY).await;
      if (save_id)() == current_id {
        state.set(AutosaveState::Saving);
      }
    });

    let result = save().await;

    match result {
      Ok(()) => {
        self.state.set(AutosaveState::Saved);
        let mut state = self.state;
        let save_id = self.save_id;
        spawn(async move {
          tokio::time::sleep(SAVED_LINGER).await;
          if (save_id)() == current_id {
            state.set(AutosaveState::Idle);
          }
        });
        Ok(())
      },
      Err(e) => {
        self.state.set(AutosaveState::Error);
        Err(e)
      },
    }
  }
}

pub fn use_autosave_indicator() -> AutosaveIndicator {
  let state = use_signal(|| AutosaveState::Idle);
  let save_id = use_signal(|| 0u64);
  AutosaveIndicator { state, save_id }
}
