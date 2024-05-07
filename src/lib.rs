use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr, UdpSocket},
    str::FromStr,
    time::{Duration, SystemTime},
};

use godot::{
    engine::{global::Key, node::ProcessMode},
    prelude::*,
};
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
    ConnectionConfig, DefaultChannel, RenetClient,
};

// Start - Register Plugin
struct ArcadeClient;

#[gdextension]
unsafe impl ExtensionLibrary for ArcadeClient {}
// End - Register Plugin

// Start - System that manages connection with the server
#[derive(GodotClass)]
#[class(init, base=Node)]
struct GameplaySessionManager {
    base: Base<Node>,
    game_session: Option<GameSession>,
}

struct GameSession {
    // The client and transport are treated as the same thing because it doesn't make an different in this game.
    // Also setting up a singleton transport in Godot is annoying because you must make a GDScript that inherits
    // and add that to autoload for it to be processed. If you add a Node or subclass singleton via code, it
    // doesn't run `process`.
    client: RenetClient,
    transport: NetcodeClientTransport,

    // If there is an error, you will need to call join_session to (re)connect.
    transport_error: Result<(), NetcodeTransportError>,
}

#[godot_api]
impl INode for GameplaySessionManager {
    // This node is not allowed to be paused, so this is set as soon as it enters the tree/exists.
    // If it could be paused, then you could get undesirable stuff like disconnecting when opening a menu.
    fn enter_tree(&mut self) {
        self.base_mut().set_process_mode(ProcessMode::ALWAYS);
    }

    // Using a physics process because it runs 60 times a second, which is the same tickrate that we want to use for networking.
    // If a higher tickrate is desired, then change it in the project settings under Physics>Common.
    fn physics_process(&mut self, delta: f64) {
        // If the transport has an error we don't want to do anything.
        // When the transport has error, it will emit a signal on `lost_connection`. You can see where it
        // emits the signal below inside this function.
        if self.transport_has_error() {
            return;
        }

        // Update client and transport.
        let deltadur = Duration::from_secs_f64(delta);
        if let Some(session) = &mut self.game_session {
            session.client.update(deltadur);
            // Capturing any errors the transport might throw.
            session.transport_error = session.transport.update(deltadur, &mut session.client);
        }

        if self.transport_has_error() {
            let message = self.transport_error_message().to_variant();
            self.base_mut()
                .emit_signal("lost_connection".into(), &[message]);
            return;
        }

        if let Some(session) = &mut self.game_session {
            if session.client.is_connected() {
                // Get messages from the server.
                while let Some(message) = session
                    .client
                    .receive_message(DefaultChannel::ReliableOrdered)
                {
                    // Handle received message
                }

                // Send messages to the server.
                if Input::singleton().is_key_pressed(Key::W) {
                    session
                        .client
                        .send_message(DefaultChannel::ReliableOrdered, vec![8]);
                }
            }

            // Sends all packets to the server based on the client settings.
            session.transport_error = session.transport.send_packets(&mut session.client);
        }

        if self.transport_has_error() {
            let message = self.transport_error_message().to_variant();
            self.base_mut()
                .emit_signal("lost_connection".into(), &[message]);
            return;
        }
    }
}

#[godot_api]
impl GameplaySessionManager {
    #[signal]
    fn lost_connection(reason: GString);

    // Input server address should be ipv6.
    #[func]
    fn join_session(&mut self, address: GString, client_id: i64) {
        // Creating a client settings profile. This profile controls how the client communicates with the server.
        let client = RenetClient::new(ConnectionConfig::default());

        // Setup transport layer
        let server_addr: SocketAddr = address.to_string().parse().unwrap();
        let socket =
            UdpSocket::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)).unwrap();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        // This struct is a connection profile. It defines which server to connect to along with other info like
        // encryption, some basic user data, protocol id, etc...
        let authentication = ClientAuthentication::Unsecure {
            server_addr,
            // The client must get its id from another server/service/api that it will use to connect with this server.
            // Current id is temporary for testing purposes.
            client_id: client_id as u64,
            user_data: None,
            protocol_id: 0,
        };

        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

        self.game_session = Some(GameSession {
            client,
            transport,
            transport_error: Result::Ok(()),
        });
    }

    #[inline]
    fn transport_has_error(&self) -> bool {
        if let Some(session) = &self.game_session {
            return session.transport_error.is_err();
        }

        return false;
    }

    /// Only returns a message if there is an error inside the gameplay session.
    /// If there is no gameplay session or error, then it returns an empty string.
    #[inline]
    fn transport_error_message(&self) -> GString {
        if let Some(session) = &self.game_session {
            if let Err(error) = &session.transport_error {
                return GString::from_str(&error.to_string()).unwrap();
            }
        }

        return GString::new();
    }
}
