pub mod theme_palette;
pub mod theme_tokens;

// Compat aliases. Prefixed module paths above are the canonical
// names; the unprefixed forms remain so existing call sites do not
// have to be touched in lockstep with the rename.
pub use theme_palette as theme;
pub use theme_tokens as tokens;
