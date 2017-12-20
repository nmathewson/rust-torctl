
extern crate rand;

pub mod command;
pub mod reply;
mod syn;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
