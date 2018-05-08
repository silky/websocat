use super::{Peer, BoxedNewPeerFuture};

use std;
use futures;
use std::io::{Read,Write};
use std::io::Result as IoResult;

use futures::Async::{Ready};


use tokio_io::{AsyncRead,AsyncWrite};

use super::ReadDebt;
use super::wouldblock;

use super::{once,Specifier,Handle,ProgramState,PeerConstructor,Options};

#[derive(Clone)]
pub struct Literal(pub Vec<u8>);
impl Specifier for Literal {
    fn construct(&self, _:&Handle, _: &mut ProgramState, _opts: &Options) -> PeerConstructor {
        once(get_literal_peer(self.0.clone()))
    }
    specifier_boilerplate!(singleconnect no_subspec noglobalstate typ=Other);
}
impl std::fmt::Debug for Literal{fn fmt(&self, f:&mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> { write!(f, "Literal") }  }

#[derive(Clone)]
pub struct Assert(pub Vec<u8>);
impl Specifier for Assert {
    fn construct(&self, _:&Handle, _: &mut ProgramState, _opts: &Options) -> PeerConstructor {
        once(get_assert_peer(self.0.clone()))
    }
    specifier_boilerplate!(noglobalstate singleconnect no_subspec typ=Other);
}
impl std::fmt::Debug for Assert{fn fmt(&self, f:&mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> { write!(f, "Assert") }  }

#[derive(Debug,Clone)]
pub struct Clogged;
impl Specifier for Clogged {
    fn construct(&self, _:&Handle, _: &mut ProgramState, _opts: &Options) -> PeerConstructor {
        once(get_clogged_peer())
    }
    specifier_boilerplate!(noglobalstate singleconnect no_subspec typ=Other);
}






struct LiteralPeer {
    debt: ReadDebt,
}

pub fn get_literal_peer(b:Vec<u8>) -> BoxedNewPeerFuture {
    let r = LiteralPeer{debt: ReadDebt(Some(b))};
    let w = DevNull;
    let p = Peer::new(r,w);
    Box::new(futures::future::ok(p)) as BoxedNewPeerFuture
}
pub fn get_assert_peer(b:Vec<u8>) -> BoxedNewPeerFuture {
    let r = DevNull;
    let w = AssertPeer(vec![], b);
    let p = Peer::new(r,w);
    Box::new(futures::future::ok(p)) as BoxedNewPeerFuture
}
/// A special peer that returns NotReady without registering for any wakeup, deliberately hanging all connections forever.
pub fn get_clogged_peer() -> BoxedNewPeerFuture {
    let r = CloggedPeer;
    let w = CloggedPeer;
    let p = Peer::new(r,w);
    Box::new(futures::future::ok(p)) as BoxedNewPeerFuture
}


impl AsyncRead for LiteralPeer
{}


impl Read for LiteralPeer
{
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        if let Some(ret) = self.debt.check_debt(buf) {
            return ret;
        }
        Ok(0)
    }
}



struct DevNull;

impl AsyncWrite for DevNull {
    fn shutdown(&mut self) -> futures::Poll<(),std::io::Error> {
        Ok(Ready(()))
    }
}
impl Write for DevNull {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}
impl AsyncRead for DevNull
{}
impl Read for DevNull
{
    fn read(&mut self, _buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        Ok(0)
    }
}


struct AssertPeer(Vec<u8>, Vec<u8>);
impl AsyncWrite for AssertPeer {
    fn shutdown(&mut self) -> futures::Poll<(),std::io::Error> {
        assert_eq!(self.0, self.1);
        info!("Assertion succeed");
        Ok(Ready(()))
    }
}

impl Write for AssertPeer {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

struct CloggedPeer;
impl AsyncWrite for CloggedPeer {
    fn shutdown(&mut self) -> futures::Poll<(),std::io::Error> {
        wouldblock()
    }
}
impl Write for CloggedPeer {
    fn write(&mut self, _buf: &[u8]) -> IoResult<usize> {
        wouldblock()
    }
    fn flush(&mut self) -> IoResult<()> {
        wouldblock()
    }
}
impl AsyncRead for CloggedPeer
{}
impl Read for CloggedPeer
{
    fn read(&mut self, _buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        wouldblock()
    }
}