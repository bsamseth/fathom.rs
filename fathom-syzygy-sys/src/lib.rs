pub const TB_CASTLING_K: u32 = 1;
pub const TB_CASTLING_Q: u32 = 2;
#[allow(non_upper_case_globals)]
pub const TB_CASTLING_k: u32 = 4;
#[allow(non_upper_case_globals)]
pub const TB_CASTLING_q: u32 = 8;

pub const TB_LOSS: u32 = 0;
pub const TB_BLESSED_LOSS: u32 = 1;
pub const TB_DRAW: u32 = 2;
pub const TB_CURSED_WIN: u32 = 3;
pub const TB_WIN: u32 = 4;

pub const TB_PROMOTES_NONE: u32 = 0;
pub const TB_PROMOTES_QUEEN: u32 = 1;
pub const TB_PROMOTES_ROOK: u32 = 2;
pub const TB_PROMOTES_BISHOP: u32 = 3;
pub const TB_PROMOTES_KNIGHT: u32 = 4;

pub const TB_RESULT_WDL_MASK: u32 = 0x0000000F;
pub const TB_RESULT_TO_MASK: u32 = 0x000003F0;
pub const TB_RESULT_FROM_MASK: u32 = 0x0000FC00;
pub const TB_RESULT_PROMOTES_MASK: u32 = 0x00080000;
pub const TB_RESULT_EP_MASK: u32 = 0x00070000;
pub const TB_RESULT_DTZ_MASK: u32 = 0xFFF00000;
pub const TB_RESULT_WDL_SHIFT: u32 = 0;
pub const TB_RESULT_TO_SHIFT: u32 = 4;
pub const TB_RESULT_FROM_SHIFT: u32 = 10;
pub const TB_RESULT_PROMOTES_SHIFT: u32 = 16;
pub const TB_RESULT_EP_SHIFT: u32 = 19;
pub const TB_RESULT_DTZ_SHIFT: u32 = 20;

pub const TB_RESULT_CHECKMATE: u32 = 0x00000004;
pub const TB_RESULT_STALEMATE: u32 = 0x00000002;
pub const TB_RESULT_FAILED: u32 = 0xFFFFFFFF;

// The original function is declared static inline and therefore unaccessible from Rust code. We
// reimplement it here, because it is just a thin wrapper around another function anyway.
#[allow(clippy::missing_safety_doc, clippy::too_many_arguments)]
pub unsafe fn tb_probe_wdl(
    white: u64,
    black: u64,
    kings: u64,
    queens: u64,
    rooks: u64,
    bishops: u64,
    knights: u64,
    pawns: u64,
    rule50: u32,
    castling: u32,
    ep: u32,
    turn: u8,
) -> u32 {
    if castling != 0 {
        return TB_RESULT_FAILED;
    }

    if rule50 != 0 {
        return TB_RESULT_FAILED;
    }

    tb_probe_wdl_impl(
        white, black, kings, queens, rooks, bishops, knights, pawns, ep, turn,
    )
}

// The original function is declared static inline and therefore unaccessible from Rust code. We
// reimplement it here, because it is just a thin wrapper around another function anyway.
#[allow(clippy::missing_safety_doc, clippy::too_many_arguments)]
pub unsafe fn tb_probe_root(
    white: u64,
    black: u64,
    kings: u64,
    queens: u64,
    rooks: u64,
    bishops: u64,
    knights: u64,
    pawns: u64,
    rule50: u32,
    castling: u32,
    ep: u32,
    turn: u8,
    results: *mut u32,
) -> u32 {
    if castling != 0 {
        return TB_RESULT_FAILED;
    }

    tb_probe_root_impl(
        white, black, kings, queens, rooks, bishops, knights, pawns, rule50, ep, turn, results,
    )
}

extern "C" {
    pub static TB_LARGEST: u32;

    pub fn tb_init(filename: *const i8) -> bool;

    pub fn tb_free();

    fn tb_probe_wdl_impl(
        white: u64,
        black: u64,
        kings: u64,
        queens: u64,
        rooks: u64,
        bishops: u64,
        knights: u64,
        pawns: u64,
        ep: u32,
        turn: u8,
    ) -> u32;

    fn tb_probe_root_impl(
        white: u64,
        black: u64,
        kings: u64,
        queens: u64,
        rooks: u64,
        bishops: u64,
        knights: u64,
        pawns: u64,
        rule50: u32,
        ep: u32,
        turn: u8,
        results: *mut u32,
    ) -> u32;
}
