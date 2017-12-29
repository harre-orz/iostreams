use std::io::{Result, Read,Write};
use std::mem;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::rc::Rc;


pub trait Sink<Ch: Copy> {
    fn write(sink: &mut Self, buf: &[Ch]) -> Result<usize>;

    fn put(sink: &mut Self, ch: Ch) -> Result<()> {
        let buf = [ch];
        let _ = Sink::write(sink, &buf)?;
        Ok(())
    }
}


pub trait OutputFilter<Ch: Copy> {
    fn put<S: Sink<Ch>>(&mut self, sink: &mut S, ch: Ch) -> Result<()>;
}


pub struct CharCounterFilter<Ch: Copy> {
    c: Rc<RefCell<usize>>,
    _marker: PhantomData<Ch>,
}

impl<Ch: Copy> CharCounterFilter<Ch> {
    pub fn new(c: Rc<RefCell<usize>>) -> Self {
        CharCounterFilter {
            c: c,
            _marker: PhantomData,
        }
    }
}

impl<Ch: Copy> OutputFilter<Ch> for CharCounterFilter<Ch> {
    fn put<S: Sink<Ch>>(&mut self, sink: &mut S, ch: Ch) -> Result<()> {
        let _ = Sink::put(sink, ch);
        *(self.c.borrow_mut()) += 1;
        Ok(())
    }
}


pub trait FilterTrait<Ch: Copy, S: Sink<Ch>> {
    fn put_trait(&mut self, s: &mut S, ch: Ch) -> Result<()>;
}

pub struct FilterWrap<Ch: Copy, T: OutputFilter<Ch>> {
    t: T,
    _marker: PhantomData<Ch>,
}

impl<Ch: Copy, S: Sink<Ch>, T: OutputFilter<Ch>>  FilterTrait<Ch, S> for FilterWrap<Ch, T> {
    fn put_trait(&mut self, s: &mut S, ch: Ch) -> Result<()> {
        self.t.put(s, ch)
    }
}


pub struct Stream<Ch: Copy, S: Sink<Ch>> {
    s: S,
    chain: Vec<Box<FilterTrait<Ch, S>>>,
}

impl<Ch: Copy + 'static, S: Sink<Ch>> Stream<Ch, S> {
    pub fn new(s: S) -> Self {
        Stream {
            s: s,
            chain: Vec::new(),
        }
    }

    pub fn push<F: OutputFilter<Ch> + 'static>(&mut self, f: F) {
        self.chain.push(Box::new(FilterWrap {
            t: f,
            _marker: PhantomData,
        }))
    }

    pub fn finish(self) -> S {
        self.s
    }

    pub fn write(&mut self, buf: &[Ch]) -> Result<usize> {
        for ch in buf {
            for f in &mut self.chain {
                f.put_trait(&mut self.s, *ch);
            }
        }
        Ok(buf.len())
    }
}

impl Sink<u8> for String {
    fn write(sink: &mut Self, buf: &[u8]) -> Result<usize> {
        for ch in buf {
            sink.push(*ch as char);
        }
        Ok(buf.len())
    }
}


#[test]
fn test_hoge() {
    let mut s = Stream::<u8, String>::new(String::new());

    let c = Rc::new(RefCell::new(0));
    s.push(CharCounterFilter::new(c.clone()));
    let _ = s.write(&b"hello"[..]);

    assert_eq!(*c.borrow(), 5);
    assert_eq!(s.finish(), "hello");
}
