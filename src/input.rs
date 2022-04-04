#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum KeyId {
    Any = 0,

    Esc,
    A,
    D,
    P,
    S,
    W,

    Down,
    Up,
    Left,
    Right,

    KeyCount,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct KeyState {
    pub is_down: bool,
    pub half_transitions: u32,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ButtonId {
    Any = 0,
    Left,
    Right,
    Middle,

    ButtonCount,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ButtonState {
    pub is_down: bool,
    pub x: i32,
    pub y: i32,
    pub half_transitions: u32,
}

#[derive(Debug, Default)]
pub struct InputState {
    pub keys: [KeyState; KeyId::KeyCount as usize],
    pub buttons: [ButtonState; ButtonId::ButtonCount as usize],
}

impl InputState {
    pub fn reset_transitions(&mut self) {
        self.keys.iter_mut().for_each(|k| k.half_transitions = 0);
        self.buttons.iter_mut().for_each(|b| b.half_transitions = 0);
    }

    pub fn set_key(&mut self, key: KeyId, is_down: bool) {
        self.keys[key as usize].is_down = is_down;
        self.keys[key as usize].half_transitions += 1;
    }

    pub fn set_button(&mut self, button: ButtonId, is_down: bool, x: i32, y: i32) {
        self.buttons[button as usize].is_down = is_down;
        self.buttons[button as usize].x = x;
        self.buttons[button as usize].y = y;
        self.buttons[button as usize].half_transitions += 1;
    }

    pub fn is_key_down(&self, id: KeyId) -> bool {
        self.keys[id as usize].is_down
    }

    pub fn was_key_pressed(&self, id: KeyId) -> bool {
        let key = self.keys[id as usize];
        key.is_down && key.half_transitions >= 1
    }

    pub fn was_key_released(&self, id: KeyId) -> bool {
        let key = self.keys[id as usize];
        !key.is_down && key.half_transitions >= 1
    }

    pub fn is_button_down(&self, id: ButtonId) -> bool {
        self.buttons[id as usize].is_down
    }

    pub fn was_button_pressed(&self, id: ButtonId) -> bool {
        let button = self.buttons[id as usize];
        button.is_down && button.half_transitions >= 1
    }

    pub fn was_button_released(&self, id: ButtonId) -> bool {
        let button = self.buttons[id as usize];
        !button.is_down && button.half_transitions >= 1
    }
}
