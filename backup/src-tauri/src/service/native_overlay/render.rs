use super::gdi::{AutoCreatedDC, AutoGdiObject, AutoReleasedDC, AutoSelectObject};
use windows::Win32::Foundation::{COLORREF, HWND, POINT, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, CreateSolidBrush, Ellipse, FrameRect,
    AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HDC, HGDIOBJ,
    SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, UpdateLayeredWindow, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, ULW_ALPHA,
};

pub fn update_with_buffer(width: i32, height: i32, _buffer: Vec<u8>) {
    if let Ok(mut guard) = super::state::STATE.lock() {
        if let Some(state) = guard.as_mut() {
            state.width = width;
            state.height = height;
        }
    }
}

pub fn redraw(hwnd: HWND, state: &mut super::state::OverlayState) {
    let x_screen = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let y_screen = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };

    unsafe {
        if let Ok(hdc_screen_raw) = windows::Win32::Graphics::Gdi::GetDC(None) {
            if let Some(hdc_screen) =
                AutoReleasedDC::new(HWND(std::ptr::null_mut()), hdc_screen_raw)
            {
                let hdc_screen_handle = hdc_screen.handle();

                if let Ok(hdc_mem_handle) = CreateCompatibleDC(Some(hdc_screen_handle)) {
                    if let Some(hdc_mem) = AutoCreatedDC::new(hdc_mem_handle) {
                        let hdc_mem_handle = hdc_mem.handle();

                        let mut bmi = BITMAPINFO {
                            bmiHeader: BITMAPINFOHEADER {
                                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                                biWidth: state.width,
                                biHeight: -state.height,
                                biPlanes: 1,
                                biBitCount: 32,
                                biCompression: BI_RGB.0,
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
                        if let Ok(hbit_handle) = CreateDIBSection(
                            Some(hdc_mem_handle),
                            &mut bmi,
                            DIB_RGB_COLORS,
                            &mut bits,
                            None,
                            0,
                        ) {
                            if let Some(hbit) = AutoGdiObject::new(hbit_handle) {
                                let hbit_handle = hbit.handle();
                                let _sel = AutoSelectObject::new(hdc_mem_handle, hbit_handle);

                                let mut p = bits as *mut u32;
                                let mask_fill = 0x80000000u32;
                                for _ in 0..(state.width * state.height) {
                                    *p = mask_fill;
                                    p = p.add(1);
                                }

                                if let Some(r) = state.final_rect {
                                    let left = std::cmp::min(r.left, r.right);
                                    let top = std::cmp::min(r.top, r.bottom);
                                    let w = (r.left - r.right).abs();
                                    let h = (r.top - r.bottom).abs();

                                    if let Ok(hdc_source_handle) =
                                        windows::Win32::Graphics::Gdi::GetDC(None)
                                    {
                                        if let Some(hdc_source) = AutoReleasedDC::new(
                                            HWND(std::ptr::null_mut()),
                                            hdc_source_handle,
                                        ) {
                                            let _ = BitBlt(
                                                hdc_mem_handle,
                                                left,
                                                top,
                                                w,
                                                h,
                                                Some(hdc_source.handle()),
                                                left + x_screen,
                                                top + y_screen,
                                                SRCCOPY,
                                            );
                                        }
                                    }

                                    for row in top..(top + h) {
                                        if row < 0 || row >= state.height {
                                            continue;
                                        }
                                        let row_start = bits as *mut u32;
                                        let p_row = row_start.add((row * state.width) as usize);
                                        for col in left..(left + w) {
                                            if col < 0 || col >= state.width {
                                                continue;
                                            }
                                            let p_pixel = p_row.add(col as usize);
                                            *p_pixel |= 0xFF000000;
                                        }
                                    }

                                    let rect = RECT {
                                        left,
                                        top,
                                        right: left + w,
                                        bottom: top + h,
                                    };

                                    let glow_colors = [0x00FFBB00, 0x00FF9900];
                                    for (i, &color) in glow_colors.iter().enumerate() {
                                        let offset = (i + 1) as i32;
                                        let glow_rect = RECT {
                                            left: rect.left - offset,
                                            top: rect.top - offset,
                                            right: rect.right + offset,
                                            bottom: rect.bottom + offset,
                                        };
                                        if let Ok(hbrush_handle) = CreateSolidBrush(COLORREF(color))
                                        {
                                            if let Some(glow_guard) =
                                                AutoGdiObject::new(hbrush_handle)
                                            {
                                                let _ = FrameRect(
                                                    hdc_mem_handle,
                                                    &glow_rect,
                                                    Some(windows::Win32::Graphics::Gdi::HBRUSH(
                                                        glow_guard.handle().0,
                                                    )),
                                                );
                                            }
                                        }
                                    }

                                    if let Ok(hbrush_cyan_handle) =
                                        CreateSolidBrush(COLORREF(0x00FFFF00))
                                    {
                                        if let Some(brush_guard) =
                                            AutoGdiObject::new(hbrush_cyan_handle)
                                        {
                                            let _ = FrameRect(
                                                hdc_mem_handle,
                                                &rect,
                                                Some(windows::Win32::Graphics::Gdi::HBRUSH(
                                                    brush_guard.handle().0,
                                                )),
                                            );
                                        }
                                    }

                                    let white_brush_handle = CreateSolidBrush(COLORREF(0x00FFFFFF));
                                    // Need to regenerate cyan brush handle if we consumed it above?
                                    // AutoGdiObject takes ownership via Drop logic (DeleteObject).
                                    // But AutoGdiObject::new takes Copy+Into<HGDIOBJ>.
                                    // Wait, AutoGdiObject DOES implement Drop.
                                    // If I create `brush_guard` above, it drops at end of scope.
                                    // I should re-create brushes or structure this better.
                                    // Actually, `CreateSolidBrush` creates a NEW object.
                                    // So reusing `hbrush_cyan_handle` is wrong if wrapped in AutoGdiObject which deletes it.
                                    // But here `hbrush_cyan_handle` is just the handle.
                                    // `AutoGdiObject::new(h)` copies the handle.
                                    // So if I have multiple AutoGdiObjects managing the SAME handle, double free occurs!
                                    // Fix: Check logic.
                                    // Above: `if let Some(brush_guard) = AutoGdiObject::new(...)`. `brush_guard` drops at end of scope.
                                    // So `hbrush_cyan_handle` is deleted.
                                    // Below: logic tries to use usage cyan/white for anchors.
                                    // I should re-create cyan brush or extend scope.

                                    // Let's simplify: Create new brushes for anchors.
                                    if let (Ok(hbrush_cyan_anchors), Ok(hbrush_white_anchors)) = (
                                        CreateSolidBrush(COLORREF(0x00FFFF00)),
                                        CreateSolidBrush(COLORREF(0x00FFFFFF)),
                                    ) {
                                        if let (Some(cyan_guard), Some(white_guard)) = (
                                            AutoGdiObject::new(hbrush_cyan_anchors),
                                            AutoGdiObject::new(hbrush_white_anchors),
                                        ) {
                                            let h_cyan = cyan_guard.handle();
                                            let h_white = white_guard.handle();

                                            let anchors = [
                                                (rect.left, rect.top),
                                                (rect.right, rect.top),
                                                (rect.right, rect.bottom),
                                                (rect.left, rect.bottom),
                                                ((rect.left + rect.right) / 2, rect.top),
                                                (rect.right, (rect.top + rect.bottom) / 2),
                                                ((rect.left + rect.right) / 2, rect.bottom),
                                                (rect.left, (rect.top + rect.bottom) / 2),
                                            ];

                                            let handle_size = 18;
                                            let half_h = handle_size / 2;

                                            for (ax, ay) in anchors.iter() {
                                                let r = RECT {
                                                    left: ax - half_h,
                                                    top: ay - half_h,
                                                    right: ax + half_h,
                                                    bottom: ay + half_h,
                                                };
                                                {
                                                    let _sel = AutoSelectObject::new(
                                                        hdc_mem_handle,
                                                        h_cyan,
                                                    );
                                                    let _ = Ellipse(
                                                        hdc_mem_handle,
                                                        r.left,
                                                        r.top,
                                                        r.right,
                                                        r.bottom,
                                                    );
                                                }

                                                let r_inner = RECT {
                                                    left: ax - half_h + 2,
                                                    top: ay - half_h + 2,
                                                    right: ax + half_h - 2,
                                                    bottom: ay + half_h - 2,
                                                };
                                                {
                                                    let _sel = AutoSelectObject::new(
                                                        hdc_mem_handle,
                                                        h_white,
                                                    );
                                                    let _ = Ellipse(
                                                        hdc_mem_handle,
                                                        r_inner.left,
                                                        r_inner.top,
                                                        r_inner.right,
                                                        r_inner.bottom,
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    state.toolbar.draw(hdc_mem_handle);

                                    use super::state::InteractionState;
                                    if state.interaction_state == InteractionState::Creating
                                        || matches!(
                                            state.interaction_state,
                                            InteractionState::Resizing(_)
                                        )
                                    {
                                        let mut pt = POINT::default();
                                        let _ =
                                            windows::Win32::UI::WindowsAndMessaging::GetCursorPos(
                                                &mut pt,
                                            );
                                        let _ = windows::Win32::Graphics::Gdi::ScreenToClient(
                                            Some(hwnd),
                                            &mut pt,
                                        );
                                        super::magnifier::draw_magnifier(
                                            hdc_mem_handle,
                                            pt.x,
                                            pt.y,
                                            state,
                                        );
                                    }
                                }

                                let pt_src = POINT::default();
                                let size_win = SIZE {
                                    cx: state.width,
                                    cy: state.height,
                                };
                                let blend = BLENDFUNCTION {
                                    BlendOp: AC_SRC_OVER as u8,
                                    SourceConstantAlpha: 255,
                                    AlphaFormat: 1,
                                    ..Default::default()
                                };

                                let _ = UpdateLayeredWindow(
                                    hwnd,
                                    Some(hdc_screen_handle),
                                    None,
                                    Some(&size_win),
                                    Some(hdc_mem_handle),
                                    Some(&pt_src),
                                    COLORREF(0),
                                    Some(&blend),
                                    ULW_ALPHA,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
