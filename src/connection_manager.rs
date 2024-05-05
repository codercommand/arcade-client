use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct ConnectionManager {
    base: Base<Object>,
}
