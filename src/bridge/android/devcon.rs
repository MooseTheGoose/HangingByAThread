pub fn devcon_loop() {
    let listener_result = {
        let unixpath = internal_path().join("devcon");
        let _ = remove_file(unixpath.as_path()); 
        UnixListener::bind(unixpath.as_path())
    };
    match listener_result {
        Ok(listener) => {
            info!("Waiting for devcon clients...");
            for stream_result in listener.incoming() {
               match stream_result {
                   Ok(stream) => {
                       let fd = stream.as_raw_fd();
                       unsafe {
                           dup2(fd, 2);
                           dup2(fd, 1);
                           dup2(fd, 0);
                       }
                       conmain();
                   },
                   Err(_) => { 
                       warn!("Connection to devcon didn't succeed...");
                   }
               } 
            }
        },
        Err(e) => {
            error!("Unable to create UNIX socket for devcon! {:?}", e);
        }
    };
}
