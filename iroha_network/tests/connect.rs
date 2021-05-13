#[cfg(test)]
mod tests {
    #![allow(clippy::restriction)]

    use std::{sync::Arc, thread, time::Duration};

    use iroha_error::{Result, WrapErr};
    use iroha_network::prelude::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        sync::RwLock,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn test_connect_handling() {
        let _drop = tokio::spawn(async {
            Network::listen(
                Arc::new(RwLock::new(())),
                "127.0.0.1:8888",
                handle_connection,
            )
            .await
            .expect("Failed to listen.");
        });
        thread::sleep(Duration::from_millis(50));
        let network = Network::new("127.0.0.1:8888");
        let mut actual_changes = Vec::new();
        let connection = network
            .connect(&[0_u8, 10])
            .await
            .expect("Failed to connect.");

        for mut change in connection {
            actual_changes.append(&mut change);
        }
        assert_eq!(actual_changes.len(), 99);
    }

    async fn handle_connection(_state: State<()>, mut stream: Box<dyn AsyncStream>) -> Result<()> {
        for i in 1..100 {
            stream
                .write_all(&[i])
                .await
                .wrap_err("Failed to write message")?;
            stream.flush().await.wrap_err("Failed to flush")?;
            let mut receipt = [0_u8; 4];
            let _ = stream
                .read(&mut receipt)
                .await
                .wrap_err("Failed to read receipt")?;
        }
        Ok(())
    }
}