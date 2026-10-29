use super::*;

#[test]
fn constructs_single_and_multiple_name_policies() {
    let one = CallerAuthorization::require_name("org.freedesktop.portal.Desktop").unwrap();
    assert_eq!(one.allowed_names.unwrap().len(), 1);

    let multiple = CallerAuthorization::require_names([
        "org.freedesktop.portal.Desktop",
        "org.freedesktop.portal.Documents",
    ])
    .unwrap();
    assert_eq!(multiple.allowed_names.unwrap().len(), 2);
}

#[test]
fn rejects_invalid_well_known_names() {
    assert!(CallerAuthorization::require_name(":1.42").is_err());
}

#[cfg(feature = "tokio")]
mod dbus {
    use std::{
        io::{BufRead, BufReader},
        process::{Child, Command, Stdio},
    };

    use zbus::{Connection, Message};

    use super::*;

    struct TestBus {
        child: Child,
        address: String,
    }

    impl TestBus {
        fn new() -> Self {
            let child = Command::new("dbus-daemon")
                .args(["--session", "--nofork", "--nopidfile", "--print-address=1"])
                .stdout(Stdio::piped())
                .spawn()
                .expect("authorization tests require dbus-daemon");
            let mut bus = Self {
                child,
                address: String::new(),
            };
            BufReader::new(bus.child.stdout.take().unwrap())
                .read_line(&mut bus.address)
                .unwrap();
            bus.address = bus.address.trim().to_owned();
            assert!(!bus.address.is_empty(), "dbus-daemon did not start");
            bus
        }

        async fn connect(&self) -> Connection {
            zbus::connection::Builder::address(self.address.as_str())
                .method_timeout(std::time::Duration::from_secs(5))
                .build()
                .await
                .unwrap()
        }
    }

    impl Drop for TestBus {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    const FRONTEND: &str = "org.test.Frontend";

    // Exercise the policy directly with real peers' unique names in the headers.
    fn message(sender: &Connection) -> Message {
        Message::method_call("/org/test/Backend", "Test")
            .sender(sender.unique_name().unwrap())
            .build(&())
            .unwrap()
    }

    fn assert_denied(result: Result<()>) {
        assert!(matches!(result, Err(PortalError::NotAllowed(_))));
    }

    #[tokio::test]
    async fn follows_current_name_ownership() {
        let bus = TestBus::new();
        let backend = bus.connect().await;
        let first = bus.connect().await;
        let second = bus.connect().await;
        let first_message = message(&first);
        let second_message = message(&second);
        let policy = CallerAuthorization::require_name(FRONTEND).unwrap();

        assert_denied(policy.authorize(&backend, &first_message.header()).await);
        first.request_name(FRONTEND).await.unwrap();
        policy
            .authorize(&backend, &first_message.header())
            .await
            .unwrap();
        assert_denied(policy.authorize(&backend, &second_message.header()).await);

        first.release_name(FRONTEND).await.unwrap();
        assert_denied(policy.authorize(&backend, &first_message.header()).await);
        second.request_name(FRONTEND).await.unwrap();
        assert_denied(policy.authorize(&backend, &first_message.header()).await);
        policy
            .authorize(&backend, &second_message.header())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn accepts_any_listed_owner_but_rejects_unlisted_peers() {
        let bus = TestBus::new();
        let backend = bus.connect().await;
        let first = bus.connect().await;
        let second = bus.connect().await;
        first.request_name(FRONTEND).await.unwrap();
        second.request_name("org.test.OtherFrontend").await.unwrap();
        let policy = CallerAuthorization::require_names([
            "org.test.Missing",
            FRONTEND,
            "org.test.OtherFrontend",
        ])
        .unwrap();

        policy
            .authorize(&backend, &message(&first).header())
            .await
            .unwrap();
        policy
            .authorize(&backend, &message(&second).header())
            .await
            .unwrap();
        assert_denied(
            policy
                .authorize(&backend, &message(&backend).header())
                .await,
        );
    }

    #[tokio::test]
    async fn empty_allowlist_denies_even_a_name_owner() {
        let bus = TestBus::new();
        let caller = bus.connect().await;
        caller.request_name(FRONTEND).await.unwrap();
        let policy = CallerAuthorization::require_names([] as [&str; 0]).unwrap();

        assert_denied(policy.authorize(&caller, &message(&caller).header()).await);
    }

    #[tokio::test]
    async fn unrestricted_policy_accepts_peers_and_senderless_headers() {
        let bus = TestBus::new();
        let caller = bus.connect().await;
        let policy = CallerAuthorization::allow_all();
        let senderless = Message::method_call("/org/test/Backend", "Test")
            .build(&())
            .unwrap();

        policy
            .authorize(&caller, &message(&caller).header())
            .await
            .unwrap();
        policy
            .authorize(&caller, &senderless.header())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn restricted_policy_rejects_senderless_headers() {
        let bus = TestBus::new();
        let caller = bus.connect().await;
        caller.request_name(FRONTEND).await.unwrap();
        let senderless = Message::method_call("/org/test/Backend", "Test")
            .build(&())
            .unwrap();
        let policy = CallerAuthorization::require_name(FRONTEND).unwrap();

        assert_denied(policy.authorize(&caller, &senderless.header()).await);
    }

    #[tokio::test]
    async fn adapts_denials_for_fdo_methods_and_property_access() {
        let bus = TestBus::new();
        let backend = bus.connect().await;
        let owner = bus.connect().await;
        owner.request_name(FRONTEND).await.unwrap();
        let allowed = message(&owner);
        let denied = message(&backend);
        let policy = CallerAuthorization::require_name(FRONTEND).unwrap();

        policy
            .authorize_fdo(&backend, &allowed.header())
            .await
            .unwrap();
        assert!(matches!(
            policy.authorize_fdo(&backend, &denied.header()).await,
            Err(zbus::fdo::Error::AccessDenied(_))
        ));
        policy
            .authorize_property(&backend, Some(&allowed.header()))
            .await
            .unwrap();
        assert!(matches!(
            policy
                .authorize_property(&backend, Some(&denied.header()))
                .await,
            Err(zbus::fdo::Error::AccessDenied(_))
        ));
        // Property-changed helpers call getters internally without a header.
        policy.authorize_property(&backend, None).await.unwrap();
    }
}
