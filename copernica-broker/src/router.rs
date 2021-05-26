use {
    crate::{
        bloom_filter::{Blooms},
        Bayes, LinkWeight
    },
    copernica_common::{LinkId, InterLinkPacket, LinkPacket, NarrowWaistPacket, serialization::*},
    anyhow::Result,
    //log::{trace},
    futures::channel::mpsc::{UnboundedSender},
    log::{debug, warn},
    std::collections::HashMap,
};

#[derive(Clone)]
pub struct Router {}

impl Router {
    pub fn handle_packet(
        ilp: &InterLinkPacket,
        r2b_tx: UnboundedSender<InterLinkPacket>,
        response_store: sled::Db,
        blooms: &mut HashMap<LinkId, Blooms>,
        bayes: &mut Bayes,
        choke: &LinkId,
    ) -> Result<()> {
        //debug!("\t|  |  |  broker-to-router");
        let this_link: LinkId = ilp.link_id();
        let nw: NarrowWaistPacket = ilp.narrow_waist();
        if let Some(this_bloom) = blooms.get_mut(&this_link) {
            match nw.clone() {
                NarrowWaistPacket::Request { hbfi, .. } => {
                    let (_hbfi_size, hbfi_s) = serialize_hbfi(&hbfi)?;
                    match response_store.get(&hbfi_s)? {
                        Some(response) => {
                            let nw: NarrowWaistPacket = deserialize_narrow_waist_packet(&response.to_vec())?;
                            debug!("\t\t|  |  |  |  RESPONSE PACKET FOUND");
                            let lp = LinkPacket::new(this_link.reply_to()?, nw);
                            let ilp = InterLinkPacket::new(this_link.clone(), lp);
                            r2b_tx.unbounded_send(ilp)?;
                            return Ok(());
                        }
                        None => {
                            debug!("\t\t|  |  |  |  FORWARD REQUEST UPSTREAM");
                            this_bloom.create_pending_request(&hbfi);
                            let link_weights = bayes.classify(&hbfi.to_bfis());
                            //std::thread::sleep_ms(500);
                            bayes.train(&hbfi.to_bfis(), choke);
                            if link_weights[0].linkid == *choke {
                                //warn!("{}, {:?}", link_weights[0].weight, link_weights[0].linkid);
                                let litmus_weight = (link_weights[0].weight * 100.00) as u64;
                                match litmus_weight {
                                    0..=35 => {
                                        warn!("Defcon 4: Do something")
                                    },
                                    36..=59 => {
                                        warn!("Defcon 3: Do something")
                                        // packets need a nonce and it needs to be signed. So as to avert the
                                        // scenario whereby an attacker replays requests thus shutting down
                                        // the flow of legitimate information.
                                    },
                                    60..=89 => {
                                        warn!("Defcon 2: Do something")
                                    },
                                    90..=u64::MAX => {
                                        warn!("Defcon 1: Deep Sixed packet: {:?}", hbfi);
                                        return Ok(())
                                    },
                                }
                            }
                            let mut forwarded = false;
                            for LinkWeight { linkid: that_link, weight} in link_weights {
                                //warn!("{}, {:?}", weight, that_link);
                                if that_link == *choke {
                                    continue;
                                }
                                if that_link.link_pid()? == this_link.link_pid()? {
                                    continue;
                                }
                                if let Some(that_bloom) = blooms.get_mut(&that_link) {
                                    if that_bloom.contains_forwarded_request(&hbfi) {
                                        continue;
                                    }
                                    if that_bloom.contains_pending_request(&hbfi) {
                                        continue;
                                    }
                                    if (weight < 0.00) && (forwarded == false) {
                                        that_bloom.create_forwarded_request(&hbfi);
                                        r2b_tx.unbounded_send(ilp.change_destination(that_link))?;
                                        continue;
                                    }
                                    that_bloom.create_forwarded_request(&hbfi);
                                    r2b_tx.unbounded_send(ilp.change_destination(that_link))?;
                                    forwarded = true;
                                }
                            }
                        }
                    }
                }
                NarrowWaistPacket::Response { hbfi, .. } => {
                    if this_bloom.contains_forwarded_request(&hbfi) {
                        let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                        let (_, nw_s) = serialize_narrow_waist_packet(&nw)?;
                        response_store.insert(hbfi_s, nw_s)?;
                        bayes.super_train(&hbfi.to_bfis(), &this_link);
                        // ^^^ think about an attack whereby a response is continually sent thus adjusting the weights
                        this_bloom.delete_forwarded_request(&hbfi);
                        for (that_link, that_bloom) in blooms.iter_mut() {
                            if that_link.link_pid()? == this_link.link_pid()? {
                                continue;
                            }
                            if that_bloom.contains_pending_request(&hbfi) {
                                that_bloom.delete_pending_request(&hbfi);
                                debug!("\t\t|  |  |  |  FORWARD RESPONSE DOWNSTREAM");
                                r2b_tx.unbounded_send(ilp.change_destination(that_link.clone()))?;
                            }
                        }
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    }
}
