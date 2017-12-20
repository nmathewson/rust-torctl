#![allow(unused)]

use nom::{IResult,Needed,digit,crlf};
use std::str::{self,FromStr};

type Error = u32;

pub trait Reply : Sized {
    fn parse(&[u8]) -> IResult<&[u8],Self>;
    fn is_ok(&self) -> bool;
}

pub struct BasicReply (
    Result<(), String>
);

impl Reply for BasicReply {
    fn parse(inp:&[u8]) -> IResult<&[u8],Self,Error> {
        IResult::Done(inp, BasicReply(Ok(())))
    }
    fn is_ok(&self) -> bool { true }
}

pub struct ReplyLine<'a> { // pub??? XXXX
    code : u16,
    more : bool,
    content : &'a [u8],
    data : &'a [u8],
}

pub struct ReplyBody<'a> { // pub??? XXXXX
    lines : Vec<ReplyLine<'a>>
}

fn validate_status_code(a : &[u8]) -> Result<u16,Error> {
    if a.len() != 3 {
        Err(7)
    } else {
        Ok( FromStr::from_str(str::from_utf8(a).unwrap()).unwrap() )
    }
}

named!(status_code(&[u8]) -> u16,
       map_res!(digit, validate_status_code)
);

named!(linecontent(&[u8]) -> &[u8],
       do_parse!(
           stuff : take_until!("\r\n") >>
           crlf >>
           (stuff)
       )
);

named!(cmd_data(&[u8])->&[u8],
       do_parse!(
           contents : take_until!(".\r\n") >>
           tag!(".") >>
           crlf >>
               (contents)
       )
);

named!(reply_line(&[u8]) -> ReplyLine,
       do_parse!(
           code : status_code >>
           x : alt!(
               do_parse!(
                   continued : alt!(tag!(b"-") | tag!(b" ")) >>
                   content : linecontent >>
                       ((continued, content, &b""[..]))
               ) |
               do_parse!(
                   continued : tag!(b"+") >>
                   content : linecontent >>
                   data : cmd_data >>
                   crlf >>
                       ((continued, content, data))
               )
           ) >>
               ({let (continued,content,data)=x;
                ReplyLine{code,
                          more:(continued != b" "),
                          content,
                          data
                } })
    )
);

fn generic_reply<'a>(input : &'a [u8])
                     -> IResult<&'a [u8],ReplyBody<'a>,Error> {
    let mut inp = input;
    let mut lines = Vec::new();
    loop {
        let ires = reply_line(inp);
        match ires {
            IResult::Done(rest,line) => {
                inp = rest;
                let more = line.more;
                lines.push(line);
                if ! more {
                    break;
                }
            }
            IResult::Error(err) => {
                return IResult::Error(err);//XXXX chain
            }
            IResult::Incomplete(needed) => {
                return IResult::Incomplete(needed);
            }
        }
    }
    IResult::Done(inp, ReplyBody{ lines })
}

pub fn read_reply<'a, R : Reply>(input : &'a [u8])
                             -> (Vec<ReplyBody>, IResult<&'a [u8], R, Error>)
{
    if input.len() == 0 {
        return (Vec::new(), IResult::Incomplete(Needed::Unknown))
    }

    let (async_replies, rest) = read_async_replies(input);
    (async_replies, R::parse(rest))
}

pub fn read_async_replies<'a>(input : &'a [u8]) -> (Vec<ReplyBody>, &'a [u8])
{
    let mut inp = input;
    let mut result = Vec::new();
    while inp.len() > 0 && inp[0] == b'6' {
        match generic_reply(inp) {
            IResult::Done(rest, reply) => {
                inp = rest;
                result.push(reply);
            }
            IResult::Error(_) => { break; }
            IResult::Incomplete(_) => { break; }
        }
    }
    (result, inp)
}
