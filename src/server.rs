use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, Semaphore}

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
