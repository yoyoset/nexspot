use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    DeleteDC, DeleteObject, ReleaseDC, SelectObject, HDC, HGDIOBJ,
};

/// RAII Wrapper for GDI Objects. Use HGDIOBJ directly to simplify types in 0.61.3
pub struct AutoGdiObject {
    handle: HGDIOBJ,
}

impl AutoGdiObject {
    pub fn new<T: Into<HGDIOBJ>>(handle: T) -> Option<Self> {
        let h: HGDIOBJ = handle.into();
        if h.is_invalid() {
            None
        } else {
            // Note: HGDIOBJ in 0.61 might be a struct with .0
            Some(Self { handle: h })
        }
    }

    pub fn handle(&self) -> HGDIOBJ {
        self.handle
    }
}

impl Drop for AutoGdiObject {
    fn drop(&mut self) {
        unsafe {
            // DeleteObject takes HGDIOBJ
            let _ = DeleteObject(self.handle);
        }
    }
}

/// RAII Wrapper for Created Device Contexts
pub struct AutoCreatedDC {
    handle: HDC,
}

impl AutoCreatedDC {
    pub fn new(handle: HDC) -> Option<Self> {
        if handle.is_invalid() {
            None
        } else {
            Some(Self { handle })
        }
    }

    pub fn handle(&self) -> HDC {
        self.handle
    }
}

impl Drop for AutoCreatedDC {
    fn drop(&mut self) {
        unsafe {
            // DeleteDC takes HDC in 0.61 (sometimes Result, let's wrap if needed)
            let _ = DeleteDC(self.handle);
        }
    }
}

/// RAII Wrapper for Released Device Contexts
pub struct AutoReleasedDC {
    hwnd: HWND,
    handle: HDC,
}

impl AutoReleasedDC {
    pub fn new(hwnd: HWND, handle: HDC) -> Option<Self> {
        if handle.is_invalid() {
            None
        } else {
            Some(Self { hwnd, handle })
        }
    }

    pub fn handle(&self) -> HDC {
        self.handle
    }
}

impl Drop for AutoReleasedDC {
    fn drop(&mut self) {
        unsafe {
            // ReleaseDC in 0.61 takes Option<HWND> and HDC
            let _ = ReleaseDC(Some(self.hwnd), self.handle);
        }
    }
}

/// RAII Wrapper for SelectObject
pub struct AutoSelectObject<'a> {
    dc: HDC,
    old_object: HGDIOBJ,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> AutoSelectObject<'a> {
    pub fn new(dc: HDC, object: HGDIOBJ) -> Self {
        unsafe {
            // SelectObject in 0.61 takes HDC and HGDIOBJ
            let old_object = SelectObject(dc, object);
            Self {
                dc,
                old_object,
                _marker: std::marker::PhantomData,
            }
        }
    }
}

impl<'a> Drop for AutoSelectObject<'a> {
    fn drop(&mut self) {
        unsafe {
            // Restore the old object
            let _ = SelectObject(self.dc, self.old_object);
        }
    }
}
