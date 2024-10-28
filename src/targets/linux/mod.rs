use std::ffi::{CStr, CString, NulError};

use super::Target;

use x11::xlib::{XFreeStringList, XGetTextProperty, XTextProperty, XmbTextPropertyToTextList};
use xcb::{
    randr::{Connection, GetCrtcInfo, GetOutputInfo, GetScreenResources},
    x::{self, GetPropertyReply},
    Xid,
};

fn get_atom(conn: &xcb::Connection, atom_name: &str) -> Result<x::Atom, xcb::Error> {
    let cookie = conn.send_request(&x::InternAtom {
        only_if_exists: true,
        name: atom_name.as_bytes(),
    });
    Ok(conn.wait_for_reply(cookie)?.atom())
}

fn get_property(
    conn: &xcb::Connection,
    win: x::Window,
    prop: x::Atom,
    typ: x::Atom,
    length: u32,
) -> Result<GetPropertyReply, xcb::Error> {
    let cookie = conn.send_request(&x::GetProperty {
        delete: false,
        window: win,
        property: prop,
        r#type: typ,
        long_offset: 0,
        long_length: length,
    });
    Ok(conn.wait_for_reply(cookie)?)
}

fn decode_compound_text(
    conn: &xcb::Connection,
    value: &[u8],
    client: &xcb::x::Window,
    ttype: xcb::x::Atom,
) -> Result<String, NulError> {
    let display = conn.get_raw_dpy();
    assert!(!display.is_null());

    let c_string = CString::new(value.to_vec())?;
    let mut fuck = XTextProperty {
        value: std::ptr::null_mut(),
        encoding: 0,
        format: 0,
        nitems: 0,
    };
    let res = unsafe {
        XGetTextProperty(
            display,
            client.resource_id() as u64,
            &mut fuck,
            x::ATOM_WM_NAME.resource_id() as u64,
        )
    };
    if res == 0 || fuck.nitems == 0 {
        return Ok(String::from("n/a"));
    }

    let mut xname = XTextProperty {
        value: c_string.as_ptr() as *mut u8,
        encoding: ttype.resource_id() as u64,
        format: 8,
        nitems: fuck.nitems,
    };
    let mut list: *mut *mut i8 = std::ptr::null_mut();
    let mut count: i32 = 0;
    let result = unsafe { XmbTextPropertyToTextList(display, &mut xname, &mut list, &mut count) };
    if result < 1 || list.is_null() || count < 1 {
        Ok(String::from("n/a"))
    } else {
        let title = unsafe { CStr::from_ptr(*list).to_string_lossy().into_owned() };
        unsafe { XFreeStringList(list) };
        Ok(title)
    }
}

pub fn get_all_targets() -> Vec<Target> {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        // On Wayland, the target is selected when a Recorder is instanciated because it requires user interaction
        Vec::new()
    } else if std::env::var("DISPLAY").is_ok() {
        let (conn, _screen_num) = xcb::Connection::connect_with_xlib_display_and_extensions(
            &[xcb::Extension::RandR],
            &[],
        )
        .unwrap();
        let setup = conn.get_setup();
        let screens = setup.roots();

        let wm_client_list = get_atom(&conn, "_NET_CLIENT_LIST").unwrap();
        assert!(wm_client_list != x::ATOM_NONE, "EWMH not supported");

        let atom_net_wm_name = get_atom(&conn, "_NET_WM_NAME").unwrap();
        let atom_text = get_atom(&conn, "TEXT").unwrap();
        let atom_utf8_string = get_atom(&conn, "UTF8_STRING").unwrap();
        let atom_compound_text = get_atom(&conn, "COMPOUND_TEXT").unwrap();

        let mut targets = Vec::new();
        for screen in screens {
            let window_list =
                get_property(&conn, screen.root(), wm_client_list, x::ATOM_NONE, 100).unwrap();

            for client in window_list.value::<x::Window>() {
                let cr =
                    get_property(&conn, *client, atom_net_wm_name, x::ATOM_STRING, 4096).unwrap();
                if !cr.value::<x::Atom>().is_empty() {
                    targets.push(Target::Window(crate::targets::Window {
                        id: 0,
                        title: String::from_utf8(cr.value().to_vec()).unwrap(),
                        raw_handle: *client,
                    }));
                    continue;
                }

                let reply =
                    get_property(&conn, *client, x::ATOM_WM_NAME, x::ATOM_ANY, 4096).unwrap();
                let value: &[u8] = reply.value();
                if !value.is_empty() {
                    let ttype = reply.r#type();
                    let title = if ttype == x::ATOM_STRING
                        || ttype == atom_utf8_string
                        || ttype == atom_text
                    {
                        String::from_utf8(reply.value().to_vec()).unwrap_or(String::from("n/a"))
                    } else if ttype == atom_compound_text {
                        decode_compound_text(&conn, value, client, ttype).unwrap()
                    } else {
                        String::from_utf8(reply.value().to_vec()).unwrap_or(String::from("n/a"))
                    };

                    targets.push(Target::Window(crate::targets::Window {
                        id: 0,
                        title,
                        raw_handle: *client,
                    }));
                    continue;
                }
                targets.push(Target::Window(crate::targets::Window {
                    id: 0,
                    title: String::from("n/a"),
                    raw_handle: *client,
                }));
            }

            let resources = conn.send_request(&GetScreenResources {
                window: screen.root(),
            });
            let resources = conn.wait_for_reply(resources).unwrap();
            for output in resources.outputs() {
                let info = conn.send_request(&GetOutputInfo {
                    output: *output,
                    config_timestamp: 0,
                });
                let info = conn.wait_for_reply(info).unwrap();
                if info.connection() == Connection::Connected {
                    let crtc = info.crtc();
                    crtc.resource_id();
                    let crtc_info = conn.send_request(&GetCrtcInfo {
                        crtc,
                        config_timestamp: 0,
                    });
                    let crtc_info = conn.wait_for_reply(crtc_info).unwrap();
                    let title =
                        String::from_utf8(info.name().to_vec()).unwrap_or(String::from("n/a"));
                    targets.push(Target::Display(crate::targets::Display {
                        id: crtc.resource_id(),
                        title,
                        width: crtc_info.width(),
                        height: crtc_info.height(),
                        x_offset: crtc_info.x(),
                        y_offset: crtc_info.y(),
                    }));
                }
            }
        }

        targets
    } else {
        panic!("Unsupported platform. Could not detect Wayland or X11 displays")
    }
}
