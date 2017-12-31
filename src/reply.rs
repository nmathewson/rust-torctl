
use nom::{IResult,Needed,digit,crlf};
use std::str::{self,FromStr};

type Error = u32; // nooo XXXX

// Like take_until, but declares the input "incomplete" if the terminator
// isn't found.
macro_rules! take_until_flex (
    ($i:expr, $substr:expr) => ( {
        use nom::{Needed,FindSubstring,Slice};
        let res : IResult<_,_> = match ($i).find_substring($substr) {
            None => {
                IResult::Incomplete(Needed::Unknown)
            }, // XXXX size would be better
            Some(index) => {
                IResult::Done($i.slice(index..), $i.slice(0..index))
            }
        };
        res
    })
);

pub trait Reply : Sized {
    fn parse(&[u8]) -> IResult<&[u8],Self>;
    fn is_ok(&self) -> bool;
}

// reply that either succeeds with no data, or fails.
#[derive(Clone,PartialEq,Debug)]
pub struct BasicReply (
    Result<(), String>
);

impl Reply for BasicReply {
    fn parse(inp:&[u8]) -> IResult<&[u8],Self,Error> {
        match generic_reply(inp) {
            IResult::Done(rest, replybody) => {
                assert!(replybody.lines.len() > 0);
                if code_is_success(replybody.lines[0].code) {
                    IResult::Done(rest, BasicReply(Ok(())))
                } else {
                    IResult::Done(rest,
                              BasicReply(
                                  Err(String::from_utf8(replybody.lines[0]
                                                        .content.to_vec())
                                      .unwrap()) // XXXX unwrap!!!
                              ))
                }
            },
            IResult::Error(e) => { IResult::Error(e) }
            IResult::Incomplete(n) => { IResult::Incomplete(n) }
        }
    }
    fn is_ok(&self) -> bool {
        self.0.is_ok()
    }
}

#[derive(Clone,PartialEq,Debug)]
pub struct ReplyLine<'a> { // pub??? XXXX
    code : u16,
    more : bool,
    content : &'a [u8],
    #[allow(unused)] // XXXXX
    data : &'a [u8],
}

#[derive(Clone,PartialEq,Debug)]
pub struct ReplyBody<'a> { // pub??? XXXXX
    lines : Vec<ReplyLine<'a>>
}

fn validate_status_code(a : &[u8]) -> Result<u16,Error> {
    if a.len() != 3 {
        Err(7) // XXXX 7
    } else {
        // XXXX unwrap
        Ok( FromStr::from_str(str::from_utf8(a).unwrap()).unwrap() )
    }
}

named!(status_code(&[u8]) -> u16,
       map_res!(digit, validate_status_code)
);

named!(linecontent(&[u8]) -> &[u8],
       do_parse!(
           stuff : take_until_flex!("\r\n") >>
           crlf >>
           (stuff)
       )
);

named!(cmd_data(&[u8])->&[u8],
       do_parse!(
           contents : take_until_flex!("\r\n.\r\n") >>
           tag!("\r\n.\r\n") >>
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

fn code_is_success(code : u16) -> bool{
    return code >= 200 && code < 300;
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{IResult,Needed};

    #[test]
    fn test_validate_status_code() {
        //assert_eq!(validate_status_code(b"abc"), Err(7));// XXXX
        assert_eq!(validate_status_code(b"abcd"), Err(7));// XXXX
        assert_eq!(validate_status_code(b"12"), Err(7));// XXXX
        assert_eq!(validate_status_code(b"123"), Ok(123));
    }

    #[test]
    fn test_reply_line() {
        assert_eq!(reply_line(b"200 OK\r\n2"),
                   IResult::Done(&b"2"[..],
                                 ReplyLine{
                                     code : 200, more: false, content: b"OK",
                                     data : b""}));

        assert_eq!(reply_line(b"205-it is all\r\n200 OK\r\n"),
                   IResult::Done(&b"200 OK\r\n"[..],
                                 ReplyLine{
                                     code : 205, more: true,
                                     content: b"it is all",
                                     data : b""}));


        assert_eq!(reply_line(
            b"205+it is all\r\n200 OK\r\nfor now ...\r\n.\r\nfoo"),
                   IResult::Done(&b"foo"[..],
                                 ReplyLine{
                                     code : 205, more: true,
                                     content: b"it is all",
                                     data : b"200 OK\r\nfor now ..."}));


        assert_eq!(reply_line(b"200 OK here it is \r"),
                   IResult::Incomplete(Needed::Unknown));

    }

}
