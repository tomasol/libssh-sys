use libssh_sys_dylib::*;
use std::ffi::CString;
use std::ffi::CStr;
use std::convert::TryInto;
use std::os::raw::c_int;

#[test]
fn open_server_socket() {
    let ssh_bind :ssh_bind = unsafe { ssh_bind_new() };
    assert!(!ssh_bind.is_null());

    // set host to localhost
    set_bind_option(ssh_bind, ssh_bind_options_e_SSH_BIND_OPTIONS_BINDADDR, "localhost");
    // set port to 2222
    set_bind_option(ssh_bind, ssh_bind_options_e_SSH_BIND_OPTIONS_BINDPORT_STR, "2222");
    // set dsa private key

    set_bind_option(ssh_bind, ssh_bind_options_e_SSH_BIND_OPTIONS_RSAKEY, "tests/assets/id_rsa");
    set_bind_option(ssh_bind, ssh_bind_options_e_SSH_BIND_OPTIONS_DSAKEY, "tests/assets/id_dsa");
    // listen
    let res = unsafe { ssh_bind_listen(ssh_bind) };

    assert!(res == SSH_OK as c_int, "Error while ssh_bind_listen");

    // spawn clinet thread
    let handler = std::thread::spawn(run_client);

    let session: ssh_session = unsafe{ ssh_new() };
    assert!(!session.is_null());
    println!("Calling ssh_bind_accept");
    // accept will block until connected
    let res = unsafe { ssh_bind_accept(ssh_bind, session) };
    assert!(res == SSH_OK as c_int, "Error while ssh_bind_accept");
    println!("Calling ssh_handle_key_exchange");
    let res = unsafe { ssh_handle_key_exchange(session) };
    assert!(res == SSH_OK as c_int, "Error while ssh_handle_key_exchange");

    // handle auth
    let mut auth = false;
    while !auth {
        let message = unsafe {ssh_message_get(session) };
        if message.is_null() {
            auth = false;
            break;
        }

        match unsafe {ssh_message_type(message)}.try_into() {
            #![allow(non_upper_case_globals)]
            Ok(ssh_requests_e_SSH_REQUEST_AUTH) => {
                match unsafe {ssh_message_subtype(message)}.try_into() {
                    Ok(SSH_AUTH_METHOD_PASSWORD) => {
                        println!("Got SSH_AUTH_METHOD_PASSWORD");
                        let user = unsafe {ssh_message_auth_user(message)};
                        let pwd = unsafe {ssh_message_auth_password(message)};
                        if !user.is_null() && !pwd.is_null() {
                            let user: &CStr = unsafe { CStr::from_ptr(user) };
                            let pwd: &CStr = unsafe { CStr::from_ptr(pwd) };
                            println!("User {:?} wants to auth with pass {:?}",
                                user,
                                pwd);

                            let allowed: &CStr = &CString::new("foo").unwrap();
                            if user == allowed && pwd == allowed {
                                unsafe { ssh_message_auth_reply_success(message,0); }
                                auth = true;
                            }
                        }
                    },
                    // not authenticated, send default message
                    Ok(subtype) => {
                        println!("Got subtype {}", subtype);
                        unsafe {
                            ssh_message_auth_set_methods(message,
                                SSH_AUTH_METHOD_PASSWORD.try_into().unwrap());
                        }
                    },
                    Err(e) => {
                        println!("Got err {:?}", e);
                    }
                }
            },
            Ok(msg_type) => {
                println!("Got msg_type {:?}", msg_type);
            },
            Err(e) => {
                println!("Got err {:?}", e);
            }
        }
        unsafe {
            ssh_message_reply_default(message);
            ssh_message_free(message);
        }
    }
    assert!(auth, "Auth error");

    unsafe {
        ssh_disconnect(session);
        ssh_free(session);
        ssh_bind_free(ssh_bind);
        ssh_finalize();
    }
    handler.join().unwrap();
}

fn run_client() {
    println!("Starting ssh clinet");
    let session: ssh_session = unsafe{ ssh_new() };
    assert!(!session.is_null());
    set_session_option(session, ssh_options_e_SSH_OPTIONS_HOST, "localhost");
    set_session_option(session, ssh_options_e_SSH_OPTIONS_PORT_STR, "2222");
    let res = unsafe { ssh_connect(session) };
    assert!(res == SSH_OK as c_int, "Error while ssh_connect");
    println!("Clinet connected");
    let res = unsafe { ssh_userauth_password(session,
        CString::new("foo").unwrap().as_ptr(),
        CString::new("foo").unwrap().as_ptr()) };
    assert!(res == SSH_OK as c_int, "Error while ssh_userauth_password");
    println!("Clinet auth ok");
    unsafe {
        ssh_disconnect(session);
        ssh_free(session);
    }
}

// util
fn set_session_option(session :ssh_session, key: ssh_options_e, value: &str) {
    let c_str = CString::new(value).expect("CString::new failed");
    unsafe {
        ssh_options_set(session, key,
            c_str.as_ptr() as *const std::os::raw::c_void);
    };
}
fn set_bind_option(ssh_bind :ssh_bind, key: ssh_bind_options_e, value: &str) {
    let c_str = CString::new(value).expect("CString::new failed");
    unsafe {
        ssh_bind_options_set(ssh_bind, key,
            c_str.as_ptr() as *const std::os::raw::c_void);
    };
}
