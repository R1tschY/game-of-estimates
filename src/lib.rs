#[cfg(test)]
macro_rules! test_for_message {
    ($rx:expr, $($pattern:tt)+) => {
        if loop {
            match $rx.try_recv() {
                Err(_) => break true,
                Ok(x) => {
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

pub mod game_server;
pub mod player;
pub mod room;

pub mod remote;
