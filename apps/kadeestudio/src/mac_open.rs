//! macOS: the standard Open panel with a "File type:" menu under it (All supported documents,
//! Markdown documents, PDF documents …).
//!
//! rfd merges all filters into one list on macOS, so the panel is built here, the way mdcraft
//! builds its Save panel: an accessory view with a pop-up button whose action switches the
//! extensions the panel lets you pick. AppKit is Objective-C, hence the unsafe blocks.
#![allow(unsafe_code)]

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSOpenPanel, NSPopUpButton, NSSavePanel, NSTextField, NSView};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize, NSString, NSURL};
use std::path::{Path, PathBuf};

struct Ivars {
    panel: Retained<NSOpenPanel>,
    popup: Retained<NSPopUpButton>,
    /// Extensions per entry of the menu, without the dot.
    types: Vec<Vec<String>>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements; the class doesn't implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "KadeeOpenFileTypeTarget"]
    #[ivars = Ivars]
    struct FileTypeTarget;

    impl FileTypeTarget {
        #[unsafe(method(fileTypeChanged:))]
        fn file_type_changed(&self, _sender: Option<&AnyObject>) {
            let iv = self.ivars();
            allow(&iv.panel, &iv.types, iv.popup.indexOfSelectedItem());
        }
    }
);

impl FileTypeTarget {
    fn new(mtm: MainThreadMarker, ivars: Ivars) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ivars);
        // SAFETY: NSObject's designated initializer.
        unsafe { msg_send![super(this), init] }
    }
}

/// Lets the panel pick the files of menu entry `index`.
#[allow(deprecated)] // allowedContentTypes needs UTType objects; the extension list is enough here
fn allow(panel: &NSSavePanel, types: &[Vec<String>], index: isize) {
    let Some(exts) = types.get(index.max(0) as usize) else { return };
    let list: Vec<Retained<NSString>> = exts.iter().map(|e| NSString::from_str(e)).collect();
    panel.setAllowedFileTypes(Some(&NSArray::from_retained_slice(&list)));
}

/// Shows the Open panel (app-modal) with `types` (label, extensions) in its File type menu.
/// Returns the chosen file.
pub fn open_panel(title: &str, dir: Option<&Path>, types: &[(String, Vec<String>)]) -> Option<PathBuf> {
    let mtm = MainThreadMarker::new()?;
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setTitle(Some(&NSString::from_str(title)));
    panel.setCanChooseFiles(true);
    panel.setCanChooseDirectories(false);
    panel.setAllowsMultipleSelection(false);

    // the accessory view: "File type: [All supported documents  ⌄]"
    let view = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(460.0, 46.0)));
    let label = NSTextField::labelWithString(&NSString::from_str("File type:"), mtm);
    label.setFrame(NSRect::new(NSPoint::new(24.0, 14.0), NSSize::new(76.0, 18.0)));
    let popup = NSPopUpButton::initWithFrame_pullsDown(
        NSPopUpButton::alloc(mtm),
        NSRect::new(NSPoint::new(102.0, 9.0), NSSize::new(330.0, 26.0)),
        false,
    );
    for (name, _) in types {
        popup.addItemWithTitle(&NSString::from_str(name));
    }
    popup.selectItemAtIndex(0);
    view.addSubview(&label);
    view.addSubview(&popup);
    panel.setAccessoryView(Some(&view));
    // the Open panel hides its accessory view behind an "Options" button unless told otherwise
    panel.setAccessoryViewDisclosed(true);

    let ivars = Ivars { panel: panel.clone(), popup: popup.clone(), types: types.iter().map(|(_, e)| e.clone()).collect() };
    let target = FileTypeTarget::new(mtm, ivars);
    let target_obj: &AnyObject = &target;
    // SAFETY: the target is a live object that answers fileTypeChanged:, kept alive until the
    // panel is closed (a control does not retain its target).
    unsafe {
        popup.setTarget(Some(target_obj));
        popup.setAction(Some(sel!(fileTypeChanged:)));
    }
    allow(&panel, &target.ivars().types, 0);
    if let Some(d) = dir {
        panel.setDirectoryURL(Some(&NSURL::fileURLWithPath(&NSString::from_str(&d.to_string_lossy()))));
    }

    let response = panel.runModal();
    drop(target);
    if response != 1 {
        return None; // NSModalResponseOK = 1
    }
    let path = panel.URL()?.path()?.to_string();
    Some(PathBuf::from(path))
}
