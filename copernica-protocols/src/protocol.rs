use {
    copernica_common::{LinkId, InterLinkPacket, PrivateIdentityInterface, constants, Operations},
    std::sync::mpsc::{sync_channel as channel, Receiver, SyncSender},
    crate::{TxRx},
    anyhow::{Result},
};

/*
     s = Protocol, l = Link, b = Broker, r = Router, 2 = to: e.g. l2b = "link to copernica_broker"
     link::{udp, mpsc_channel, mpsc_corruptor, etc}
                                                            +----------------------------+
    +-----------+p2l_tx   p2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx           b2r_rx    |   +-----------+   +-----------+
    |           +-------------->+           +-------------->-------------------------+   +-->+           +-->+           |
    | Protocol   |l2p_rx   l2p_tx|   Link    |b2l_rx   b2l_tx| r2b_rx       r2b_tx    |   |   |   Link    |   | Protocol   |
    |           +<--------------+           +<---------------<-------------------+   |   +<--+           +<--+           |
    +-----------+               +-----------+               |                    |   v   |   +-----------+   +-----------+
                                                            |                +---+---+-+ |
    +-----------+p2l_tx   p2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx   b2r_rx|         | |   +-----------+   +-----------+
    |           +-------------->+           +-------------->---------------->+         | +-->+           +-->+           |
    | Protocol   |l2p_rx   l2p_tx|   Link    |b2l_rx   b2l_tx| r2b_rx   r2b_tx|  Router | |   |   Link    |   |  Broker   |
    |           +<--------------+           +<---------------<---------------+         | +<--+           +<--+           |
    +-----------+               +-----------+               |                |         | |   +-----------+   +-----------+
                                                            |                +---+---+-+ |
    +-----------+b2l_tx   b2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx      b2r_rx ^   |   |   +-----------+   +-----------+
    |           +-------------->+           +-------------->---------------------+   |   +-->+           +-->+           |
    |  Broker   |l2b_rx   l2b_tx|   Link    |b2l_rx   b2l_tx| r2b_rx          r2b_tx |   |   |   Link    |   | Protocol   |
    |           +<--------------+           +<---------------<-----------------------+   +<--+           +<--+           |
    +-----------+               +-----------+               |           Broker           |   +-----------+   +-----------+
                                                            +----------------------------+
*/
pub trait Protocol {
    fn get_protocol_sid(&mut self) -> PrivateIdentityInterface;
    fn get_ops(&self) -> Operations;
    fn set_txrx(&mut self, txrx: TxRx);
    fn get_label(&self) -> String;
    fn peer_with_link(&mut self, link_id: LinkId) -> Result<(SyncSender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        let (l2p_tx, l2p_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (p2l_tx, p2l_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let txrx = TxRx::init(self.get_label(), self.get_ops(), link_id, self.get_protocol_sid(), p2l_tx, l2p_rx);
        self.set_txrx(txrx);
        Ok((l2p_tx, p2l_rx))
    }
    #[allow(unreachable_code)]
    fn run(&self) -> Result<()>;
    fn new(protocol_sid: PrivateIdentityInterface, ops: (String, Operations)) -> Self where Self: Sized;
}


