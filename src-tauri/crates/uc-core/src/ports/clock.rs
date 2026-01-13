pub trait ClockPort: Send + Sync {
    fn now_ms(&self) -> i64;
}
