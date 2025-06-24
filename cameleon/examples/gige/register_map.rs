use std::net::Ipv4Addr;

use cameleon::gige::enumerate_cameras;

fn main() {
    let localhost_v4 = Ipv4Addr::new(10, 227, 12, 10);
    let mut cameras = enumerate_cameras(localhost_v4).unwrap();
    if cameras.is_empty() {
        println!("no camera found!");
        return;
    }

    let mut camera = cameras.pop().unwrap();
    camera.open().unwrap();
    camera.load_context().unwrap();

    camera.close().unwrap();
}
