use network::error::NetworkStartError;

#[derive(Debug)]
pub enum NodeLaunchError {
    NetworkStartError(NetworkStartError),
}

impl From<NetworkStartError> for NodeLaunchError {
    fn from(value: NetworkStartError) -> Self {
        Self::NetworkStartError(value)
    }
}