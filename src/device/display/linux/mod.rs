use ashpd::{desktop::screencast::{Screencast, SourceType}, WindowIdentifier};

use super::Target;

// TODO
pub fn is_supported() -> bool {
    true
    // false
}

// TODO
pub fn has_permission() -> bool {
    true
    // false
}

// TODO: Place this functionality somewhere else
pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    async_std::task::block_on(async {
        let proxy = Screencast::new()
            .await
            .expect("Failed to create Screencast proxy");
        let session = proxy
            .create_session()
            .await
            .expect("Failed to create Screencast session");
        proxy
            .select_sources(
                &session,
                ashpd::desktop::screencast::CursorMode::Embedded,
                SourceType::Monitor | SourceType::Window,
                true,
                None,
                ashpd::desktop::screencast::PersistMode::DoNot,
            )
            .await
            .expect("Failed to select sources");

        let response = proxy.start(&session, &WindowIdentifier::default())
            .await.expect("Failed to start")
            .response().expect("Failed to get response");

        response.streams().iter().enumerate().for_each(|(n, stream)| {
            targets.push(
                Target {
                    title: format!("node-{n}"),
                    id: stream.pipe_wire_node_id()
                }
            );
            println!("node id: {}", stream.pipe_wire_node_id());
            println!("size: {:?}", stream.size());
            println!("position: {:?}", stream.position());
        });
    });

    targets
}
