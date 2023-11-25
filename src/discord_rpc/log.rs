use home::home_dir;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::{format::FmtSpan, Subscriber};
pub fn setup() {
    let log_dir = if cfg!(windows) {
        home_dir()
            .expect("Could not get home directory")
            .join(".logs\\discord-rpc-lastfm")
            .display()
            .to_string()
    } else {
        "/var/log/".to_string()
    };
    let file_appender =
        RollingFileAppender::new(Rotation::DAILY, log_dir, "discord-rpc-lastfm.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = Subscriber::builder()
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");
}
