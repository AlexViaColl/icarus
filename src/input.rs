#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum KeyId {
    Esc,
    A,
    D,
    S,
    W,

    Down,
    Up,

    KeyCount,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Key {
    is_down: bool,
    half_transitions: u32,
}

#[derive(Debug, Default)]
pub struct InputState {
    pub keys: [Key; KeyId::KeyCount as usize],
}

impl InputState {
    pub fn reset_transitions(&mut self) {
        self.keys.iter_mut().for_each(|k| k.half_transitions = 0);
    }

    pub fn set_key(&mut self, key: KeyId, is_down: bool) {
        self.keys[key as usize].is_down = is_down;
        self.keys[key as usize].half_transitions += 1;
    }

    pub fn is_down(&self, key: KeyId) -> bool {
        self.keys[key as usize].is_down
    }

    pub fn was_pressed(&self, key: KeyId) -> bool {
        let key = self.keys[key as usize];
        key.is_down && key.half_transitions >= 1
    }

    pub fn was_released(&self, key: KeyId) -> bool {
        let key = self.keys[key as usize];
        !key.is_down && key.half_transitions >= 1
    }
}
