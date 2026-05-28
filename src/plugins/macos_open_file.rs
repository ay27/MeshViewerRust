#[cfg(target_os = "macos")]
use std::ffi::CStr;
#[cfg(target_os = "macos")]
use std::os::raw::c_char;
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "macos")]
use std::sync::{LazyLock, Mutex};
#[cfg(target_os = "macos")]
use std::{thread, time::Duration};

use bevy::prelude::*;

use crate::resources::ModelLoadRequest;
use crate::utils::file_utils::is_supported_format;

/// Handle Finder "Open With" / double-click document events on macOS.
///
/// Drag-and-drop and file dialogs are already covered elsewhere; this plugin
/// fills the AppKit delegate path used by Finder.
pub struct MacOsOpenFilePlugin;

impl Plugin for MacOsOpenFilePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_os = "macos")]
        app.add_systems(Startup, install_macos_open_hooks)
            .add_systems(Update, consume_pending_macos_open_files);
    }
}

fn partition_supported_paths<I>(paths: I) -> (Vec<PathBuf>, Vec<PathBuf>)
where
    I: IntoIterator<Item = PathBuf>,
{
    let mut supported = Vec::new();
    let mut unsupported = Vec::new();

    for path in paths {
        if is_supported_format(&path) {
            supported.push(path);
        } else {
            unsupported.push(path);
        }
    }

    (supported, unsupported)
}

#[cfg(target_os = "macos")]
static PENDING_MACOS_OPEN_FILES: LazyLock<Mutex<Vec<PathBuf>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
#[cfg(target_os = "macos")]
static OPEN_HOOKS_INSTALLED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
fn install_macos_open_hooks() {
    unsafe {
        install_winit_delegate_open_file_hooks();
    }
}

#[cfg(target_os = "macos")]
pub fn install_macos_open_hooks_early() {
    unsafe {
        install_winit_delegate_open_file_hooks();
    }
    start_hook_watcher_thread();
}

#[cfg(not(target_os = "macos"))]
pub fn install_macos_open_hooks_early() {}

#[cfg(target_os = "macos")]
fn consume_pending_macos_open_files(mut load_requests: MessageWriter<ModelLoadRequest>) {
    let mut pending = PENDING_MACOS_OPEN_FILES
        .lock()
        .expect("macOS open-file queue poisoned");
    if pending.is_empty() {
        return;
    }

    let paths = std::mem::take(&mut *pending);
    drop(pending);

    let (supported, unsupported) = partition_supported_paths(paths);
    for path in supported {
        info!("Opening file from Finder/Open With: {:?}", path);
        load_requests.write(ModelLoadRequest { path });
    }
    for path in unsupported {
        warn!("Finder passed unsupported format: {:?}", path);
    }
}

#[cfg(target_os = "macos")]
fn push_pending_path(path: PathBuf) {
    if path.as_os_str().is_empty() {
        return;
    }
    let mut pending = PENDING_MACOS_OPEN_FILES
        .lock()
        .expect("macOS open-file queue poisoned");
    if !pending.iter().any(|existing| existing == &path) {
        pending.push(path);
    }
}

#[cfg(target_os = "macos")]
unsafe fn push_pending_nsstring_path(ns_string: *mut objc::runtime::Object) {
    use objc::{msg_send, sel, sel_impl};

    if ns_string.is_null() {
        return;
    }

    let c_path: *const c_char = unsafe { msg_send![ns_string, UTF8String] };
    if c_path.is_null() {
        return;
    }

    let path = unsafe { CStr::from_ptr(c_path) }
        .to_string_lossy()
        .to_string();
    push_pending_path(PathBuf::from(path));
}

#[cfg(target_os = "macos")]
unsafe fn install_winit_delegate_open_file_hooks() {
    use std::ffi::CString;

    use objc::runtime::{class_addMethod, Class, Imp, Object, Sel, BOOL, NO, YES};
    use objc::{msg_send, sel, sel_impl};

    extern "C" fn application_open_file(
        _this: &Object,
        _cmd: Sel,
        _app: *mut Object,
        filename: *mut Object,
    ) -> BOOL {
        unsafe {
            push_pending_nsstring_path(filename);
        }
        YES
    }

    extern "C" fn application_open_files(
        _this: &Object,
        _cmd: Sel,
        app: *mut Object,
        filenames: *mut Object,
    ) {
        if filenames.is_null() {
            return;
        }

        unsafe {
            let count: usize = msg_send![filenames, count];
            for idx in 0..count {
                let filename: *mut Object = msg_send![filenames, objectAtIndex: idx];
                push_pending_nsstring_path(filename);
            }

            // NSApplicationDelegateReplySuccess = 0
            let _: () = msg_send![app, replyToOpenOrPrint: 0usize];
        }
    }

    extern "C" fn application_open_urls(
        _this: &Object,
        _cmd: Sel,
        _app: *mut Object,
        urls: *mut Object,
    ) {
        if urls.is_null() {
            return;
        }

        unsafe {
            let count: usize = msg_send![urls, count];
            for idx in 0..count {
                let url: *mut Object = msg_send![urls, objectAtIndex: idx];
                let is_file_url: BOOL = msg_send![url, isFileURL];
                if is_file_url == NO {
                    continue;
                }
                let path: *mut Object = msg_send![url, path];
                push_pending_nsstring_path(path);
            }
        }
    }

    extern "C" fn application_open_file_without_ui(
        _this: &Object,
        _cmd: Sel,
        _app: *mut Object,
        filename: *mut Object,
    ) -> BOOL {
        unsafe {
            push_pending_nsstring_path(filename);
        }
        YES
    }

    extern "C" fn application_open_temp_file(
        _this: &Object,
        _cmd: Sel,
        _app: *mut Object,
        filename: *mut Object,
    ) -> BOOL {
        unsafe {
            push_pending_nsstring_path(filename);
        }
        YES
    }

    if OPEN_HOOKS_INSTALLED.load(Ordering::SeqCst) {
        return;
    }

    let Some(delegate_cls) = Class::get("WinitApplicationDelegate") else {
        warn!("WinitApplicationDelegate not found; Finder open-file hooks not installed yet");
        return;
    };

    let delegate_cls = delegate_cls as *const Class as *mut Class;

    let open_file_imp: Imp = unsafe { std::mem::transmute(application_open_file as extern "C" fn(&Object, Sel, *mut Object, *mut Object) -> BOOL) };
    let open_file_types = CString::new("B@:@@").expect("valid Objective-C method type encoding");
    let added_open_file = unsafe {
        class_addMethod(
            delegate_cls,
            sel!(application:openFile:),
            open_file_imp,
            open_file_types.as_ptr(),
        )
    };

    let open_files_imp: Imp = unsafe { std::mem::transmute(application_open_files as extern "C" fn(&Object, Sel, *mut Object, *mut Object)) };
    let open_files_types = CString::new("v@:@@").expect("valid Objective-C method type encoding");
    let added_open_files = unsafe {
        class_addMethod(
            delegate_cls,
            sel!(application:openFiles:),
            open_files_imp,
            open_files_types.as_ptr(),
        )
    };

    let open_urls_imp: Imp = unsafe { std::mem::transmute(application_open_urls as extern "C" fn(&Object, Sel, *mut Object, *mut Object)) };
    let open_urls_types = CString::new("v@:@@").expect("valid Objective-C method type encoding");
    let added_open_urls = unsafe {
        class_addMethod(
            delegate_cls,
            sel!(application:openURLs:),
            open_urls_imp,
            open_urls_types.as_ptr(),
        )
    };

    let open_file_without_ui_imp: Imp = unsafe { std::mem::transmute(application_open_file_without_ui as extern "C" fn(&Object, Sel, *mut Object, *mut Object) -> BOOL) };
    let open_file_without_ui_types =
        CString::new("B@:@@").expect("valid Objective-C method type encoding");
    let _added_open_file_without_ui = unsafe {
        class_addMethod(
            delegate_cls,
            sel!(application:openFileWithoutUI:),
            open_file_without_ui_imp,
            open_file_without_ui_types.as_ptr(),
        )
    };

    let open_temp_file_imp: Imp = unsafe { std::mem::transmute(application_open_temp_file as extern "C" fn(&Object, Sel, *mut Object, *mut Object) -> BOOL) };
    let open_temp_file_types =
        CString::new("B@:@@").expect("valid Objective-C method type encoding");
    let _added_open_temp_file = unsafe {
        class_addMethod(
            delegate_cls,
            sel!(application:openTempFile:),
            open_temp_file_imp,
            open_temp_file_types.as_ptr(),
        )
    };

    if added_open_file == NO {
        warn!("application:openFile: already exists or failed to add");
    }
    if added_open_files == NO {
        warn!("application:openFiles: already exists or failed to add");
    }
    if added_open_urls == NO {
        warn!("application:openURLs: already exists or failed to add");
    }

    OPEN_HOOKS_INSTALLED.store(true, Ordering::SeqCst);
}

#[cfg(target_os = "macos")]
fn start_hook_watcher_thread() {
    thread::spawn(|| {
        for _attempt in 1..=400 {
            if OPEN_HOOKS_INSTALLED.load(Ordering::SeqCst) {
                return;
            }
            unsafe {
                install_winit_delegate_open_file_hooks();
            }
            thread::sleep(Duration::from_millis(5));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::partition_supported_paths;
    use std::path::PathBuf;

    #[test]
    fn partitions_supported_and_unsupported_extensions() {
        let input = vec![
            PathBuf::from("/tmp/a.fbx"),
            PathBuf::from("/tmp/b.gltf"),
            PathBuf::from("/tmp/c.txt"),
        ];

        let (supported, unsupported) = partition_supported_paths(input);

        assert_eq!(supported, vec![PathBuf::from("/tmp/a.fbx"), PathBuf::from("/tmp/b.gltf")]);
        assert_eq!(unsupported, vec![PathBuf::from("/tmp/c.txt")]);
    }
}
