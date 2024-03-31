use crate::core::container::network::Network;

use super::property::TransportPropertyProvider;

struct TransportBuilder<'a, TP>
where
    TP: TransportPropertyProvider,
{
    network: Network,
    property_provider: &'a TP,
    //node_open: BinaryHeap<SiteCandidate>,
    //config: NetworkConfig,
}

impl<'a, TP> TransportBuilder<'a, TP>
where
    TP: TransportPropertyProvider,
{
    fn new(property_provider: &'a TP) -> Self {
        Self {
            network: Network::new(),
            property_provider,
            //node_open: BinaryHeap::new(),
            //config: NetworkConfig::new(),
        }
    }
}
