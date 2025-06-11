use nockvm::jets::sort::util::gor;
use nockvm::mem::NockStack;
use nockvm::noun::{Cell, Noun};
use nockvm::unifying_equality::unifying_equality;

use crate::noun::noun_ext::NounExt;

#[derive(Copy, Clone)]
pub struct HoonList {
    pub(super) next: Option<Cell>,
}

impl Iterator for HoonList {
    type Item = Noun;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|cell| {
            let tail = cell.tail();
            self.next = if tail.is_cell() {
                Some(tail.as_cell().unwrap_or_else(|err| {
                    panic!(
                        "Panicked with {err:?} at {}:{} (git sha: {:?})",
                        file!(),
                        line!(),
                        option_env!("GIT_SHA")
                    )
                }))
            } else {
                None
            };
            cell.head()
        })
    }
}

#[derive(Copy, Clone)]
pub struct HoonListLen<const N: usize> {
    next: Option<Cell>,
}

impl<const N: usize> HoonListLen<N> {
    pub fn new(next: Cell) -> Self {
        Self { next: Some(next) }
    }
    pub fn iter(self) -> HoonListLenIter<N> {
        HoonListLenIter {
            current: 0,
            next: self.next,
        }
    }
}

#[derive(Copy, Clone)]
pub struct HoonListLenIter<const N: usize> {
    current: usize,
    next: Option<Cell>,
}

impl<const N: usize> Default for HoonListLenIter<N> {
    fn default() -> Self {
        Self {
            current: 0,
            next: None,
        }
    }
}

pub fn next_cell(cell: Cell) -> Option<Cell> {
    let tail = cell.tail();
    if tail.is_cell() {
        Some(tail.as_cell().unwrap_or_else(|err| {
            panic!(
                "Panicked with {err:?} at {}:{} (git sha: {:?})",
                file!(),
                line!(),
                option_env!("GIT_SHA")
            )
        }))
    } else {
        None
    }
}

impl<const N: usize> Iterator for HoonListLenIter<N> {
    type Item = Noun;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= N {
            None
        } else {
            let new_cell = next_cell(self.next.expect("next wasn't a cell? in HoonListLenIter"))
                .expect("HoonListLenIter: next_cell failed, bad length?");
            self.next = Some(new_cell);
            self.current += 1;
            Some(new_cell.head())
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct HoonMap {
    pub(super) node: Noun,
    pub(super) left: Option<Cell>,
    pub(super) right: Option<Cell>,
}

impl HoonMap {
    pub fn get(&self, stack: &mut NockStack, mut k: Noun) -> Option<Noun> {
        let [mut ck, cv] = self.node.uncell().ok()?;

        if unsafe { unifying_equality(stack, &mut ck, &mut k) } {
            // ?:  =(b p.n.a)
            //   (some q.n.a)
            Some(cv)
        } else if gor(stack, k, ck).as_direct().map(|v| v.data()) == Ok(0) {
            // ?:  (gor b p.n.a)
            //   $(a l.a)
            let map: Self = self.left?.try_into().ok()?;
            map.get(stack, k)
        } else {
            // $(a r.a)
            let map: Self = self.right?.try_into().ok()?;
            map.get(stack, k)
        }
    }
}

use std::convert::TryFrom;

impl TryFrom<Noun> for HoonList {
    type Error = ();

    fn try_from(noun: Noun) -> Result<Self, Self::Error> {
        if noun.is_cell() {
            Ok(HoonList {
                next: Some(noun.as_cell().map_err(|_| ())?)
            })
        } else if noun.is_zero() {
            // Empty list is represented as 0
            Ok(HoonList { next: None })
        } else {
            Err(())
        }
    }
}

impl TryFrom<Cell> for HoonMap {
    type Error = ();

    fn try_from(cell: Cell) -> Result<Self, Self::Error> {
        // A map node has the structure [node left right]
        let node = cell.head();
        let tail = cell.tail();
        
        if let Ok(tail_cell) = tail.as_cell() {
            let left = tail_cell.head();
            let right = tail_cell.tail();
            
            Ok(HoonMap {
                node,
                left: left.as_cell().ok(),
                right: right.as_cell().ok(),
            })
        } else {
            Err(())
        }
    }
}

impl TryFrom<Noun> for HoonMap {
    type Error = ();

    fn try_from(noun: Noun) -> Result<Self, Self::Error> {
        if let Ok(cell) = noun.as_cell() {
            HoonMap::try_from(cell)
        } else {
            Err(())
        }
    }
}

impl From<Noun> for HoonMapIter {
    fn from(noun: Noun) -> Self {
        HoonMapIter {
            stack: vec![noun.as_cell().ok()],
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct HoonMapIter {
    pub(super) stack: Vec<Option<Cell>>,
}

impl Iterator for HoonMapIter {
    type Item = Noun;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(maybe_cell) = self.stack.pop() {
            if let Some(cell) = maybe_cell {
                if let Ok(cell_trie) = HoonMap::try_from(cell) {
                    self.stack.push(cell_trie.right);
                    self.stack.push(cell_trie.left);
                    return Some(cell_trie.node);
                } else {
                    return self.next();
                }
            } else {
                return self.next();
            }
        }
        None
    }
}
