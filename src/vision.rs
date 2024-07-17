use std::io::Error;
use find_subimage::{Backend, SubImageFinderState};
use winapi::shared::windef::HWND__;
use winapi::shared::minwindef::{UINT, DWORD};
use winapi::um::winuser::{GetDC, FindWindowW, GetClientRect, ReleaseDC, IsWindow, GetWindowTextW, GetForegroundWindow};
use winapi::um::wingdi::{DIB_RGB_COLORS, LPBITMAPINFO, GetDIBits, BI_RGB, BITMAPINFOHEADER, SRCCOPY, CreateCompatibleBitmap, CreateCompatibleDC, SelectObject, DeleteDC, DeleteObject, BitBlt};
use std::{mem, ptr};
use std::ffi::OsString;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

pub struct Vision {
    needles: Vec<Vec<u8>>,
    n_width: usize,
    n_height: usize,
    pub tags: Vec<String>,
    pub current_tags: Vec<String>,
    pub last_tags: Vec<String>,
    pub last_time: u128,
    err_img_minimized: eframe::egui::ColorImage,
    err_img_not_running: eframe::egui::ColorImage,
}

impl Vision {
    pub fn new() -> Self {
        let mut needles = Vec::new();
        let mut tags = Vec::new();
        let mut n_width: usize = 0;
        let mut n_height: usize = 0;
        let paths = std::fs::read_dir("./res/needles").unwrap();

        for path in paths {
            let path = path.unwrap();

            if path.path().extension().unwrap_or_default() == "jpg" {

                let img = image::io::Reader::open(path.path()).unwrap().decode().unwrap();
                n_width = img.width() as usize;
                n_height = img.height() as usize;
                let needle_buffer = img.to_rgba8();
                let needle = needle_buffer.as_flat_samples();

                needles.push(needle.as_slice().to_owned());

                let mut tag = path.file_name().into_string().unwrap();
                // tag = tag.strip_suffix(".jpg").unwrap().to_string();
                let n = tag.len() - 4;
                tag.truncate(n);

                tags.push(tag);
            }
        }
        let path = "./res/img/err/minimized.png";
        let err_img_minimized = super::load_image_from_path(path);
        let path = "./res/img/err/notrunning.png";
        let err_img_not_running = super::load_image_from_path(path);
        Self {
            needles,
            n_height,
            n_width,
            tags,
            current_tags: vec!["Starter".to_string(); 5],
            last_tags: vec!["Starter".to_string(); 5],
            last_time: 0,
            err_img_minimized,
            err_img_not_running,
        }

    }
    
    // pub fn no_top_operator(&self) -> bool {
    //     for tag in &self.current_tags {
    //         if tag == "Top Operator" {
    //             return false
    //         }
    //     }
    //     true
    // }

    pub fn get_window_name_of_foreground_window() -> String {
        let hwnd = unsafe { GetForegroundWindow() };
        let mut title = vec![0u16; 1024];
        let len = unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as _) };
        title.truncate(len as usize);
        let title = OsString::from_wide(&title);
        title.to_string_lossy().to_string()
    }
    
    #[allow(non_snake_case)]
    fn get_window_hwnd(window_name: &str) -> *mut HWND__ {

        let lpClassName = ptr::null_mut();
        let lpWindowName: Vec<u16> = OsStr::new(window_name).encode_wide().chain(once(0)).collect();
        // let lpWindowName = std::ffi::CString::new(window_name).unwrap();

        unsafe{ FindWindowW(lpClassName, lpWindowName.as_ptr()) }
        //unsafe{winapi::um::winuser::FindWindowA(lpClassName, lpWindowName.as_ptr())}

    }

    fn get_window_size(hwnd: *mut HWND__) -> (i32, i32){
        use std::mem::zeroed;

        let mut rect = unsafe { zeroed::<winapi::shared::windef::RECT>() };

        let ret = unsafe {
            GetClientRect(hwnd, &mut rect)
        };
        if ret == 0 { std::println!("{}", Error::last_os_error()) }

        (rect.right, rect.bottom)
    }

    fn image_resize(buffer: Vec<u8>, width: i32, height: i32, new_width: i32 , new_height: i32) -> Vec<u8> {
        let new_buffer = Vision::horizontal_sample(buffer, width as u32, height as u32, new_width as u32);
        Vision::vertical_sample(new_buffer, new_width as u32, height as u32, new_height as u32)
    }

    fn horizontal_sample(old_buffer: Vec<u8>, width: u32, height: u32, new_width: u32) -> Vec<u8> {
        let mut new_buffer: Vec<u8> = vec![0; (new_width * height * 4) as usize];
        let mut ws = Vec::new();

        let max: u8 = 255;
        let min: u8 = 0;
        let ratio = width as f32 / new_width as f32;
        let sratio = if ratio < 1.0 { 1.0 } else { ratio };
        // let src_support = filter.support * sratio;
        let src_support = 0.0 * sratio;

        for outx in 0..new_width {
            // Find the point in the input image corresponding to the centre
            // of the current pixel in the output image.
            let inputx = (outx as f32 + 0.5) * ratio;
    
            // Left and right are slice bounds for the input pixels relevant
            // to the output pixel we are calculating.  Pixel x is relevant
            // if and only if (x >= left) && (x < right).
    
            // Invariant: 0 <= left < right <= width
    
            let left = (inputx - src_support).floor() as i64;
            let left = Vision::clamp(left, 0, <i64 as From<_>>::from(width) - 1) as u32;
    
            let right = (inputx + src_support).ceil() as i64;
            let right = Vision::clamp(
                right,
                <i64 as From<_>>::from(left) + 1,
                <i64 as From<_>>::from(width),
            ) as u32;
    
            // Go back to left boundary of pixel, to properly compare with i
            // below, as the kernel treats the centre of a pixel as 0.
            let _inputx = inputx - 0.5;
    
            ws.clear();
            let mut sum = 0.0;
            for _i in left..right {
                // let w = (filter.kernel)((i as f32 - inputx) / sratio);
                let w = 1.0;
                ws.push(w);
                sum += w;
            }
            ws.iter_mut().for_each(|w| *w /= sum);
    
            for y in 0..height {
                let mut t = (0.0, 0.0, 0.0, 0.0);
    
                for (i, w) in ws.iter().enumerate() {
                    // let p = image.get_pixel(left + i as u32, y);
                    let index = ((y * width + (left + i as u32)) * 4) as usize;
                    let r: f64 = (old_buffer[index] as f64) / max as f64; 
                    let g: f64 = (old_buffer[index + 1] as f64) / max as f64; 
                    let b: f64 = (old_buffer[index + 2] as f64) / max as f64; 
                    let a: f64 = (old_buffer[index + 3] as f64) / max as f64; 
    
                    // #[allow(deprecated)]
                    // let vec = p.channels4();
    
                    t.0 += r * w;
                    t.1 += g * w;
                    t.2 += b * w;
                    t.3 += a * w;
                }
    
                // #[allow(deprecated)]
                // let t = Pixel::from_channels(
                //     NumCast::from(FloatNearest(clamp(t.0, min, max))).unwrap(),
                //     NumCast::from(FloatNearest(clamp(t.1, min, max))).unwrap(),
                //     NumCast::from(FloatNearest(clamp(t.2, min, max))).unwrap(),
                //     NumCast::from(FloatNearest(clamp(t.3, min, max))).unwrap(),
                // );
    
                // out.put_pixel(outx, y, t);
                new_buffer[((y * new_width + outx) * 4) as usize] = Vision::clamp((t.0 * max as f64) as u8, min, max);
                new_buffer[((y * new_width + outx) * 4) as usize + 1] = Vision::clamp((t.1 * max as f64) as u8, min, max);
                new_buffer[((y * new_width + outx) * 4) as usize + 2] = Vision::clamp((t.2 * max as f64) as u8, min, max);
                new_buffer[((y * new_width + outx) * 4) as usize + 3] = Vision::clamp((t.3 * max as f64) as u8, min, max);
            }
        }
        new_buffer
    }

    fn vertical_sample(old_buffer: Vec<u8>, width: u32, height: u32, new_height: u32) -> Vec<u8> {
        let mut new_buffer: Vec<u8> = vec![0; (width * new_height * 4) as usize];
        let mut ws = Vec::new();

        let max: u8 = 255;
        let min: u8 = 0;
        let ratio = height as f32 / new_height as f32;
        let sratio = if ratio < 1.0 { 1.0 } else { ratio };
        // let src_support = filter.support * sratio;
        let src_support = 0.0 * sratio;

        for outy in 0..new_height {
            // Find the point in the input image corresponding to the centre
            // of the current pixel in the output image.
            let inputy = (outy as f32 + 0.5) * ratio;
    
            // Left and right are slice bounds for the input pixels relevant
            // to the output pixel we are calculating.  Pixel x is relevant
            // if and only if (x >= left) && (x < right).
    
            // Invariant: 0 <= left < right <= width
    
            let left = (inputy - src_support).floor() as i64;
            let left = Vision::clamp(left, 0, <i64 as From<_>>::from(height) - 1) as u32;
    
            let right = (inputy + src_support).ceil() as i64;
            let right = Vision::clamp(
                right,
                <i64 as From<_>>::from(left) + 1,
                <i64 as From<_>>::from(height),
            ) as u32;
    
            // Go back to left boundary of pixel, to properly compare with i
            // below, as the kernel treats the centre of a pixel as 0.
            let _inputy = inputy - 0.5;
    
            ws.clear();
            let mut sum = 0.0;
            for _i in left..right {
                // let w = (filter.kernel)((i as f32 - inputy) / sratio);
                let w = 1.0;
                ws.push(w);
                sum += w;
            }
            ws.iter_mut().for_each(|w| *w /= sum);
    
            for x in 0..width {
                let mut t = (0.0, 0.0, 0.0, 0.0);
    
                for (i, w) in ws.iter().enumerate() {
                    // let p = image.get_pixel(x, left + i as u32);
                    // let index = ((y * width + (left + i as u32)) * 4) as usize;
                    let index = (((left + i as u32) * width + x) * 4) as usize;
                    let r: f64 = (old_buffer[index] as f64) / max as f64; 
                    let g: f64 = (old_buffer[index + 1] as f64) / max as f64; 
                    let b: f64 = (old_buffer[index + 2] as f64) / max as f64; 
                    let a: f64 = (old_buffer[index + 3] as f64) / max as f64; 
    
                    // #[allow(deprecated)]
                    // let vec = p.channels4();
    
                    t.0 += r * w;
                    t.1 += g * w;
                    t.2 += b * w;
                    t.3 += a * w;
                }
    
                // #[allow(deprecated)]
                // let t = Pixel::from_channels(
                //     NumCast::from(FloatNearest(clamp(t.0, min, max))).unwrap(),
                //     NumCast::from(FloatNearest(clamp(t.1, min, max))).unwrap(),
                //     NumCast::from(FloatNearest(clamp(t.2, min, max))).unwrap(),
                //     NumCast::from(FloatNearest(clamp(t.3, min, max))).unwrap(),
                // );
    
                // out.put_pixel(outx, y, t);
                new_buffer[((outy * width + x) * 4) as usize] = Vision::clamp((t.0 * max as f64) as u8, min, max);
                new_buffer[((outy * width + x) * 4) as usize + 1] = Vision::clamp((t.1 * max as f64) as u8, min, max);
                new_buffer[((outy * width + x) * 4) as usize + 2] = Vision::clamp((t.2 * max as f64) as u8, min, max);
                new_buffer[((outy * width + x) * 4) as usize + 3] = Vision::clamp((t.3 * max as f64) as u8, min, max);
            }
        }
        new_buffer
    }

    #[inline]
    fn clamp<N>(a: N, min: N, max: N) -> N
    where
        N: PartialOrd,
    {
        if a < min {
            min
        } else if a > max {
            max
        } else {
            a
        }
    }

    pub fn get_image_and_tag(&self, window_name: &str, position: (f32, f32, f32, f32)) -> (eframe::egui::ColorImage, String){

        let hwnd = Vision::get_window_hwnd(window_name);

        if unsafe {IsWindow(hwnd)} == 0 {
            return (self.err_img_not_running.clone(), "Starter".to_string());
        }

        let (mut width,mut height) = Vision::get_window_size(hwnd);

        // let resize = if width == 1920 && height == 1080 {false} else {true};

        if width < 100 || height < 100 {
            return (self.err_img_minimized.clone(), "Starter".to_string());
        }

        let mut adjust_left = 0;
        if (width as f32 / height as f32) > 1.777 {
            adjust_left = ((width as f32 - (height as f32 * 1.777)) / 2.0) as i32;
            width = (height as f32 * 1.777) as i32;
        }

        let mut left = (width as f32 * position.0) as i32;
        let top = (height as f32 * position.1) as i32;
        height = ((height as f32 * position.2) as i32) - top;
        width = ((width as f32 * position.3) as i32) - left;
        left = left + adjust_left;
        println!("left: {left} top: {top} height: {height} width: {width}");

        let mut buffer = Vision::get_screenshot(hwnd, height, width, left, top);

        // scale buffer for find_subimage if the resolution isn't 1920 x 1080
        // if resize {
            buffer = Vision::image_resize(buffer, width, height, 215, 69);
            width = 215;
            height = 69;
        // }


        let mut top_score = 1.0;
        let mut top_tag = "Starter".to_string();

        let mut finder = SubImageFinderState::new().with_backend(Backend::RuntimeDetectedSimd { threshold: 0.9, step_x: 1, step_y: 1 });
        for (i, needle) in self.needles.iter().enumerate() {
            let positions: &[(usize, usize, f32)] = finder.find_subimage_positions(
                    (&buffer, width as usize, height as usize),
                    (needle, self.n_width, self.n_height), 4, );

            if !positions.is_empty() && top_score > positions[0].2 {
                top_score = positions[0].2;
                top_tag = self.tags[i].clone();
            }
        }

        (eframe::egui::ColorImage::from_rgba_unmultiplied([width as _, height as _], &buffer), top_tag)
    }

    pub fn get_image(&self, window_name: &str, position: (f32, f32, f32, f32)) -> eframe::egui::ColorImage {

        let hwnd = Vision::get_window_hwnd(window_name);

        if unsafe {IsWindow(hwnd)} == 0 {
            return self.err_img_not_running.clone();
        }

        let (mut width,mut height) = Vision::get_window_size(hwnd);

        if width < 100 || height < 100 {
            return self.err_img_minimized.clone();
        }

        let mut adjust_left = 0;
        if (width as f32 / height as f32) > 1.777 {
            adjust_left = ((width as f32 - (height as f32 * 1.777)) / 2.0) as i32;
            width = (height as f32 * 1.777) as i32;
        }

        let mut left = (width as f32 * position.0) as i32;
        let top = (height as f32 * position.1) as i32;
        height = ((height as f32 * position.2) as i32) - top;
        width = ((width as f32 * position.3) as i32) - left;
        left = left + adjust_left;

        let buffer = Vision::get_screenshot(hwnd, height, width, left, top);

        eframe::egui::ColorImage::from_rgba_unmultiplied([width as _, height as _], &buffer)
    }

    fn get_screenshot(hwnd: *mut HWND__, height: i32, width: i32, left: i32, top: i32) -> Vec<u8>{
        unsafe {
            let screen = GetDC(hwnd);
            assert!(!screen.is_null());
            let dc = CreateCompatibleDC(screen);
            assert!(!dc.is_null());
            let bitmap = CreateCompatibleBitmap(screen, width, height);
            assert!(!bitmap.is_null());
            let result = SelectObject(dc, bitmap.cast());
            assert!(!result.is_null());
            let result = BitBlt(dc, 0, 0, width, height, screen, left, top, SRCCOPY);
            // let result = winuser::PrintWindow(hwnd, dc, winuser::PW_CLIENTONLY);
            assert_ne!(result, 0);
            assert_eq!(ReleaseDC(ptr::null_mut(), screen), 1);
            let mut header = BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as DWORD,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            };
            // let buf_size = width as usize * height as usize * 4;
            // let mut buffer: Vec<u8> = Vec::with_capacity(buf_size);
            // buffer.set_len(buf_size);
            let mut buffer: Vec<u8> = vec![0; (4 * width * height) as usize];
            let result = GetDIBits(
                dc,
                bitmap,
                0,
                height as UINT,
                buffer.as_mut_ptr().cast(),
                &mut header as *mut _ as LPBITMAPINFO,
                DIB_RGB_COLORS,
            );
            assert_eq!(result, height);
            assert_ne!(DeleteObject(bitmap.cast()), 0);
            assert_ne!(DeleteDC(dc), 0);

            // swap color channels
            buffer.chunks_exact_mut(4).for_each(|c| c.swap(0, 2));

            buffer
        }
    }
}

