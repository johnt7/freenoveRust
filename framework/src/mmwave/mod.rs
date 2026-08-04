pub struct mmWave<'ch> {
    channel: Option<Channel<'ch, Blocking, Tx>>,
}