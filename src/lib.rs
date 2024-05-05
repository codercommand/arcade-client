use godot::engine::Engine;
use godot::prelude::*;

mod connection_manager;
use connection_manager::ConnectionManager;

struct ArcadeClient;

#[gdextension]
unsafe impl ExtensionLibrary for ArcadeClient {
    fn on_level_init(level: InitLevel) {
        // Only want this logic ran when at Scene level, because lower than this and some stuff might not be initialised.
        if level != InitLevel::Scene {
            return;
        }

        // The string name is an identifier that can be used to access the singleton in other parts of the code.
        Engine::singleton().register_singleton(
            StringName::from("ConnectionManager"),
            ConnectionManager::new_alloc().upcast(),
        );
    }

    fn on_level_deinit(level: InitLevel) {
        if level != InitLevel::Scene {
            return;
        }

        // Get the `Engine` instance and `StringName` for your singleton.
        let mut engine = Engine::singleton();
        let singleton_name = StringName::from("ConnectionManager");

        // We need to retrieve the pointer to the singleton object,
        // as it has to be freed manually - unregistering singleton
        // doesn't do it automatically.
        let singleton = engine
            .get_singleton(singleton_name.clone())
            .expect("cannot retrieve the singleton");

        // Unregistering singleton and freeing the object itself is needed
        // to avoid memory leaks and warnings, especially for hot reloading.
        engine.unregister_singleton(singleton_name);
        singleton.free();
    }
}
