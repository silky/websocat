extern crate tokio_io;
extern crate futures;

use futures::future::ok;
use std::rc::Rc;
use std::cell::RefCell;

use super::{Peer, BoxedNewPeerFuture};

use tokio_io::{AsyncRead,AsyncWrite};
use std::io::{Read, Write, Error as IoError};

use std::ops::DerefMut;
use futures::Future;

type PeerSlot = Rc<RefCell<Option<Peer>>>;

#[derive(Default,Clone)]
pub struct GlobalState(PeerSlot);

#[derive(Clone)]
struct PeerHandle(PeerSlot);


impl Read for PeerHandle {
    fn read (&mut self, b:&mut [u8]) -> Result<usize, IoError> {
        if let Some(ref mut x) = self.0.borrow_mut().deref_mut() {
            x.0.read(b)
        } else {
            unreachable!()
        }
    }
}
impl AsyncRead for PeerHandle{}

impl Write for PeerHandle {
    fn write (&mut self, b: &[u8]) -> Result<usize, IoError> {
        if let Some(ref mut x) = self.0.borrow_mut().deref_mut() {
            x.1.write(b)
        } else {
            unreachable!()
        }
    }
    fn flush (&mut self) -> Result<(), IoError> {
        if let Some(ref mut x) = self.0.borrow_mut().deref_mut() {
            x.1.flush()
        } else {
            unreachable!()
        }
    }
}
impl AsyncWrite for PeerHandle {
    fn shutdown(&mut self) -> futures::Poll<(),IoError> {
        if let Some(ref mut x) = self.0.borrow_mut().deref_mut() {
            x.1.shutdown()
        } else {
            unreachable!()
        }
    }
}


pub fn connection_reuser(s: &mut GlobalState, inner_peer : BoxedNewPeerFuture) -> BoxedNewPeerFuture
{
    let need_init = s.0.borrow().is_none();
    
    let rc = s.0.clone();
    
    if need_init {
        Box::new(inner_peer.and_then(move |inner| {
            {
                let mut b = rc.borrow_mut();
                let x : &mut Option<Peer> = b.deref_mut();
                *x = Some(inner);
            }
            
            let ps : PeerSlot = rc.clone();
        
            let ph1 = PeerHandle(ps);
            let ph2 = ph1.clone();
            let peer = Peer::new(ph1, ph2);
            ok(peer)
        })) as BoxedNewPeerFuture
    } else {
        let ps : PeerSlot = rc.clone();
    
        let ph1 = PeerHandle(ps);
        let ph2 = ph1.clone();
        let peer = Peer::new(ph1, ph2);
        Box::new(ok(peer)) as BoxedNewPeerFuture
    }
}