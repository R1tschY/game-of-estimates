#[cfg(test)]
macro_rules! test_for_message {
    ($rx:expr, $($pattern:tt)+) => {
        if loop {
            use futures_util::future::FutureExt;
            match $rx.recv().now_or_never() {
                None | Some(None) => break true,
                Some(Some(x)) => {
                    match x {
                        $($pattern)+ => break false,
                        x => println!("Ignored non-matching message {:?}", x),
                    }
                }
            }
        } {
            panic!("assertion failed: no message found that match `{}`",
                stringify!($($pattern)+));
        };
    };
}

#[cfg(test)]
macro_rules! assert_no_message {
    ($rx:expr, $($pattern:tt)+) => {
        let res = loop {
            use futures_util::future::FutureExt;
            match $rx.recv().now_or_never() {
                None | Some(None) => break None,
                Some(Some(x)) => {
                    match x {
                        $($pattern)+ => break Some(x),
                        x => println!("Ignored non-matching message {:?}", x),
                    }
                }
            }
        };
        if let Some(evt) = res {
            panic!("assertion failed: message found that match `{}`: {:?}",
                stringify!($($pattern)+), evt);
        };
    };
}

pub mod assets;
pub mod game_server;
pub mod player;
pub mod room;

pub mod adapters;
pub mod ports;

pub mod remote;

#[cfg(test)]
mod tests {
    use env_logger::WriteStyle;
    use log::LevelFilter;

    #[ctor::ctor]
    fn init() {
        env_logger::builder()
            .is_test(true)
            .filter(None, LevelFilter::Debug)
            .write_style(WriteStyle::Never)
            .init();
    }
}
