use std::collections::HashMap;
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Keys {
    RotateUp,
    RotateDown,
    RotateRight,
    RotateLeft,
    MoveUp,
    MoveDown,
    MoveRight,
    MoveLeft,
    MoveForward,
    MoveBackward,
}

pub struct Keyboard {
    key_map: HashMap<VirtualKeyCode, Keys>,
    pub active: Vec<Keys>,
}

// TODO: Transformar numa classe onde o usuário pode mudar o input da manéira que quiser

impl Default for Keyboard {
    fn default() -> Keyboard {
        let mut default_key_map: HashMap<VirtualKeyCode, Keys> = HashMap::new();
        default_key_map.insert(VirtualKeyCode::Up, Keys::RotateUp);
        default_key_map.insert(VirtualKeyCode::Down, Keys::RotateDown);
        default_key_map.insert(VirtualKeyCode::Right, Keys::RotateRight);
        default_key_map.insert(VirtualKeyCode::Left, Keys::RotateLeft);
        default_key_map.insert(VirtualKeyCode::W, Keys::MoveForward);
        default_key_map.insert(VirtualKeyCode::S, Keys::MoveBackward);
        default_key_map.insert(VirtualKeyCode::D, Keys::MoveRight);
        default_key_map.insert(VirtualKeyCode::A, Keys::MoveLeft);
        default_key_map.insert(VirtualKeyCode::E, Keys::MoveUp);
        default_key_map.insert(VirtualKeyCode::Q, Keys::MoveDown);
        let active = vec![];
        Keyboard {
            key_map: default_key_map,
            active,
        }
    }
}

impl Keyboard {
    pub fn keyboard_events(&mut self, input: KeyboardInput) {
        let input_match = &self.key_map.get(&input.virtual_keycode.unwrap());
        match input_match {
            Some(..) => {
                if input.state == ElementState::Pressed {
                    if !self.active.contains(input_match.unwrap()) {
                        self.active.push(input_match.unwrap().clone());
                    }
                } else if input.state == ElementState::Released {
                    // keep all items that are not input_match
                    self.active.retain(|key| key != input_match.unwrap());
                }
            }
            _ => {}
        }
    }
}
