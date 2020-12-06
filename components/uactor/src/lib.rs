pub mod blocking;
pub mod core;
pub mod nonblocking;
pub mod tokio;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
