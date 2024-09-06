#![no_std]
#![no_main]

#[cfg(test)]
#[embedded_test::tests(executor = embassy_executor::Executor::new())]
mod tests {
    use daisy_embassy::default_rcc;
    use daisy_embassy::DaisyBoard;
    use defmt_rtt as _;

    // A init function which is called before every test
    #[init]
    async fn init() -> DaisyBoard<'static> {
        let rcc = default_rcc();
        let p = embassy_stm32::init(rcc);
        let board = daisy_embassy::new_daisy_board!(p);

        // The init function can return some state, which can be consumed by the testcases
        board
    }

    #[test]
    fn first_test(_board: DaisyBoard<'static>) -> Result<(), &'static str> {
        Ok(())
    }
}
