use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{HBITMAP, HBRUSH, HDC, HFONT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendHWND(pub HWND);
unsafe impl Send for SendHWND {}
unsafe impl Sync for SendHWND {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendHDC(pub HDC);
unsafe impl Send for SendHDC {}
unsafe impl Sync for SendHDC {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendHBITMAP(pub HBITMAP);
unsafe impl Send for SendHBITMAP {}
unsafe impl Sync for SendHBITMAP {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendHBRUSH(pub HBRUSH);
unsafe impl Send for SendHBRUSH {}
unsafe impl Sync for SendHBRUSH {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendHFONT(pub HFONT);
unsafe impl Send for SendHFONT {}
unsafe impl Sync for SendHFONT {}
