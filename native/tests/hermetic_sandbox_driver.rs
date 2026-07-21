//! Hermetic SandboxDriver spawn — exercises Direct backend without host binaries.
//!
//! FR-006 / ADR-006 M2: `PLAYCUA_SANDBOX_BACKEND=direct` must spawn a real
//! child, expose tunnel stdio, and shut down within the 5s grace window.

use playcua_native::modality::sandbox::{SandboxBackend, SandboxDriver};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn direct_driver_spawn_tunnel_and_shutdown() {
    let mut driver = SandboxDriver::new(SandboxBackend::Direct);
    #[cfg(unix)]
    {
        driver
            .spawn_guest("cat", &[])
            .await
            .expect("direct cat spawn");
    }
    #[cfg(windows)]
    {
        driver
            .spawn_guest(
                "cmd",
                &["/Q".into(), "/K".into(), "more".into()],
            )
            .await
            .expect("direct more spawn");
    }

    let pid = driver.child_id().expect("pid");
    assert!(pid > 0);

    let mut stdin = driver.tunnel_stdin().expect("stdin");
    let mut stdout = driver.tunnel_stdout().expect("stdout");

    stdin.write_all(b"ping\n").await.expect("write");
    stdin.flush().await.expect("flush");
    // Close stdin so `cat`/`more` can finish echoing on some platforms.
    drop(stdin);

    let mut buf = vec![0u8; 4];
    let n = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        stdout.read_exact(&mut buf),
    )
    .await
    .expect("read timed out")
    .expect("read");
    assert_eq!(n, 4);
    assert_eq!(&buf, b"ping");

    driver.shutdown().await.expect("shutdown");
}
