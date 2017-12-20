
use std::io::{self,Write};
use rand;

use reply;

pub trait Command {
    type Reply : reply::Reply;
    fn encode_into<W>(&self, &mut W) -> Result<(), io::Error>
        where W : Write + ?Sized;
    fn encode(&self) -> Result<Vec<u8>,io::Error> {
        let mut res = Vec::new();
        self.encode_into(&mut res)?;
        Ok(res)
    }
}

// SETCONF

#[derive(Debug,Clone)]
pub struct SetConf<'a> {
    values : Vec<(&'a str, &'a str)>
}

impl<'a> SetConf<'a> {
    pub fn new() -> Self {
        SetConf { values : Vec::new() }
    }
    pub fn add(&mut self, key : &'a str, val : &'a str) {
        self.values.push((key, val));
    }
}

impl<'a> Command for SetConf<'a> {
    type Reply = reply::BasicReply;
    fn encode_into<W>(&self, w : &mut W) -> Result<(),io::Error>
        where W : Write + ?Sized {
        write!(w, "SETCONF ")?;
        for &(k,v) in self.values.iter() {
            write!(w, "{}=", k)?;
            write_quoted(w, v.as_bytes())?;
        }
        write!(w, "\r\n")
    }
}

// RESETCONF

// XXXX

// GETCONF

#[derive(Debug,Clone)]
pub struct GetConf<'a> {
    names : Vec<&'a str>
}

impl<'a> GetConf<'a> {
    pub fn new() -> Self {
        GetConf { names : Vec::new() }
    }
    pub fn add(&mut self, key : &'a str) {
        self.names.push(key);
    }
}

impl<'a> Command for GetConf<'a> {
    type Reply = reply::BasicReply;
    fn encode_into<W>(&self, w : &mut W) -> Result<(),io::Error>
        where W : Write + ?Sized {
        write!(w, "GETCONF")?;
        for k in self.names.iter() {
            write!(w, " {}", k)?;
        }
        write!(w, "\r\n")
    }
}

// SETEVENTS

// SETEVENTS
// AUTHENTICATE ++

pub struct Authenticate<'a> {
    authdata : &'a [u8],
    is_password : bool
}

impl<'a> Authenticate<'a> {
    pub fn new_cookie(cookie: &'a [u8]) -> Self {
        Authenticate { authdata : cookie, is_password : false }
    }
    pub fn new_passwd(passwd: &'a [u8]) -> Self {
        Authenticate { authdata : passwd, is_password : true }
    }
}

impl<'a> Command for Authenticate<'a> {
    type Reply = reply::BasicReply;
    fn encode_into<W>(&self, w : &mut W) -> Result<(),io::Error>
        where W : Write + ?Sized {
        write!(w, "AUTHENTICATE ")?;
        if self.is_password {
            write_quoted(w, self.authdata)?
        } else {
            write_hex(w, self.authdata)?
        }
        write!(w, "\r\n")
    }
}

// SAVECONF
// SIGNAL
// MAPADDRESS
// GETINFO
// EXTENDCIRCUIT
// SETCIRCUITPURPOSE
// SETROUTERPURPOSE
// ATTACHSTREAM
// POSTDESCRIPTOR
// REDIRECTSTREAM
// CLOSESTREAM
// CLOSECIRCUIT
// QUIT
// USEFEATURE
// RESOLVE
// PROTOCOLINFO ++

pub struct ProtocolInfo;

impl ProtocolInfo {
    pub fn new() -> ProtocolInfo {
        ProtocolInfo
    }
}

impl Command for ProtocolInfo {
    type Reply = reply::BasicReply;
    fn encode_into<W>(&self, w : &mut W) -> Result<(),io::Error>
        where W : Write + ?Sized {
        write!(w, "PROTOCOLINFO 1\r\n")
    }
}

// LOADCONF
// TAKEOWNERSHIP
// AUTHCHALLENGE

#[derive(Debug,Clone)]
pub struct AuthChallenge {
    nonce : Vec<u8>
}

impl AuthChallenge {
    pub fn new<R:rand::Rng>(rng : &mut R) -> AuthChallenge {
        let mut nonce = vec![0;32];
        rng.fill_bytes(&mut nonce);
        AuthChallenge { nonce }
    }
    pub fn new_with_nonce(use_nonce : &[u8]) -> AuthChallenge {
        let mut nonce = Vec::new();
        nonce.extend_from_slice(use_nonce);
        AuthChallenge { nonce }
    }
}

impl Command for AuthChallenge {
    type Reply = reply::BasicReply;
    fn encode_into<W>(&self, w : &mut W) -> Result<(),io::Error>
        where W : Write + ?Sized {
        write!(w, "AUTHCHALLENGE ")?;
        write_hex(w, &self.nonce)?;
        write!(w, "\r\n")
    }
}

// DROPGUARDS
// HSFETCH
// ADD_ONION
// DEL_ONION
// HSPOST



fn write_quoted<W>(w : &mut W, data:&[u8]) -> Result<(),io::Error>
    where W : Write + ?Sized {
    write!(w, "\"")?;
    w.write_all(data)?; // doesn't actually quote anything.
    write!(w, "\"")
}

fn write_hex<W>(w : &mut W, data:&[u8]) -> Result<(),io::Error>
    where W : Write + ?Sized {
    for byte in data.iter() {
        write!(w, "{:02x}", byte)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
