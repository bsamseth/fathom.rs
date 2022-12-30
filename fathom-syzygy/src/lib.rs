use std::{
    ffi::CString,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use fathom_syzygy_sys as fathom;
use thiserror::Error;

pub const CASTLE_WHITE_KINGSIDE: u32 = fathom::TB_CASTLING_K;
pub const CASTLE_WHITE_QUEENSIDE: u32 = fathom::TB_CASTLING_Q;
pub const CASTLE_BLACK_KINGSIDE: u32 = fathom::TB_CASTLING_k;
pub const CASTLE_BLACK_QUEENSIDE: u32 = fathom::TB_CASTLING_q;

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("Invalid path")]
    InvalidPath,
    #[error("Library already initialized")]
    AlreadyInitialized,
}

static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct Fathom;

impl Fathom {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let initialized_before = unsafe { INITIALIZED.swap(true, Ordering::SeqCst) };
        if initialized_before {
            return Err(Error::AlreadyInitialized);
        }

        Self.reload(path)
    }

    pub fn reload<P: AsRef<Path>>(self, path: P) -> Result<Self, Error> {
        let pathref = path.as_ref();
        let pathstr = pathref.to_str().ok_or(Error::InvalidPath)?;
        let c_string = CString::new(pathstr).map_err(|_| Error::InvalidPath)?;

        unsafe { fathom::tb_init(c_string.as_ptr()) };

        Ok(self)
    }

    pub fn get_probers(&mut self) -> (RootProber<'_>, Prober<'_>) {
        (RootProber::new(), Prober::new())
    }

    pub fn max_pieces(&self) -> u32 {
        unsafe { fathom::TB_LARGEST }
    }
}

impl Drop for Fathom {
    fn drop(&mut self) {
        unsafe {
            fathom::tb_free();
            INITIALIZED.store(false, Ordering::SeqCst);
        }
    }
}

#[derive(Debug)]
pub struct RootProber<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> RootProber<'a> {
    // This function MUST NOT be public.
    fn new() -> Self {
        Self {
            phantom: Default::default(),
        }
    }

    pub fn max_pieces(&self) -> u32 {
        unsafe { fathom::TB_LARGEST }
    }

    pub fn probe(&mut self, position: &Position) -> Option<RootProbeResult> {
        let result = unsafe {
            fathom::tb_probe_root(
                position.white,
                position.black,
                position.kings,
                position.queens,
                position.rooks,
                position.bishops,
                position.knights,
                position.pawns,
                position.rule50,
                position.castling,
                position.ep,
                position.turn,
                std::ptr::null_mut(),
            )
        };

        if result == fathom::TB_RESULT_CHECKMATE
            || result == fathom::TB_RESULT_STALEMATE
            || result == fathom::TB_RESULT_FAILED
        {
            return None;
        }

        let wdl = Wdl::extract(result)?;
        let best_move = Move::extract(result)?;
        let dtz =
            u16::try_from((result & fathom::TB_RESULT_DTZ_MASK) >> fathom::TB_RESULT_DTZ_SHIFT)
                .ok()?;

        Some(RootProbeResult {
            wdl,
            best_move,
            dtz,
        })
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct RootProbeResult {
    pub wdl: Wdl,
    pub best_move: Move,
    pub dtz: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct Prober<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Prober<'a> {
    // This function MUST NOT be public.
    fn new() -> Self {
        Self {
            phantom: Default::default(),
        }
    }

    pub fn max_pieces(&self) -> u32 {
        unsafe { fathom::TB_LARGEST }
    }

    pub fn probe(&self, position: &Position) -> Option<Wdl> {
        let result = unsafe {
            fathom::tb_probe_root(
                position.white,
                position.black,
                position.kings,
                position.queens,
                position.rooks,
                position.bishops,
                position.knights,
                position.pawns,
                position.rule50,
                position.castling,
                position.ep,
                position.turn,
                std::ptr::null_mut(),
            )
        };

        if result == fathom::TB_RESULT_FAILED {
            return None;
        }

        Wdl::extract(result)
    }
}

pub struct Position {
    pub white: u64,
    pub black: u64,
    pub kings: u64,
    pub queens: u64,
    pub rooks: u64,
    pub bishops: u64,
    pub knights: u64,
    pub pawns: u64,
    pub rule50: u32,
    pub castling: u32,
    pub ep: u32,
    pub turn: u8,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Square(u8);

impl From<Square> for u8 {
    fn from(sq: Square) -> Self {
        sq.0
    }
}

impl Square {
    fn extract_to(result: u32) -> Option<Self> {
        let sq = (result & fathom::TB_RESULT_TO_MASK) >> fathom::TB_RESULT_TO_SHIFT;
        let sq = u8::try_from(sq).ok()?;
        if sq < 64 {
            Some(Self(sq))
        } else {
            None
        }
    }

    fn extract_from(result: u32) -> Option<Self> {
        let sq = (result & fathom::TB_RESULT_FROM_MASK) >> fathom::TB_RESULT_FROM_SHIFT;
        let sq = u8::try_from(sq).ok()?;
        if sq < 64 {
            Some(Self(sq))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum PromotionPiece {
    None,
    Knight,
    Bishop,
    Rook,
    Queen,
}

impl PromotionPiece {
    fn extract(result: u32) -> Option<Self> {
        match (result & fathom::TB_RESULT_PROMOTES_MASK) >> fathom::TB_RESULT_PROMOTES_SHIFT {
            fathom::TB_PROMOTES_NONE => Some(Self::None),
            fathom::TB_PROMOTES_KNIGHT => Some(Self::Knight),
            fathom::TB_PROMOTES_BISHOP => Some(Self::Bishop),
            fathom::TB_PROMOTES_ROOK => Some(Self::Rook),
            fathom::TB_PROMOTES_QUEEN => Some(Self::Queen),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promote: PromotionPiece,
    pub en_passant: bool,
}

impl Move {
    fn extract(result: u32) -> Option<Self> {
        let from = Square::extract_from(result)?;
        let to = Square::extract_to(result)?;
        let promote = PromotionPiece::extract(result)?;
        let en_passant = (result & fathom::TB_RESULT_EP_MASK) >> fathom::TB_RESULT_EP_SHIFT > 0;

        Some(Self {
            from,
            to,
            promote,
            en_passant,
        })
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Wdl {
    Loss,
    BlessedLoss,
    Draw,
    CursedWin,
    Win,
}

impl Wdl {
    fn extract(result: u32) -> Option<Self> {
        match (result & fathom::TB_RESULT_WDL_MASK) >> fathom::TB_RESULT_WDL_SHIFT {
            fathom::TB_LOSS => Some(Wdl::Loss),
            fathom::TB_BLESSED_LOSS => Some(Wdl::BlessedLoss),
            fathom::TB_DRAW => Some(Wdl::Draw),
            fathom::TB_CURSED_WIN => Some(Wdl::CursedWin),
            fathom::TB_WIN => Some(Wdl::Win),
            _ => None,
        }
    }
}
