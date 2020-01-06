use libssh_sys_dylib::*;
use std::convert::TryInto;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::os::raw::c_int;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn check_username_password(username: &CStr, password: &CStr) -> bool {
    println!("User {:?} wants to auth with pass {:?}", username, password);
    let allowed: &CStr = &CString::new("foo").unwrap();
    username == allowed && password == allowed
}

fn authenticate(session: ssh_session) -> bool {
    #![allow(non_upper_case_globals)]
    loop {
        let message = unsafe { ssh_message_get(session) };
        if message.is_null() {
            return false;
        }
        let msg_type = unsafe { ssh_message_type(message) }.try_into();
        if msg_type == Ok(ssh_requests_e_SSH_REQUEST_AUTH) {
            let msg_subtype = unsafe { ssh_message_subtype(message) }.try_into();
            if msg_subtype == Ok(SSH_AUTH_METHOD_PASSWORD) {
                println!("Got SSH_AUTH_METHOD_PASSWORD");
                let user = unsafe { ssh_message_auth_user(message) };
                let pwd = unsafe { ssh_message_auth_password(message) };
                if !user.is_null() && !pwd.is_null() {
                    let user: &CStr = unsafe { CStr::from_ptr(user) };
                    let pwd: &CStr = unsafe { CStr::from_ptr(pwd) };
                    if check_username_password(user, pwd) {
                        unsafe {
                            ssh_message_auth_reply_success(message, 0);
                            ssh_message_free(message);
                        }
                        return true;
                    }
                }
            }
        }
        unsafe {
            ssh_message_auth_set_methods(message, SSH_AUTH_METHOD_PASSWORD.try_into().unwrap());
            ssh_message_reply_default(message);
            ssh_message_free(message);
        }
    }
}

fn wait_for_channel(session: ssh_session) -> Option<ssh_channel> {
    #![allow(non_upper_case_globals)]
    loop {
        let message = unsafe { ssh_message_get(session) };
        if message.is_null() {
            return None;
        }
        println!("ssh_message_type");
        let msg_type = unsafe { ssh_message_type(message) }.try_into();
        println!("ssh_message_type {:?}", msg_type);

        if msg_type == Ok(ssh_requests_e_SSH_REQUEST_CHANNEL_OPEN) {
            let msg_subtype = unsafe { ssh_message_subtype(message) }.try_into();
            if msg_subtype == Ok(ssh_channel_type_e_SSH_CHANNEL_SESSION) {
                let chan = unsafe { ssh_message_channel_request_open_reply_accept(message) };
                unsafe {
                    ssh_message_free(message);
                }
                return Some(chan);
            }
        }
        println!("reply_default");
        unsafe {
            ssh_message_reply_default(message);
            ssh_message_free(message);
        }
    }
}

fn wait_for_pty_shell(session: ssh_session) -> bool {
    #![allow(non_upper_case_globals)]
    loop {
        let message = unsafe { ssh_message_get(session) };
        if message.is_null() {
            return false;
        }
        println!("ssh_message_type");
        let msg_type = unsafe { ssh_message_type(message) }.try_into();
        println!("ssh_message_type {:?}", msg_type);
        if msg_type == Ok(ssh_requests_e_SSH_REQUEST_CHANNEL) {
            let msg_subtype = unsafe { ssh_message_subtype(message) }.try_into();
            if msg_subtype == Ok(ssh_channel_requests_e_SSH_CHANNEL_REQUEST_SHELL) {
                println!("SSH_CHANNEL_REQUEST_SHELL");
                unsafe {
                    ssh_message_channel_request_reply_success(message);
                    ssh_message_free(message);
                }
                return true;
            } else if msg_subtype == Ok(ssh_channel_requests_e_SSH_CHANNEL_REQUEST_PTY) {
                println!("SSH_CHANNEL_REQUEST_PTY");
                unsafe {
                    ssh_message_channel_request_reply_success(message);
                    ssh_message_free(message);
                }
                continue;
            }
        }
        println!("reply_default");
        unsafe {
            ssh_message_reply_default(message);
            ssh_message_free(message);
        }
    }
}

fn check_file_exists(file: &str) -> Result<&str> {
    File::open(file).map_err(|x| format!("{}: {}", x, file))?;
    Ok(file)
}

#[test]
fn open_server_socket() -> Result<()> {
    let ssh_bind: ssh_bind = unsafe { ssh_bind_new() };
    assert!(ssh_bind.is_null() == false, "Cannot create bind");

    // set host to localhost
    set_bind_option(
        ssh_bind,
        ssh_bind_options_e_SSH_BIND_OPTIONS_BINDADDR,
        "localhost",
    );
    // set port to 2222
    set_bind_option(
        ssh_bind,
        ssh_bind_options_e_SSH_BIND_OPTIONS_BINDPORT_STR,
        "2222",
    );
    // set dsa private key

    set_bind_option(
        ssh_bind,
        ssh_bind_options_e_SSH_BIND_OPTIONS_RSAKEY,
        check_file_exists("tests/assets/id_rsa")?,
    );
    set_bind_option(
        ssh_bind,
        ssh_bind_options_e_SSH_BIND_OPTIONS_DSAKEY,
        check_file_exists("tests/assets/id_dsa")?,
    );
    // listen
    let res = unsafe { ssh_bind_listen(ssh_bind) };

    assert!(res == SSH_OK as c_int, "Error while ssh_bind_listen");

    // spawn clinet thread
    let handler = std::thread::spawn(run_client);

    let session: ssh_session = unsafe { ssh_new() };
    assert!(session.is_null() == false, "Server session is null");
    println!("Calling ssh_bind_accept");
    // accept will block until connected
    let res = unsafe { ssh_bind_accept(ssh_bind, session) };
    assert!(res == SSH_OK as c_int, "Error while ssh_bind_accept");
    println!("Calling ssh_handle_key_exchange");
    let res = unsafe { ssh_handle_key_exchange(session) };
    assert!(
        res == SSH_OK as c_int,
        "Error while ssh_handle_key_exchange"
    );

    // handle auth
    let auth = authenticate(session);
    assert!(auth, "Auth error");

    // wait for requesting channel, shell, pty
    let ch = wait_for_channel(session);
    assert!(ch.is_some(), "Channel not opened");

    let pty = wait_for_pty_shell(session);
    assert!(pty, "Shell not requested");
    println!("It works!");

    unsafe {
        ssh_disconnect(session);
        ssh_free(session);
        ssh_bind_free(ssh_bind);
        ssh_finalize();
    }
    handler.join().unwrap();
    Ok(())
}

fn run_client() {
    println!("Starting ssh clinet");
    let session: ssh_session = unsafe { ssh_new() };
    assert!(session.is_null() == false, "Clinet session is null");
    set_session_option(session, ssh_options_e_SSH_OPTIONS_HOST, "localhost");
    set_session_option(session, ssh_options_e_SSH_OPTIONS_PORT_STR, "2222");
    let res = unsafe { ssh_connect(session) };
    assert!(res == SSH_OK as c_int, "Error while ssh_connect");
    println!("Clinet connected");
    let res = unsafe {
        ssh_userauth_password(
            session,
            CString::new("foo").unwrap().as_ptr(),
            CString::new("foo").unwrap().as_ptr(),
        )
    };
    assert!(res == SSH_OK as c_int, "Error while ssh_userauth_password");
    println!("Clinet auth ok");

    let channel = unsafe { ssh_channel_new(session) };
    assert!(channel.is_null() == false, "Cannot open clinet channel");

    let res = unsafe { ssh_channel_open_session(channel) };
    assert!(res == SSH_OK as c_int, "Error while opening client session");

    println!("Clinet about to request pty");
    // request pty, shell
    let res = unsafe { ssh_channel_request_pty(channel) };
    assert!(res == SSH_OK as c_int, "Error while requesting pty");
    println!("Clinet about to request shell");
    let res = unsafe { ssh_channel_request_shell(channel) };
    assert!(res == SSH_OK as c_int, "Error while requesting shell");

    unsafe {
        ssh_channel_free(channel);
        ssh_disconnect(session);
        ssh_free(session);
    }
}

// util
fn set_session_option(session: ssh_session, key: ssh_options_e, value: &str) {
    let c_str = CString::new(value).expect("CString::new failed");
    unsafe {
        ssh_options_set(session, key, c_str.as_ptr() as *const std::os::raw::c_void);
    };
}
fn set_bind_option(ssh_bind: ssh_bind, key: ssh_bind_options_e, value: &str) {
    let c_str = CString::new(value).expect("CString::new failed");
    unsafe {
        ssh_bind_options_set(ssh_bind, key, c_str.as_ptr() as *const std::os::raw::c_void);
    };
}
