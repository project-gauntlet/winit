use std::ptr::NonNull;

use icrate::Foundation::NSObject;
use objc2::declare::{IvarBool, IvarEncode};
use objc2::rc::Id;
use objc2::runtime::AnyObject;
use objc2::{class, declare_class, msg_send, msg_send_id, mutability, sel, ClassType};

use crate::event::{Event, MacOS, PlatformSpecific};

use super::app_state::AppState;
use super::appkit::NSApplicationActivationPolicy;

/// Apple constants
#[allow(non_upper_case_globals)]
pub const kInternetEventClass: u32 = 0x4755524c;
#[allow(non_upper_case_globals)]
pub const kAEGetURL: u32 = 0x4755524c;
#[allow(non_upper_case_globals)]
pub const keyDirectObject: u32 = 0x2d2d2d2d;

declare_class!(
    #[derive(Debug)]
    pub(super) struct ApplicationDelegate {
        activation_policy: IvarEncode<NSApplicationActivationPolicy, "_activation_policy">,
        default_menu: IvarBool<"_default_menu">,
        activate_ignoring_other_apps: IvarBool<"_activate_ignoring_other_apps">,
    }

    mod ivars;

    unsafe impl ClassType for ApplicationDelegate {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "WinitApplicationDelegate";
    }

    unsafe impl ApplicationDelegate {
        #[method(initWithActivationPolicy:defaultMenu:activateIgnoringOtherApps:)]
        unsafe fn init(
            this: *mut Self,
            activation_policy: NSApplicationActivationPolicy,
            default_menu: bool,
            activate_ignoring_other_apps: bool,
        ) -> Option<NonNull<Self>> {
            let this: Option<&mut Self> = unsafe { msg_send![super(this), init] };
            this.map(|this| {
                *this.activation_policy = activation_policy;
                *this.default_menu = default_menu;
                *this.activate_ignoring_other_apps = activate_ignoring_other_apps;
                NonNull::from(this)
            })
        }

        #[method(applicationDidFinishLaunching:)]
        fn did_finish_launching(&self, _sender: Option<&AnyObject>) {
            trace_scope!("applicationDidFinishLaunching:");
            AppState::launched(
                *self.activation_policy,
                *self.default_menu,
                *self.activate_ignoring_other_apps,
            );
        }

        #[method(applicationWillFinishLaunching:)]
        fn will_finish_launching(&self, _sender: Option<&AnyObject>) {
            trace_scope!("applicationWillFinishLaunching");

            unsafe {
                let event_manager = class!(NSAppleEventManager);
                let shared_manager: *mut AnyObject =
                    msg_send![event_manager, sharedAppleEventManager];

                let () = msg_send![shared_manager,
                    setEventHandler: self
                    andSelector: sel!(handleEvent:withReplyEvent:)
                    forEventClass: kInternetEventClass
                    andEventID: kAEGetURL
                ];
            }
        }

        #[method(handleEvent:withReplyEvent:)]
        fn handle_url(&self, event: *mut AnyObject, _reply: u64) {
            if let Some(string) = parse_url(event) {
                AppState::queue_event(Event::PlatformSpecific(PlatformSpecific::MacOS(
                    MacOS::ReceivedUrl(string),
                )));
            }
        }

        #[method(applicationWillTerminate:)]
        fn will_terminate(&self, _sender: Option<&AnyObject>) {
            trace_scope!("applicationWillTerminate:");
            // TODO: Notify every window that it will be destroyed, like done in iOS?
            AppState::internal_exit();
        }
    }
);

impl ApplicationDelegate {
    pub(super) fn new(
        activation_policy: NSApplicationActivationPolicy,
        default_menu: bool,
        activate_ignoring_other_apps: bool,
    ) -> Id<Self> {
        unsafe {
            msg_send_id![
                Self::alloc(),
                initWithActivationPolicy: activation_policy,
                defaultMenu: default_menu,
                activateIgnoringOtherApps: activate_ignoring_other_apps,
            ]
        }
    }
}

fn parse_url(event: *mut AnyObject) -> Option<String> {
    unsafe {
        let class: u32 = msg_send![event, eventClass];
        let id: u32 = msg_send![event, eventID];
        if class != kInternetEventClass || id != kAEGetURL {
            return None;
        }
        let subevent: *mut AnyObject = msg_send![event, paramDescriptorForKeyword: keyDirectObject];
        let nsstring: *mut AnyObject = msg_send![subevent, stringValue];
        let cstr: *const i8 = msg_send![nsstring, UTF8String];
        if !cstr.is_null() {
            Some(
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
        }
    }
}
