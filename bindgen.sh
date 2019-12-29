# /usr/include/libssh/libssh.h
bindgen $(pwd)/wrapper.h -o src/lib.rs  --raw-line '#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]'
# IPPORT_RESERVED is generated twice
sed -i 's/pub const IPPORT_RESERVED: _bindgen_ty_9 = 1024;//g' src/lib.rs
