use std::sync::Arc;

use anyhow::Ok;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, Semaphore}
use tracing::{info, instrument};

use crate::{Connection, Db, DbDropGuard, Shutdown};

#[derive(Debug)]
struct Listener {
    db_holder: DbDropGuard,
    listener : TcpStream,
    limit_connections : Arc<Semaphore>,
    notify_shutdown : broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}
#[derive(Debug)]
struct Handler {
    db :Db,
    connection : Connection,
    shutdown : Shutdown,
    _shutdown_complete: mpsc::Sender<()>,
}

const MAX_CONNECTIONS: usize = 250;


pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Listener{
        listener,
        db_holder: DbDropGuard::new(),
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        shutdown_complete_tx,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "Failed to run server");
            }
        },
        _ = shutdown => {
            info!("shutting down")
        }
    }

    let Listener {
        shutdown_complete_tx,
        notify_shutdown,
        ..
    };
    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    let _ = shutdown_complete_tx.recv().await;
}

impl Listener {
    async fn run(&mut self) -> crate::Result<()> {
        info!("accepting inbound connections");

        loop {
            let permit = self.limit_connections.clone().acquire_owned().await.unwrap();

            let socket = self.accept().await?;

            let mut handler = Handler {
                db: self.db_holder.db(),
                connection : Connection::new(socket),
                shutdown : Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            tokio::spwan(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = %err, "Error running handler");
                }
                drop(permit);
            });

        }
    }

    async fn accept(&mut self) -> crate::Result<TcpStream> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into())
                    }
                }
            }
            time::sleep(Duration::from_millis(backoff)).await;
            backoff *= 2;
        }
    }
}

impl Handler {

    #[instrument(skip(self))]
    async fn run(&mut self) -> crate::Result<() > {
        while !self.shutdown.is_shutdown() {
            let maybe_frame = tokio::select! {
                res = self.connection.read_frame() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            };
        let frame = match maybe_frame {
            Some(frame) => frame,
            None => return Ok(()),
        };

        let cmd = Command::from_frame(frame)?;


        debug!(?cmd);


        cmd.apply(&self.db, &mut self.connection, &mut self.shutdown).await?;
        };

        Ok(())

    }

}
