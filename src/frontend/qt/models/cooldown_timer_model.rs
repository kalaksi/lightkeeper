
extern crate qmetaobject;
use qmetaobject::*;
use std::collections::HashMap;



const COOLDOWN_LENGTH: u32 = 45000;

#[allow(non_snake_case)]
#[derive(QObject, Default)]
/// For command cooldown mechanism.
pub struct CooldownTimerModel {
    base: qt_base_class!(trait QObject),

    allFinished: qt_signal!(),

    // State has to be stored and handled here and not in CommandButton or CommandButtonRow since table content isn't persistent.
    startCooldown: qt_method!(fn(&mut self, button_identifier: QString, invocation_id: u64)),
    updateCooldowns: qt_method!(fn(&mut self, cooldown_decrement: u32) -> u32),
    finishCooldown: qt_method!(fn(&mut self, invocation_id: u64)),
    getCooldown: qt_method!(fn(&self, button_identifier: QString) -> f32),

    cooldown_times: HashMap<String, u32>,
    cooldowns_finishing: Vec<String>,
    invocation_buttons: HashMap<u64, String>,
}

#[allow(non_snake_case)]
impl CooldownTimerModel {
    fn startCooldown(&mut self, button_identifier: QString, invocation_id: u64) {
        let button_identifier = button_identifier.to_string();
        self.cooldown_times.insert(button_identifier.clone(), COOLDOWN_LENGTH);
        self.invocation_buttons.insert(invocation_id, button_identifier);
    }

    fn updateCooldowns(&mut self, cooldown_decrement: u32) -> u32 {
        for (button_identifier, cooldown_time) in self.cooldown_times.iter_mut() {
            // Quickly decrease cooldown if command is finished.
            let actual_decrement = match self.cooldowns_finishing.contains(button_identifier) {
                true => 30 * cooldown_decrement,
                false => cooldown_decrement,
            };

            if actual_decrement > *cooldown_time {
                *cooldown_time = 0;
                self.cooldowns_finishing.retain(|c| c != button_identifier);
            }
            else {
                *cooldown_time -= actual_decrement;
            };
        }

        self.cooldown_times.retain(|_, cooldown_time| *cooldown_time > 0);

        if self.cooldown_times.is_empty() {
            self.allFinished();
        }

        self.cooldown_times.len() as u32
    }

    fn finishCooldown(&mut self, invocation_id: u64) {
        // Does nothing if the invocation_id doesn't belong to this table instance.
        if let Some(button_identifier) = self.invocation_buttons.remove(&invocation_id) {
            self.cooldowns_finishing.push(button_identifier);
        }
    }

    fn getCooldown(&self, button_identifier: QString) -> f32 {
        let button_identifier = button_identifier.to_string();
        let cooldown_time = *self.cooldown_times.get(&button_identifier).unwrap_or(&0);
        cooldown_time as f32 / COOLDOWN_LENGTH as f32
    }
}