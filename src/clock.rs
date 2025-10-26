use jiff::Timestamp;

pub fn now() -> Timestamp {
    #[cfg(test)]
    {
        use crate::tests::utils::mock_clock::TEST_CLOCK;
        TEST_CLOCK.with(|clock| clock.current())
    }
    #[cfg(not(test))]
    {
        Timestamp::now()
    }
}
